use serde::Deserialize;
use once_cell::sync::Lazy;

#[derive(Deserialize)]

// Recipe struct to represent a recipe
pub struct Recipe {
    pub name: &'static str,
    pub ingredients: Vec<&'static str>, // CHANGED: now a vector of strings
    pub instructions: &'static str,
}

// A hardcoded example recipe
// lazy-initialized (nothing else was working)
pub static THE_RECIPE: Lazy<Recipe> = Lazy::new(|| Recipe {
    name: "PB&J Sandwich",
    ingredients: vec![
        "2 slices of bread",
        "2 tbsp peanut butter",
        "2 tbsp jelly",
    ],
    instructions: "Spread peanut butter on one slice of bread. Spread jelly on the other. Combine. Enjoy!",
});

