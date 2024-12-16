use std::sync::Arc;

use axum::{routing::get, Router};

use tokio::sync::RwLock;

pub mod common;
pub mod engine;
mod handler;
pub mod plugin;
pub mod query;
pub mod router;

struct AppState {
    pub engine: engine::ArenaEngine,
    router: Box<dyn router::Router + Send + Sync>,
}

#[tokio::main]
async fn main() {
    // let plugins = plugin::GroupedPlugins {
    //     plugins: vec![
    //         Box::new(plugin::python::PluginPython::new()),
    //         Box::new(plugin::javascript::PluginJavascript),
    //         Box::new(plugin::search::PluginSearch::new())
    //     ],
    // };
    let plugin_python = plugin::python::PluginPython::new();
    let plugin_javascript = plugin::javascript::PluginJavascript;
    let plugin_search = plugin::search::PluginSearch::new();
    let plugin_rust = plugin::rust::PluginRust::new();

    let mut arena = engine::ArenaEngine::new();
    let router_python: Arc<dyn router::Router + Send + Sync> =
        Arc::new(plugin_python.router(&mut arena));
    let router_javascript: Arc<dyn router::Router + Send + Sync> =
        Arc::new(plugin_javascript.router(&mut arena));
    let router_rust: Arc<dyn router::Router + Send + Sync> =
        Arc::new(plugin_rust.router(&mut arena));
    let (router_search, router_search_fallback) = plugin_search.router(&mut arena);
    let router_search_fallback: Arc<dyn router::Router + Send + Sync> =
        Arc::new(router_search_fallback);

    let router = (
        router::RouterTerminal(Arc::clone(&router_search_fallback)),
        router_search,
        std::collections::HashMap::from([
            (String::from("py"), router_python),
            (String::from("js"), router_javascript),
            (String::from("rs"), router_rust),
        ]),
        Arc::clone(&router_search_fallback),
    );

    let app_state = Arc::new(RwLock::new(AppState {
        engine: arena,
        router: Box::new(router),
    }));

    // build our application with a single route
    let app = Router::new()
        .route(
            "/",
            get(|| async { axum::response::Html(include_str!("./index.html")) }),
        )
        .route("/search", get(handler::search))
        .route("/search.xml", get(handler::open_search_plugin))
        .with_state(Arc::clone(&app_state));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
