//! Built-in plugin for Rust search

use std::collections::HashMap;

use url::Url;

use crate::{
    common::{Fail, Result},
    engine::{ArenaEngine, Engine, ExecuteAction},
    query::Query,
    router::{Router, RouterMapLeaves, RouterTerminal},
};

pub struct PluginRust;

impl Default for PluginRust {
    fn default() -> Self {
        Self::new()
    }
}

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
            RouterTerminal(handle_rust_std_docs),
            RouterMapLeaves::new(HashMap::from([
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
    fn execute(&self, query: &Query) -> Result<ExecuteAction> {
        Ok(ExecuteAction::redirect_to_query(
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
    fn execute(&self, query: &Query) -> Result<ExecuteAction> {
        let url = match self {
            EngineCrates::Crates => URL_CRATES_IO_SEARCH,
            EngineCrates::Lib => URL_LIB_RS_SEARCH,
        };

        Ok(ExecuteAction::redirect_to_query(
            url,
            &[("q", query.content())],
        ))
    }
}

#[derive(Clone, Copy)]
struct EngineDocsRs;

static URL_DOCS_RS: &str = "https://docs.rs/";

impl Engine for EngineDocsRs {
    fn execute(&self, query: &Query) -> Result<ExecuteAction> {
        let url = Url::parse(URL_DOCS_RS).unwrap();
        let mut url = match query.residue() {
            [crate_name, version, ..] => url.join(crate_name).unwrap().join(version).unwrap(),
            [crate_name] => url.join(crate_name).unwrap(),
            _ => return Err(Fail::Incomplete),
        };

        url.query_pairs_mut().append_pair("search", query.content());

        Ok(ExecuteAction::Redirect(url.to_string()))
    }
}
