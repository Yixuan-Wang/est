use std::sync::Arc;

use axum::{extract::{Path, State}, routing::get, Json};
use serde_json::{json, Value};

use crate::AppState;


async fn list_engines(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let engines: Vec<String> = state.instance.read().await.iter_engine_ids().map(Clone::clone).collect();
    Json(json!({
        "engines": engines,
    }))
}

async fn description(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    let description = state.instance.read().await.describe(&id);
    Json(json!({
        "id": id,
        "description": description,
    }))
}

pub fn router() -> axum::Router<Arc<AppState>> {
    axum::Router::new()
        .route("/engines", get(list_engines))
        .route("/description/{id}", get(description))
}
