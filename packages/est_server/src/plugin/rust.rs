//! Built-in plugin for Rust search

use std::collections::HashMap;

use url::Url;

use crate::{
    engine::{ArenaEngine, Engine, SearchAction, SearchError, SearchResult},
    query::Query,
    router::{Router, RouterDebug, RouterMapLeaf, Terminal},
};

pub struct PluginRust;

impl PluginRust {
    pub fn new() -> Self {
        PluginRust
    }

    pub fn router(&self, arena: &mut ArenaEngine) -> impl Router {
        let (handle_rust_std_docs, handle_crates_io, handle_lib_rs, handle_docs_rs) = (
            arena.insert(EngineRustStdDocs),
            arena.insert(EngineCrates::Crates),
            arena.insert(EngineCrates::Lib),
            arena.insert(EngineDocsRs),
        );

        (
            Terminal(handle_rust_std_docs),
            RouterMapLeaf::new(HashMap::from([
                (String::from("crates"), handle_crates_io),
                (String::from("lib"), handle_lib_rs),
                (String::from("docs"), handle_docs_rs),
                (String::from("std"), handle_rust_std_docs),
            ])),
            handle_docs_rs,
        )
    }
}

#[derive(Clone, Copy)]
struct EngineRustStdDocs;
static URL_RUST_STD_DOCS: &str = "https://doc.rust-lang.org/std/index.html";

impl Engine for EngineRustStdDocs {
    fn search(&self, query: &Query) -> SearchResult {
        Ok(SearchAction::redirect_to_query(
            URL_RUST_STD_DOCS,
            &[("search", query.content())],
        ))
    }
}

#[derive(Clone, Copy)]
enum EngineCrates {
    Crates,
    Lib,
}

static URL_CRATES_IO_SEARCH: &str = "https://crates.io/search";
static URL_LIB_RS_SEARCH: &str = "https://lib.rs/search";

impl Engine for EngineCrates {
    fn search(&self, query: &Query) -> SearchResult {
        let url = match self {
            EngineCrates::Crates => URL_CRATES_IO_SEARCH,
            EngineCrates::Lib => URL_LIB_RS_SEARCH,
        };

        Ok(SearchAction::redirect_to_query(
            url,
            &[("q", query.content())],
        ))
    }
}

#[derive(Clone, Copy)]
struct EngineDocsRs;

static URL_DOCS_RS: &str = "https://docs.rs/";

impl Engine for EngineDocsRs {
    fn search(&self, query: &Query) -> SearchResult {
        let url = Url::parse(URL_DOCS_RS).unwrap();
        dbg!(query.mention().residue());
        let mut url = match query.mention().residue() {
            [crate_name, version, ..] => url.join(crate_name).unwrap().join(version).unwrap(),
            [crate_name] => url.join(crate_name).unwrap(),
            _ => return Err(SearchError::Incomplete),
        };

        dbg!(&url);

        url.query_pairs_mut().append_pair("search", query.content());

        Ok(SearchAction::Redirect(url.to_string()))
    }
}
