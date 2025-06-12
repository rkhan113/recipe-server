// recipe.rs

use std::path::Path;
use std::collections::HashSet;
use std::ops::Deref;

use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::{AppState};
use crate::error::RecipeError;
use utoipa::ToSchema;

/// Core recipe model representing the database record
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub ingredients: Vec<String>,
    pub instructions: String,
    pub tags: Option<Vec<String>>,
    pub source: Option<String>,
}

/// Returns a fallback recipe for use when the database is empty or unavailable.
pub fn fallback_recipe() -> Recipe {
    Recipe {
        id: "fallback".to_string(),
        name: "PB&J Sandwich".to_string(),
        ingredients: vec![
            "2 slices of bread".to_string(),
            "2 tbsp peanut butter".to_string(),
            "2 tbsp jelly".to_string(),
        ],
        instructions: "Spread peanut butter on one slice of bread. Spread jelly on the other. Combine. Enjoy!".to_string(),
        tags: Some(vec!["quick".to_string(), "easy".to_string(), "no-cook".to_string()]),
        source: Some("Roha Khan".to_string()),
    }
}

/// Load all recipes from a JSON file into a Vec<Recipe>
pub fn read_recipes<P: AsRef<Path>>(recipes_path: P) -> Result<Vec<Recipe>, RecipeError> {
    let f = std::fs::File::open(recipes_path.as_ref())?;
    let recipes = serde_json::from_reader(f)?;
    Ok(recipes)
}

/// Struct for API JSON response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct JsonRecipe {
    id: String,
    name: String,
    ingredients: Vec<String>,
    instructions: String,
    tags: HashSet<String>,
    source: Option<String>,
}

impl JsonRecipe {
    pub fn new(recipe: Recipe, tags: Vec<String>) -> Self {
        let tag_set = tags.into_iter().collect();
        Self {
            id: recipe.id,
            name: recipe.name,
            ingredients: recipe.ingredients,
            instructions: recipe.instructions,
            tags: tag_set,
            source: recipe.source,
        }
    }

    pub fn to_recipe(&self) -> (Recipe, impl Iterator<Item = &str>) {
        let recipe = Recipe {
            id: self.id.clone(),
            name: self.name.clone(),
            ingredients: self.ingredients.clone(),
            instructions: self.instructions.clone(),
            tags: Some(self.tags.iter().cloned().collect()),
            source: self.source.clone(),
        };
        let tags = self.tags.iter().map(String::deref);
        (recipe, tags)
    }
}

impl IntoResponse for JsonRecipe {
    fn into_response(self) -> Response {
        (axum::http::StatusCode::OK, Json(self)).into_response()
    }
}

/// Fetch a recipe and its tags for use in API response
pub async fn get(db: &SqlitePool, recipe_id: &str) -> Result<(Recipe, Vec<String>), sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT id, name, ingredients, instructions, tags, source
        FROM recipes
        WHERE id = ?
        "#,
        recipe_id
    )
    .fetch_one(db)
    .await?;

    let recipe = Recipe {
        id: row.id.expect("Missing id"),
        name: row.name,
        ingredients: serde_json::from_str(&row.ingredients).unwrap(),
        instructions: row.instructions,
        tags: row.tags.clone().map(|t| serde_json::from_str(&t).unwrap()),
        source: row.source,
    };

    let tags = recipe.tags.clone().unwrap_or_default();

    Ok((recipe, tags))
}

/// Fetch a random recipe ID from the database
pub async fn get_random(db: &SqlitePool) -> Result<String, sqlx::Error> {
    let recipe_result = sqlx::query_scalar!(
        "SELECT id FROM recipes ORDER BY RANDOM() LIMIT 1;"
    )
    .fetch_one(db)
    .await?
    .ok_or_else(|| sqlx::Error::RowNotFound)?; 
    Ok(recipe_result)
}
