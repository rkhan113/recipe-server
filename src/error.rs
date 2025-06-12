// error.rs
extern crate serde_json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RecipeError {
    #[error("could not find recipe file: {0}")]
    RecipeFileNotFound(#[from] std::io::Error),

    #[error("could not read recipe file: {0}")]
    RecipeMisformat(#[from] serde_json::Error),
}
