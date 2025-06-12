use crate::*;
use axum::{extract::{Query, State}, http, response, response::IntoResponse};
use serde::Deserialize;
use sqlx::SqlitePool;
use tokio::sync::RwLock;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct GetRecipeParams {
    pub id: Option<String>,
    pub tags: Option<String>,
}

/// HTML handler for displaying a recipe via ID or tag filtering
pub async fn get_recipe(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<GetRecipeParams>,
) -> Result<response::Response, http::StatusCode> {
    let mut app_state = app_state.write().await;
    let db = app_state.db.clone();

    // If specific ID is provided, render that recipe
    if let Some(id) = params.id {
        let recipe_result = recipe::get(&db, &id).await;
        match recipe_result {
            Ok((recipe, tags)) => {
                let tag_string = tags.join(", ");
                app_state.current_recipe = recipe.clone();
                let template = IndexTemplate::new(recipe.clone(), tag_string);
                Ok(response::Html(template.render().unwrap()).into_response())
            }
            Err(e) => {
                log::warn!("recipe fetch failed: {}", e);
                Err(http::StatusCode::NOT_FOUND)
            }
        }
    } else {
        // If no ID, redirect to a random recipe
        let recipe_result = sqlx::query_scalar!("SELECT id FROM recipes ORDER BY RANDOM() LIMIT 1;")
            .fetch_one(&db)
            .await;

        match recipe_result {
            Ok(id) => {
                let uri = format!("/?id={}", id.clone().unwrap_or_default());
                Ok(response::Redirect::to(&uri).into_response())
            }
            Err(e) => {
                log::error!("random recipe selection failed: {}", e);
                Err(http::StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}