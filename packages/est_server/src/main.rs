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

mod main {
    pub(super) fn prepare_config() -> Option<toml::Value> {
        let config = std::fs::read_to_string("config.toml").unwrap_or_else(|err| {
            eprintln!("Cannot find `config.toml`: {}, using default config.", err);
            include_str!("asset/config.default.toml").to_string()
        });
        toml::from_str(&config)
            .map_err(|e| {
                eprintln!("Failed to parse config file: {}", e);
            })
            .ok()
    }
}

#[tokio::main]
async fn main() {
    let Some(config) = main::prepare_config() else {
        eprintln!("Failed to load config file.");
        return;
    };

    let plugin_python = plugin::python::PluginPython::new();
    let plugin_javascript = plugin::javascript::PluginJavascript;
    let plugin_search = plugin::search::PluginSearch::new();
    let plugin_rust = plugin::rust::PluginRust::new();

    let plugin_simple = plugin::simple::PluginSimple::from_config(&config);

    let mut arena = engine::ArenaEngine::new();

    type DynRouter = Arc<dyn router::Router + Send + Sync>;

    let router_python: DynRouter = Arc::new(plugin_python.router(&mut arena));
    let router_javascript: DynRouter = Arc::new(plugin_javascript.router(&mut arena));
    let router_rust: DynRouter = Arc::new(plugin_rust.router(&mut arena));
    let (router_search, router_search_fallback) = plugin_search.router(&mut arena);
    let router_simple: DynRouter = Arc::new(plugin_simple.router(&mut arena));
    let router_search_fallback: DynRouter = Arc::new(router_search_fallback);

    // FIXME: The main router is growing too large.
    //        All hashmap should be ideally merged together.
    let router = (
        router::RouterTerminal(Arc::clone(&router_search_fallback)),
        router_search,
        router_simple,
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
