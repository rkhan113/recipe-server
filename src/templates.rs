// Import Recipe struct from recipe.rs
use crate::recipe::Recipe;

// Bring in Askama templating
use askama::Template;

// Define a template struct that references index.html
#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    recipe: Recipe,          // Recipe to display
    stylesheet: &'static str,    // Path to CSS file
    tags: String,
}

impl IndexTemplate {
    // Helper to create an IndexTemplate from a recipe
    pub fn new(recipe: Recipe, tags: String) -> Self {
        Self {
            recipe,
            stylesheet: "/recipe.css",
            tags,
        }
    }
}
