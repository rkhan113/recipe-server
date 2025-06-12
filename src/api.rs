use crate::*;
use axum::{
    extract::{Path, State},
    http::{self, StatusCode},
    response::{self, IntoResponse, Response},
    Json,
};
use std::sync::Arc;
use tokio::sync::RwLock;

/// A JSON-safe version of a recipe (no need to wrap Option fields manually)
#[derive(serde::Serialize)]
pub struct JsonRecipe {
    id: String,
    name: String,
    ingredients: Vec<String>,
    instructions: String,
    tags: Option<Vec<String>>,
    source: Option<String>,
}

impl From<Recipe> for JsonRecipe {
    fn from(r: Recipe) -> Self {
        JsonRecipe {
            id: r.id,
            name: r.name,
            ingredients: r.ingredients,
            instructions: r.instructions,
            tags: r.tags,
            source: r.source,
        }
    }
}

/// API route handler: returns a recipe in JSON by ID
pub async fn get_recipe(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(recipe_id): Path<String>,
) -> Result<Response, StatusCode> {
    let app_reader = app_state.read().await;
    let db = &app_reader.db;

    let row = sqlx::query!(
        r#"SELECT id, name, ingredients, instructions, tags, source FROM recipes WHERE id = ?"#,
        recipe_id
    )
    .fetch_one(db)
    .await;

    match row {
        Ok(row) => {
            let recipe = Recipe {
                id: row.id.expect("Missing id"),
                name: row.name,
                ingredients: serde_json::from_str(&row.ingredients).unwrap_or_default(),
                instructions: row.instructions,
                tags: row.tags.map(|t| serde_json::from_str(&t).unwrap_or_default()),
                source: row.source,
            };
            let json_recipe = JsonRecipe::from(recipe);
            Ok(Json(json_recipe).into_response())
        }
        Err(e) => {
            log::warn!("Recipe fetch failed: {}", e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}
