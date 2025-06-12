// Bring in required crates and local modules
use axum::{self, extract::State, response, routing}; // Axum web framework
use clap::Parser; // CLI parsing
use sqlx::SqlitePool; // Database connection pool
use tokio::{net, sync::RwLock}; // Async runtime and synchronization
use tower_http::{services, trace}; // HTTP tracing and static file serving
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt}; // Logging
use askama::Template; // Template rendering for HTML

mod error;
mod recipe;
mod templates;

use error::*;
use recipe::*;
use templates::*;

use std::sync::Arc;
use std::path::PathBuf;

/// CLI arguments
#[derive(Parser)]
struct Args {
    /// Optional JSON file to initialize the database from
    #[arg(short, long, name = "init-from")]
    init_from: Option<PathBuf>,
}

/// Application shared state (holds the database pool)
struct AppState {
    db: SqlitePool,
}

/// HTTP GET handler for `/`
/// Returns a random recipe rendered as HTML.
async fn get_recipe(State(app_state): State<Arc<RwLock<AppState>>>) -> response::Html<String> {
    let app_state = app_state.read().await;
    let db = &app_state.db;

    // NOTE: Manually fetch row and convert fields for `Vec<String>`
    let row = sqlx::query!(
        r#"
        SELECT id, name, ingredients, instructions, tags, source 
        FROM recipes 
        ORDER BY RANDOM() LIMIT 1;
        "#
    )
    .fetch_one(db)
    .await
    .expect("Failed to fetch recipe");

    // NOTE: Parse JSON fields into Vec<String> for ingredients and tags
    let recipe = Recipe {
        id: row.id.expect("Expected non-null id"),
        name: row.name,
        ingredients: serde_json::from_str(&row.ingredients).unwrap(),
        instructions: row.instructions,
        tags: row
            .tags
            .map(|t| serde_json::from_str(&t).unwrap()),
        source: row.source,
    };

    // Render HTML template
    let template = templates::IndexTemplate::new(&recipe);
    response::Html(template.render().unwrap())
}


/// Reads the JSON file and seeds the database with recipes.
/// Inserts each recipe, serializing ingredients and tags as JSON strings.
async fn seed_db_from_file(db: &SqlitePool, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let recipes = read_recipes(path)?;

    let mut tx = db.begin().await?;
    for r in &recipes {
        let ingredients_json = serde_json::to_string(&r.ingredients)?;
        let tags_json = r
            .tags
            .as_ref()
            .map(|tags| serde_json::to_string(tags))
            .transpose()?; // tags_json: Option<String>

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

/// Main server setup and run
async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    // Parse CLI arguments
    let args = Args::parse();

    // Connect to the SQLite database and run migrations
    let db = SqlitePool::connect("sqlite://db/recipe.db").await?;
    sqlx::migrate!().run(&db).await?;

    // If `--init-from` provided, seed the database and exit immediately
    if let Some(path) = args.init_from {
        seed_db_from_file(&db, &path).await?;
        println!("Database seeded successfully. Exiting.");
        std::process::exit(0);
    }

    // Create shared application state
    let state = Arc::new(RwLock::new(AppState { db }));

    // Initialize structured logging and HTTP tracing for Axum with environment-based filtering.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "recipe-server=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Configure tracing layer to log request/response info
    let trace_layer = trace::TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO));

    // MIME type for favicon.ico
    let mime_favicon = "image/vnd.microsoft.icon".parse().unwrap();

    // Build Axum router with routes and middleware
    let app = axum::Router::new()
        // Route to get a random recipe
        .route("/", routing::get(get_recipe))
        // Serve CSS static file with correct MIME type
        .route_service(
            "/recipe.css",
            services::ServeFile::new_with_mime(
                "assets/static/recipe.css",
                &mime::TEXT_CSS_UTF_8,
            ),
        )
        // Serve favicon.ico static file
        .route_service(
            "/favicon.ico",
            services::ServeFile::new_with_mime("assets/static/favicon.ico", &mime_favicon),
        )
        // Add HTTP tracing middleware
        .layer(trace_layer)
        // Attach shared app state
        .with_state(state);

    // Bind server to localhost:3000
    let listener = net::TcpListener::bind("127.0.0.1:3000").await?;

    // Run the server
    axum::serve(listener, app).await?;

    Ok(())
}

/// Program entry point
#[tokio::main]
async fn main() {
    // Run the server, exit with error message if it fails
    if let Err(err) = serve().await {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}
