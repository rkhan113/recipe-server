use axum::{self, response, routing}; // brings in Axum components to return raw HTML content & set up GET routes
use tokio::net; // brings in TCPListener - binds to IP + port for our server

// will be hardcoding some test data for now
struct Recipe {
    name: &'static str,
    ingreds: &'static str, // ingredients
    instrucs: &'static str, // instructions
}

// this is the actual data
const THE_RECIPE: Recipe = Recipe {
    name: "Jelly Sandwich",
    ingreds: "Bread, Jelly",
    instrucs: "1. Spread any type of fruit jelly generously on the a slice of bread. \n2. Put another slice of bread on top of the slice with jelly. \n3. Enjoy!",  
};

// function to render
fn render_recipe(recipe: &Recipe) -> String{
    format!(
        r#"<h1>{}</h1>
        <h2>Ingredients</h2>
        <p>{}</p>
        <h2>Instructions</h2>
        <p>{}</p>"#,
        recipe.name,
        recipe.ingreds,
        recipe.instrucs
    )
}

// route handler
async fn hello() -> response::Html<String>{
    let recipe_html = render_recipe(&THE_RECIPE); // generate HTML string form recipe
    response::Html(format!( // wrap it in an HTML doc
        r#"<head><title>{}</title></head><body>{}</body>"#,
        THE_RECIPE.name,
        recipe_html
    ))
}

// sreve function
async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let app = axum::Router::new().route("/", routing::get(hello));
    let listener = net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[tokio::main]
async fn main(){
    if let Err(err) = serve().await {
        eprintln!("recipe-server: error: {}", err);
        std::process::exit(1);
    }
}