use std::sync::Arc;

use axum::{
    routing::get,
    Router,
};

use tokio::sync::RwLock;

pub mod common;
pub mod engine;
pub mod query;
pub mod router;
pub mod plugin;

mod handler {
    use std::sync::Arc;
    use axum::extract::{Query, State};
    use axum::response::{IntoResponse, Response};
    use tokio::sync::RwLock;
    use crate::AppState;

    #[derive(serde::Deserialize)]
    pub struct SearchQuery {
        q: String,
    }

    pub async fn search(
        State(state): State<Arc<RwLock<AppState>>>,
        Query(query): Query<SearchQuery>
    ) -> Response {
        let mut query = crate::query::Query::try_from(query.q.as_ref()).unwrap();

        let handle_engine = match state.read().await.router.route(&mut query) {
            Ok(Some(handle_engine)) => handle_engine,
            Err(_) => return (
                axum::http::StatusCode::BAD_REQUEST,
                format!("Incomplete query: @{}", query.mention().join("."))
            ).into_response(),
            Ok(None) => return (
                axum::http::StatusCode::NOT_FOUND,
                format!("No plugin found for this instruction: @{}", query.mention().join("."))
            ).into_response(),
        };

        let Some(engine) = state.read().await.engine.get(handle_engine) else {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Engine not found".to_string()
            ).into_response();
        };
        
        let Ok(result) = engine.read().await.search(&query) else {
            return (
                axum::http::StatusCode::NOT_FOUND,
                format!("Unable to search this term: {}", query.content())
            ).into_response();
        };

        match result {
            crate::engine::SearchAction::Redirect(url) => axum::response::Redirect::to(&url).into_response()
        }
    }

    pub async fn open_search_plugin() -> Response {
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
}

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
    let router_python: Arc<dyn router::Router + Send + Sync> = Arc::new(plugin_python.router(&mut arena));
    let router_javascript: Arc<dyn router::Router + Send + Sync> = Arc::new(plugin_javascript.router(&mut arena));
    let router_rust: Arc<dyn router::Router + Send + Sync> = Arc::new(plugin_rust.router(&mut arena));
    let (router_search, router_search_fallback) = plugin_search.router(&mut arena);
    let router_search_fallback: Arc<dyn router::Router + Send + Sync> = Arc::new(router_search_fallback);

    let router = (
        router::Terminal(Arc::clone(&router_search_fallback)),
        router_search,
        router::RouterMapLayer::new(std::collections::HashMap::from([
            (String::from("py"), router_python),
            (String::from("js"), router_javascript),
            (String::from("rs"), router_rust),
        ])),
        Arc::clone(&router_search_fallback),
    );

    let app_state = Arc::new(RwLock::new(AppState {
        engine: arena,
        router: Box::new(router),
    }));

    // build our application with a single route
    let app = Router::new()
        .route("/", get( || async {
            axum::response::Html(include_str!("./index.html"))
        }))
        .route("/search", get(handler::search))
        .route("/search.xml", get(handler::open_search_plugin))
        .with_state(Arc::clone(&app_state));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
