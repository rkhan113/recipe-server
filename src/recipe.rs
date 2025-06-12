// recipes.rs

use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::RecipeError;
// use once_cell::sync::Lazy;


#[derive(Debug, Deserialize, Serialize)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub ingredients: Vec<String>,
    pub instructions: String,
    pub tags: Option<Vec<String>>,
    pub source: Option<String>,
}

/*
// Keeping a hardcoded recipe for fallback/testing
// lazy-initialized (nothing else was working)
pub static THE_RECIPE: Lazy<Recipe> = Lazy::new(|| Recipe {
    name: "PB&J Sandwich".to_string(),
    ingredients: vec![
        "2 slices of bread".to_string(),
        "2 tbsp peanut butter".to_string(),
        "2 tbsp jelly".to_string(),
    ],
    instructions: "Spread peanut butter on one slice of bread. Spread jelly on the other. Combine. Enjoy!".to_string(),
    tags: Some(vec!["quick".to_string(), "easy".to_string(), "no-cook".to_string()]),
    source: Some("Roha Khan".to_string()),
});
i*/

// Load all recipes from JSON file
pub fn read_recipes<P: AsRef<Path>>(recipes_path: P) -> Result<Vec<Recipe>, RecipeError> {
    let f = std::fs::File::open(recipes_path.as_ref())?;
    let recipes = serde_json::from_reader(f)?;
    Ok(recipes)
}
