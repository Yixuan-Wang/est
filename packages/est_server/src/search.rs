use std::sync::Arc;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;

use crate::AppState;

#[derive(Deserialize)]
pub struct SearchUrlQuery {
    q: String,
}

pub async fn handle_search(
    State(state): State<Arc<AppState>>,
    Query(url_query): Query<SearchUrlQuery>,
) -> Result<Response, (StatusCode, String)> {
    let url_query = url_query.q;
    let query = url_query
        .parse::<est_core::Query>()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid query".to_string()))?;
    // instance.read().await.react()

    use est_core::{ReactionErr, ReactionVerb};
    let reaction = state.instance
        .read()
        .await
        .react(query)
        .await
        .map_err(|err| match err {
            ReactionErr::Panic(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal server error: {}", err),
            ),
            ReactionErr::Nothing => (
                StatusCode::NOT_FOUND,
                format!("No reaction found for query: {}", err),
            ),
            _ => (
                StatusCode::BAD_REQUEST,
                format!("Bad request because of error: {}", err),
            ),
        })?;

    let response = match reaction {
        ReactionVerb::Navigate(nav) => Redirect::to(nav.url().as_str()).into_response(),
        _ => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unsupported reaction returned by the engine".to_string(),
            ))
        }
    };

    Ok(response)
}
