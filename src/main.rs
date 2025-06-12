mod error;
mod recipe;
mod templates;

use axum::{self, extract::State, response, routing};
use clap::Parser;
use sqlx::{SqlitePool, migrate::MigrateDatabase};
use tokio::{net, sync::RwLock};
use tower_http::{services, trace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use crate::error::RecipeError;
use std::{path::PathBuf, sync::Arc};
use recipe::*;
use templates::*;
use askama::Template;
use recipe::fallback_recipe;
use axum::http::StatusCode;


extern crate log;


/// Command-line arguments
#[derive(Parser)]
struct Args {
    /// Optional path to JSON file to seed the database
    #[arg(short, long, name = "init-from")]
    init_from: Option<PathBuf>,
    #[arg(short, long, name = "db-uri")]
    db_uri: Option<String>,
}

/// Shared application state (just the database for now)
struct AppState {
    db: SqlitePool,
    current_recipe: Recipe, // Holds fallback recipe if DB query fails
}
/// Query parameters for selecting a recipe by ID
#[derive(serde::Deserialize)]
struct GetRecipeParams {
    id: Option<String>,
}

/// Determine the database URI from the following sources:
fn get_db_uri(db_uri: Option<&str>) -> String {
    if let Some(db_uri) = db_uri {
        db_uri.to_string()
    } else if let Ok(env_uri) = std::env::var("RECIPE_DB_URI") {
        env_uri
    } else {
        "sqlite://db/recipe.db".to_string()
    }
}

/// Extracts the directory path from a SQLite URI for folder creation.
/// Returns an error if the URI isn't a valid SQLite file URI.
fn extract_db_dir(db_uri: &str) -> Result<&str, RecipeError> {
    if db_uri.starts_with("sqlite://") && db_uri.ends_with(".db") {
        let start = db_uri.find(':').unwrap() + 3; // Skip past "sqlite://"
        let mut path = &db_uri[start..];
        if let Some(end) = path.rfind('/') {
            path = &path[..end]; // Get folder path only
        } else {
            path = ""; // DB file in root
        }
        Ok(path)
    } else {
        Err(RecipeError::InvalidDbUri(db_uri.to_string()))
    }
}

/// Select a recipe by ID if provided, or choose a random recipe otherwise
async fn choose_recipe(db: &SqlitePool, params: &GetRecipeParams) -> Result<Recipe, sqlx::Error> {
    if let Some(id) = &params.id {
        let row = sqlx::query!(
            r#"
            SELECT id, name, ingredients, instructions, tags, source
            FROM recipes WHERE id = ?
            "#,
            id
        )
        .fetch_one(db)
        .await?;

        Ok(Recipe {
            id: row.id.expect("Missing id"),
            name: row.name,
            ingredients: serde_json::from_str(&row.ingredients).unwrap(),
            instructions: row.instructions,
            tags: row.tags.map(|t| serde_json::from_str(&t).unwrap()),
            source: row.source,
        })
    } else {
        let row = sqlx::query!(
            r#"
            SELECT id, name, ingredients, instructions, tags, source
            FROM recipes ORDER BY RANDOM() LIMIT 1
            "#
        )
        .fetch_one(db)
        .await?;

        Ok(Recipe {
            id: row.id.expect("Missing id"),
            name: row.name,
            ingredients: serde_json::from_str(&row.ingredients).unwrap(),
            instructions: row.instructions,
            tags: row.tags.map(|t| serde_json::from_str(&t).unwrap()),
            source: row.source,
        })
    }
}

/// Route handler: fetch a random recipe and render it as HTML
async fn get_recipe(
    State(app_state): State<Arc<RwLock<AppState>>>,
    axum::extract::Query(params): axum::extract::Query<GetRecipeParams>,
)-> Result<response::Html<String>, StatusCode> {
    let mut app_state = app_state.write().await;
    let db = app_state.db.clone();

    let recipe_result = choose_recipe(&db, &params).await;

    match recipe_result {
        Ok(row) => {
            let recipe = Recipe {
                id: row.id,
                name: row.name,
                ingredients: row.ingredients,
                instructions: row.instructions,
                tags: row.tags,
                source: row.source,
            };

            // Extract tag list if available
            let tag_list = recipe
                .tags
                .clone()
                .unwrap_or_default()
                .into_iter()
                .collect::<Vec<String>>();

            let tag_string = tag_list.join(", ");

            app_state.current_recipe = recipe.clone();
            let template = IndexTemplate::new(recipe.clone(), tag_string);
            Ok(response::Html(template.render().unwrap()))
        }
        Err(e) => {
            log::warn!("Recipe fetch failed: {}", e);
            Err(StatusCode::NOT_FOUND)
        }
    }

    
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

    // Get database URI (from CLI, env var, or fallback)
    let db_uri = get_db_uri(args.db_uri.as_deref());

    // If the SQLite file doesn't exist, create the necessary folder and file
    if !sqlx::sqlite::Sqlite::database_exists(&db_uri).await? {
        let db_dir = extract_db_dir(&db_uri)?;
        std::fs::create_dir_all(db_dir)?; // Ensure folder exists
        sqlx::sqlite::Sqlite::create_database(&db_uri).await?; // Create the DB file
    }

    // Now connect to the SQLite database
    let db = SqlitePool::connect(&db_uri).await?;

    sqlx::migrate!().run(&db).await?;

    // Optionally seed the database and exit
    if let Some(path) = args.init_from {
        seed_db_from_file(&db, &path).await?;
        println!("Database seeded. Exiting.");
        std::process::exit(0);
    }

    // Shared app state
    let current_recipe = fallback_recipe();
    let app_state = AppState { db, current_recipe };
    let state = Arc::new(RwLock::new(app_state));

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
