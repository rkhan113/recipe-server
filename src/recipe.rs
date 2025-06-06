// Recipe struct to represent a recipe
pub struct Recipe {
    pub name: &'static str,
    pub ingredients: &'static str,
    pub instructions: &'static str,
}

// A hardcoded example recipe
pub const THE_RECIPE: Recipe = Recipe {
    name: "PB&J Sandwich",
    ingredients: "Bread, peanut butter, jelly",
    instructions: "Spread peanut butter on bread. Spread jelly on another bread slice. Put them together. Enjoy!",
};
