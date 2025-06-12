use crate::*;
use axum::{
    extract::{Query, State},
    http,
    response::{self, IntoResponse},
};
use serde::Deserialize;
use sqlx::SqlitePool;
use tokio::sync::RwLock;
use std::sync::Arc;

/// Parameters passed from the query string (e.g., `/?id=recipe_id`)
#[derive(Deserialize)]
pub struct GetRecipeParams {
    pub id: Option<String>,
    pub tags: Option<String>, // ignored for now
}

/// HTML handler: loads a recipe by ID or redirects to a random one
pub async fn get_recipe(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<GetRecipeParams>,
) -> Result<response::Response, http::StatusCode> {
    let mut app_writer = app_state.write().await;
    let db = app_writer.db.clone();

    // If an ID is provided, fetch and display that recipe
    if let Some(id) = params.id {
        let recipe_result = recipe::get(&db, &id).await;
        match recipe_result {
            Ok((recipe, tags)) => {
                let tag_string = tags.join(", ");
                app_writer.current_recipe = recipe.clone();

                let page = IndexTemplate::new(recipe.clone(), tag_string);
                Ok(response::Html(page.render().unwrap()).into_response())
            }
            Err(e) => {
                log::warn!("recipe fetch failed: {}", e);
                Err(http::StatusCode::NOT_FOUND)
            }
        }
    } else {
        // No ID? Redirect to a random recipe
        match recipe::get_random(&db).await {
            Ok(id) => {
                let uri = format!("/?id={}", id);
                Ok(response::Redirect::to(&uri).into_response())
            }
            Err(e) => {
                log::error!("random recipe selection failed: {}", e);
                panic!("random recipe selection failed: {}", e);
            }
        }
    }
}
