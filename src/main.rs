mod error;
mod recipe;
mod templates;

use axum::{self, extract::State, response, routing};
use clap::Parser;
use sqlx::SqlitePool;
use tokio::{net, sync::RwLock};
use tower_http::{services, trace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use crate::error::RecipeError;
use std::{path::PathBuf, sync::Arc};
use recipe::*;
use templates::*;
use askama::Template;


/// Command-line arguments
#[derive(Parser)]
struct Args {
    /// Optional path to JSON file to seed the database
    #[arg(short, long, name = "init-from")]
    init_from: Option<PathBuf>,
}

/// Shared application state (just the database for now)
struct AppState {
    db: SqlitePool,
}

/// Route handler: fetch a random recipe and render it as HTML
async fn get_recipe(State(app_state): State<Arc<RwLock<AppState>>>) -> response::Html<String> {
    let app_state = app_state.read().await;
    let db = &app_state.db;

    let row = sqlx::query!(
        r#"
        SELECT id, name, ingredients, instructions, tags, source 
        FROM recipes 
        ORDER BY RANDOM() LIMIT 1
        "#
    )
    .fetch_one(db)
    .await
    .expect("Failed to fetch recipe");

    let recipe = Recipe {
        id: row.id.expect("Missing id"),
        name: row.name,
        ingredients: serde_json::from_str(&row.ingredients).unwrap(),
        instructions: row.instructions,
        tags: row.tags.map(|t: String| serde_json::from_str(&t).unwrap()),
        source: row.source,
    };

    let template = IndexTemplate::new(&recipe);
    response::Html(template.render().unwrap())
}

/// Seeds the database from a local JSON file
async fn seed_db_from_file(db: &SqlitePool, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let recipes = read_recipes(path)?;

    let mut tx = db.begin().await?;
    for r in &recipes {
        let ingredients_json = serde_json::to_string(&r.ingredients)?;
        let tags_json = r.tags.as_ref().map(|tags| serde_json::to_string(tags)).transpose()?;

        sqlx::query!(
            r#"
            INSERT INTO recipes (id, name, ingredients, instructions, tags, source)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            r.id,
            r.name,
            ingredients_json,
            r.instructions,
            tags_json,
            r.source,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    println!("Seeded {} recipes from {:?}", recipes.len(), path);
    Ok(())
}

/// Starts the Axum web server
async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Set up the DB and run any pending migrations
    let db = SqlitePool::connect("sqlite://db/recipe.db").await?;
    sqlx::migrate!().run(&db).await?;

    // Optionally seed the database and exit
    if let Some(path) = args.init_from {
        seed_db_from_file(&db, &path).await?;
        println!("Database seeded. Exiting.");
        std::process::exit(0);
    }

    // Shared app state
    let state = Arc::new(RwLock::new(AppState { db }));

    // Set up logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "recipe-server=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Enable tracing layer for incoming HTTP traffic
    let trace_layer = trace::TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO));

    let mime_favicon = "image/vnd.microsoft.icon".parse().unwrap();

    // Router definition
    let app = axum::Router::new()
        .route("/", routing::get(get_recipe))
        .route_service(
            "/recipe.css",
            services::ServeFile::new_with_mime("assets/static/recipe.css", &mime::TEXT_CSS_UTF_8),
        )
        .route_service(
            "/favicon.ico",
            services::ServeFile::new_with_mime("assets/static/favicon.ico", &mime_favicon),
        )
        .layer(trace_layer)
        .with_state(state);

    // Bind and run the server
    let listener = net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Main entry point
#[tokio::main]
async fn main() {
    if let Err(err) = serve().await {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}
