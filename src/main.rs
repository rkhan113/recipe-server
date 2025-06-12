// Bring in required crates
// use axum::{self, response, routing};
// use tokio::net;
use axum::{self, extract::State, response, routing};
extern crate fastrand;
use tokio::{net, sync::RwLock};
use tower_http::{services, trace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use askama::Template; // Import Askama's Template trait to enable render() method (was getting a warning before)

// Bring in our local modules
mod error;
mod recipe;
mod templates;

use error::*;
use recipe::*;
use templates::*;

use std::sync::Arc;

struct AppState {
    recipes: Vec<Recipe>,
}

// GET /random
async fn get_recipe(State(app_state): State<Arc<RwLock<AppState>>>) -> response::Html<String> {
    let app_state = app_state.read().await;
    let nrecipes = app_state.recipes.len();
    let i = fastrand::usize(0..nrecipes);
    let recipe = &app_state.recipes[i];

    let template = IndexTemplate::new(recipe);
    response::Html(template.render().unwrap())
}


// Main server setup
async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let recipes = read_recipes("assets/static/recipes.json")?;
    let state = Arc::new(RwLock::new(AppState{recipes}));
    
    // Initialize structured logging and HTTP tracing for Axum with environment-based filtering.
     tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "kk2=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    // https://carlosmv.hashnode.dev/adding-logging-and-tracing-to-an-axum-app-rust
    let trace_layer = trace::TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO));
    
    // Define MIME type for favicon (.ico file)
    let mime_favicon = "image/vnd.microsoft.icon".parse().unwrap();
    
    // Create the router
    let app = axum::Router::new()
        .route("/", routing::get(get_recipe)) // Route for the index page
        // Serve static CSS file (must match file path & MIME)
        .route_service(
            "/recipe.css",
            services::ServeFile::new_with_mime(
                "assets/static/recipe.css",
                &mime::TEXT_CSS_UTF_8,
            ),
        )
        // Serve favicon (browser requests this at /favicon.ico)        
        .route_service(
            "/favicon.ico",
            services::ServeFile::new_with_mime(
                "assets/static/favicon.ico",
                &mime_favicon,
            ),
        )
    .layer(trace_layer)
    .with_state(state);

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
