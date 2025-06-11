// Bring in required crates
use axum::{self, response, routing};
use tokio::net;
use tower_http::services;
use askama::Template; // Import Askama's Template trait to enable render() method (was getting a warning before)

// Bring in our local modules
mod recipe;
mod templates;

use recipe::*;
use templates::*;

// Route handler for the index page
async fn get_recipe() -> response::Html<String> {
    // Create the template with the example recipe
    let template = IndexTemplate::new(&THE_RECIPE);
    // Render it to a String and wrap it in Html response
    response::Html(template.render().unwrap())
}

// Main server setup
async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    // Create the router
    let app = axum::Router::new()
        .route("/", routing::get(get_recipe)) // Route for the index page
        // Serve static CSS file from disk
        .route_service(
            "/recipe.css",
            services::ServeFile::new_with_mime("assets/static/recipe.css", &mime::TEXT_CSS),
        );

    // Bind to localhost on port 3000
    let listener = net::TcpListener::bind("127.0.0.1:3000").await?;
    // Start the server
    axum::serve(listener, app).await?;
    Ok(())
}

// Entry point of the app
#[tokio::main]
async fn main() {
    // If serve() returns an error, log and exit
    if let Err(err) = serve().await {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}
