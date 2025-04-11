use std::sync::Arc;

use axum::{response::{Html, Response, IntoResponse}, routing::get, Router};
use tokio::sync::RwLock;

mod config;
mod search;

use search::handle_search;

async fn main_route_placeholder() -> Html<&'static str> {
    Html(include_str!("./index.html"))
}

async fn opensearch_placeholder() -> Response {
    let xml = include_str!("./search.xml");

    // Return the XML as a response
    let mut response = xml.into_response();
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("application/xml; charset=utf-8"),
    );
    response
}


#[tokio::main]
async fn main() {
    let instance = Arc::new(RwLock::new(config::build()));
    let app = Router::new()
        .route("/", get(main_route_placeholder))
        .route("/search", get(handle_search))
        .route("/search.xml", get(opensearch_placeholder))
        .with_state(Arc::clone(&instance));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
