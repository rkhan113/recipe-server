// Bring in Askama templating
use askama::Template;

// Import Recipe struct from recipe.rs
use crate::recipe::Recipe;

// Define a template struct that references index.html
#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub recipe: &'a Recipe,          // Recipe to display
    pub stylesheet: &'static str,    // Path to CSS file
}

impl<'a> IndexTemplate<'a> {
    // Helper to create an IndexTemplate from a recipe
    pub fn new(recipe: &'a Recipe) -> Self {
        Self {
            recipe,
            stylesheet: "/recipe.css",
        }
    }
}
