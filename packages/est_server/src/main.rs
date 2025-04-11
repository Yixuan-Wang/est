use std::sync::Arc;

use axum::{response::Html, routing::get, Router};
use tokio::sync::RwLock;

mod config;
mod search;

use search::handle_search;

async fn main_route_placeholder() -> Html<&'static str> {
    Html(include_str!("./index.html"))
}

#[tokio::main]
async fn main() {
    let instance = Arc::new(RwLock::new(config::build()));
    let app = Router::new()
        .route("/", get(main_route_placeholder))
        .route("/search", get(handle_search))
        .with_state(Arc::clone(&instance));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
