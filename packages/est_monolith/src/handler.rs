use crate::AppState;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(serde::Deserialize)]
pub(crate) struct SearchQuery {
    pub q: String,
}

/// Route for searching
pub(crate) async fn search(
    State(state): State<Arc<RwLock<AppState>>>,
    Query(query): Query<SearchQuery>,
) -> Response {
    let mut query = crate::query::Query::try_from(query.q.as_ref()).unwrap();

    let route = match state.read().await.router.route(&mut query) {
        Ok(Some(route)) => route,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                format!("Incomplete query: @{}", query.mention.join(".")),
            )
                .into_response()
        }
        Ok(None) => {
            return (
                axum::http::StatusCode::NOT_FOUND,
                format!(
                    "No plugin found for this instruction: @{}",
                    query.mention.join(".")
                ),
            )
                .into_response()
        }
    };

    let handle_engine = route.engine;

    let Some(engine) = state.read().await.engine.get(handle_engine) else {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Engine not found".to_string(),
        )
            .into_response();
    };

    let Ok(result) = engine.read().await.execute(&query) else {
        return (
            axum::http::StatusCode::NOT_FOUND,
            format!("Unable to search this term: {}", query.content()),
        )
            .into_response();
    };

    match result {
        crate::engine::ExecuteAction::Redirect(url) => {
            axum::response::Redirect::to(&url).into_response()
        }
    }
}

/// Route for using as an [OpenSearch](https://developer.mozilla.org/en-US/docs/Web/OpenSearch) plugin
pub(crate) async fn open_search_plugin() -> Response {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<OpenSearchDescription xmlns="http://a9.com/-/spec/opensearch/1.1/">
  <ShortName>Est</ShortName>
  <Description>Yixuan's Extensible Search Tool</Description>
  <InputEncoding>UTF-8</InputEncoding>
  <Url type="text/html" template="https://est.tomyxw.me/search?q={searchTerms}"/>
  <Url type="application/x-suggestions+json" template="https://ac.duckduckgo.com/ac/?q={searchTerms}&amp;type=list"/>
</OpenSearchDescription>"#;

    // Return the XML as a response
    let mut response = xml.into_response();
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("application/xml; charset=utf-8"),
    );
    response
}
