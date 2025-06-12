mod error;
mod recipe;
mod templates;
mod web;
mod api;

use axum::{
    self,
    extract::{State},
    http::{self, StatusCode},
    response::{self, IntoResponse},
    routing,
};
use clap::Parser;
use sqlx::{SqlitePool, migrate::MigrateDatabase};
use tokio::{net, sync::RwLock};
use tower_http::{services, trace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::{path::PathBuf, sync::Arc};
use recipe::*;
use templates::*;
use askama::Template;
use recipe::fallback_recipe;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;
use utoipa_axum::router::OpenApiRouter;
use crate::error::RecipeError;
use utoipa::OpenApi;


extern crate log;

#[derive(Parser)]
struct Args {
    #[arg(short, long, name = "init-from")]
    init_from: Option<PathBuf>,
    #[arg(short, long, name = "db-uri")]
    db_uri: Option<String>,
}

struct AppState {
    db: SqlitePool,
    current_recipe: Recipe,
}

fn get_db_uri(db_uri: Option<&str>) -> String {
    if let Some(db_uri) = db_uri {
        db_uri.to_string()
    } else if let Ok(env_uri) = std::env::var("RECIPE_DB_URI") {
        env_uri
    } else {
        "sqlite://db/recipe.db".to_string()
    }
}

fn extract_db_dir(db_uri: &str) -> Result<&str, RecipeError> {
    if db_uri.starts_with("sqlite://") && db_uri.ends_with(".db") {
        let start = db_uri.find(':').unwrap() + 3;
        let mut path = &db_uri[start..];
        if let Some(end) = path.rfind('/') {
            path = &path[..end];
        } else {
            path = "";
        }
        Ok(path)
    } else {
        Err(RecipeError::InvalidDbUri(db_uri.to_string()))
    }
}

async fn seed_db_from_file(db: &SqlitePool, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let recipes = read_recipes(path)?;

    let mut tx = db.begin().await?;
    for r in &recipes {
        let ingredients_json = serde_json::to_string(&r.ingredients)?;
        let tags_json = r.tags.as_ref().map(|tags| serde_json::to_string(tags)).transpose()?;

        sqlx::query!(
            r#"INSERT INTO recipes (id, name, ingredients, instructions, tags, source) VALUES (?, ?, ?, ?, ?, ?)"#,
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

async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let db_uri = get_db_uri(args.db_uri.as_deref());

    if !sqlx::sqlite::Sqlite::database_exists(&db_uri).await? {
        let db_dir = extract_db_dir(&db_uri)?;
        std::fs::create_dir_all(db_dir)?;
        sqlx::sqlite::Sqlite::create_database(&db_uri).await?;
    }

    let db = SqlitePool::connect(&db_uri).await?;
    sqlx::migrate!().run(&db).await?;

    if let Some(path) = args.init_from {
        seed_db_from_file(&db, &path).await?;
        println!("Database seeded. Exiting.");
        std::process::exit(0);
    }

    let current_recipe = fallback_recipe();
    let app_state = AppState { db, current_recipe };
    let state = Arc::new(RwLock::new(app_state));

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "recipe-server=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let trace_layer = trace::TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO));

    let cors = tower_http::cors::CorsLayer::new()
        .allow_methods([http::Method::GET])
        .allow_origin(tower_http::cors::Any);

    async fn handler_404() -> axum::response::Response {
        (http::StatusCode::NOT_FOUND, "404 Not Found").into_response()
    }

    let mime_favicon = "image/vnd.microsoft.icon".parse().unwrap();

    let (api_router, api) = OpenApiRouter::with_openapi(api::ApiDoc::openapi())
        .nest("/api/v1", api::router())
        .split_for_parts();

    let swagger_ui = SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", api.clone());
    let redoc_ui = Redoc::with_url("/redoc", api.clone());
    let rapidoc_ui = RapiDoc::new("/api-docs/openapi.json").path("/rapidoc");

    let app = axum::Router::new()
        .route("/", routing::get(web::get_recipe))
        .route_service(
            "/recipe.css",
            services::ServeFile::new_with_mime("assets/static/recipe.css", &mime::TEXT_CSS_UTF_8),
        )
        .route_service(
            "/favicon.ico",
            services::ServeFile::new_with_mime("assets/static/favicon.ico", &mime_favicon),
        )
        .merge(swagger_ui)
        .merge(redoc_ui)
        .merge(rapidoc_ui)
        .merge(api_router)
        .fallback(handler_404)
        .layer(cors)
        .layer(trace_layer)
        .with_state(state);

    let listener = net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = serve().await {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}