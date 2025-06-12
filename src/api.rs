use crate::*;
use axum::{
    extract::{Path, State},
    http::{self, StatusCode},
    response::{self, IntoResponse, Response},
    Json,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::routes;


/// OpenAPI doc container
#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "recipe-server", description = "Recipe REST API")
    )
)]
pub struct ApiDoc;

/// Utoipa-compatible router with documented routes
pub fn router() -> OpenApiRouter<Arc<RwLock<AppState>>> {
    OpenApiRouter::new()
        .routes(routes!(get_recipe))
        .routes(routes!(get_random_recipe))
}

/// API-compatible version of a recipe
#[derive(serde::Serialize, ToSchema)]
pub struct JsonRecipe {
    id: String,
    name: String,
    ingredients: Vec<String>,
    instructions: String,
    tags: Option<Vec<String>>,
    source: Option<String>,
}

impl From<Recipe> for JsonRecipe {
    fn from(r: Recipe) -> Self {
        JsonRecipe {
            id: r.id,
            name: r.name,
            ingredients: r.ingredients,
            instructions: r.instructions,
            tags: r.tags,
            source: r.source,
        }
    }
}

/// Helper to fetch a recipe by ID from DB and wrap in JsonRecipe
async fn get_recipe_by_id(db: &sqlx::SqlitePool, recipe_id: &str) -> Result<Response, StatusCode> {
    let recipe_result = recipe::get(db, recipe_id).await;
    match recipe_result {
        Ok((recipe, _tags)) => Ok(Json(JsonRecipe::from(recipe)).into_response()),
        Err(e) => {
            log::warn!("recipe fetch failed: {}", e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Get a recipe by ID
#[utoipa::path(
    get,
    path = "/recipe/{recipe_id}",
    responses(
        (status = 200, description = "Get a recipe by id", body = [JsonRecipe]),
        (status = 404, description = "No matching recipe found"),
    )
)]
pub async fn get_recipe(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(recipe_id): Path<String>,
) -> Result<Response, StatusCode> {
    let app_reader = app_state.read().await;
    let db = &app_reader.db;
    get_recipe_by_id(db, &recipe_id).await
}

/// Get a random recipe
#[utoipa::path(
    get,
    path = "/random-recipe",
    responses(
        (status = 200, description = "Get a random recipe", body = [JsonRecipe]),
        (status = 404, description = "No recipe available"),
    )
)]
pub async fn get_random_recipe(
    State(app_state): State<Arc<RwLock<AppState>>>,
) -> Result<Response, StatusCode> {
    let app_reader = app_state.read().await;
    let db = &app_reader.db;

    match recipe::get_random(db).await {
        Ok(recipe_id) => get_recipe_by_id(db, &recipe_id).await,
        Err(e) => {
            log::warn!("get random recipe failed: {}", e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}
