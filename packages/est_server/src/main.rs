use std::sync::Arc;

use axum::{
    http::{header, HeaderMap, HeaderValue},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use tokio::sync::RwLock;

mod config;
mod search;
mod experimental;

use search::handle_search;

async fn main_route_placeholder() -> Html<&'static str> {
    Html(include_str!("./index.html"))
}

/// Serves the OpenSearch description document.
///
/// Chrome rejects OpenSearch descriptions whose search template is not an
/// absolute http(s) URL, so the origin is filled in at request time: from
/// `EST_PUBLIC_URL` if set, otherwise from the (forwarded) request headers.
async fn opensearch(headers: HeaderMap) -> Response {
    let origin = std::env::var("EST_PUBLIC_URL")
        .ok()
        .filter(|url| !url.is_empty())
        .unwrap_or_else(|| {
            let header = |name: &str| {
                headers
                    .get(name)
                    .and_then(|value| value.to_str().ok())
                    .and_then(|value| value.split(',').next())
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
            };
            let host = header("x-forwarded-host")
                .or_else(|| header("host"))
                .unwrap_or("localhost:3000");
            let proto = header("x-forwarded-proto").unwrap_or("http");
            format!("{proto}://{host}")
        });

    let xml = include_str!("./search.xml").replace("{origin}", origin.trim_end_matches('/'));

    (
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/opensearchdescription+xml; charset=utf-8"),
        )],
        xml,
    )
        .into_response()
}

pub struct AppState {
    instance: RwLock<est_core::Instance>,
}

#[tokio::main]
async fn main() {
    let instance = RwLock::new(config::build());
    let state = Arc::new(AppState {
        instance,
    });

    let app = Router::new()
        .route("/", get(main_route_placeholder))
        .route("/search", get(handle_search))
        .route("/search.xml", get(opensearch))
        .nest("/experimental", experimental::router())
        .with_state(state.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
