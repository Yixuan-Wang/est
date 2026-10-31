use std::sync::Arc;

use axum::{
    extract::State,
    http::{header, HeaderValue},
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

/// Fallback for `EST_PUBLIC_URL`, matching the address the server binds to.
const DEFAULT_PUBLIC_URL: &str = "http://localhost:3000";

/// Builds the OpenSearch description document from `EST_PUBLIC_URL`.
///
/// Chrome rejects OpenSearch descriptions whose search template is not an
/// absolute http(s) URL, and the server cannot reliably know the address it is
/// reached at (e.g. behind a reverse proxy), so the public URL is configured
/// explicitly. When unset, it defaults to [`DEFAULT_PUBLIC_URL`] for local use.
fn build_opensearch() -> String {
    let public_url = match std::env::var("EST_PUBLIC_URL") {
        Ok(url) if !url.trim().is_empty() => url,
        _ => {
            eprintln!(
                "warning: EST_PUBLIC_URL is not set, defaulting to {DEFAULT_PUBLIC_URL}; \
                 browsers on other machines cannot use Est as a search engine \
                 until it is set to the URL Est is served at"
            );
            DEFAULT_PUBLIC_URL.to_string()
        }
    };
    let origin = public_url.trim().trim_end_matches('/');

    let host = origin
        .strip_prefix("https://")
        .or_else(|| origin.strip_prefix("http://"));
    if host.is_none_or(str::is_empty) {
        panic!(
            "EST_PUBLIC_URL must be the absolute http(s) URL Est is served at, \
             e.g. EST_PUBLIC_URL=https://est.example.com (got {public_url:?})"
        );
    }

    include_str!("./search.xml").replace("{origin}", origin)
}

async fn opensearch(State(state): State<Arc<AppState>>) -> Response {
    (
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/opensearchdescription+xml; charset=utf-8"),
        )],
        state.opensearch.clone(),
    )
        .into_response()
}

pub struct AppState {
    instance: RwLock<est_core::Instance>,
    opensearch: String,
}

#[tokio::main]
async fn main() {
    let instance = RwLock::new(config::build());
    let state = Arc::new(AppState {
        instance,
        opensearch: build_opensearch(),
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
