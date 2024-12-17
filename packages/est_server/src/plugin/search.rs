use std::collections::HashMap;

use crate::{
    common::Result,
    engine::{ArenaEngine, Engine, ExecuteAction},
    query::Query,
    router::{self, Router, RouterFn, RouterMapLeaves},
};

trait CharExt {
    fn is_cjk(&self) -> bool;
}

impl CharExt for char {
    fn is_cjk(&self) -> bool {
        matches!(
            *self,
            '\u{4E00}'..='\u{9FFF}' |  // CJK Unified Ideographs
            '\u{3400}'..='\u{4DBF}' |  // CJK Unified Ideographs Extension A
            '\u{20000}'..='\u{2A6DF}'    // CJK Unified Ideographs Extension B
        )
    }
}

pub struct PluginSearch;

impl Default for PluginSearch {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginSearch {
    pub fn new() -> Self {
        PluginSearch
    }

    // pub fn router_fallback<'r, 'q>(
    //     _route: &'r [&'q str],
    //     query: &'q Query,
    // ) -> router::Result<(Option<HandleEngine>, &'r [&'q str])> {

    // }

    pub fn router(&self, arena: &mut ArenaEngine) -> (RouterMapLeaves, impl Router) {
        let engine_google = arena.insert(EngineGoogle);
        let engine_bing = arena.insert(EngineBing { is_china: false });
        let engine_bing_china = arena.insert(EngineBing { is_china: true });
        let engine_ddg = arena.insert(EngineDuckDuckGo {
            is_bang: false,
            is_direct: false,
        });
        let engine_ddg_bang = arena.insert(EngineDuckDuckGo {
            is_bang: true,
            is_direct: false,
        });
        let engine_ddg_direct = arena.insert(EngineDuckDuckGo {
            is_bang: false,
            is_direct: true,
        });
        let engine_wikipedia = arena.insert(EngineWikipedia);
        let engine_baidu = arena.insert(EngineChina::Baidu);
        let engine_sogou = arena.insert(EngineChina::Sogou);

        let map = RouterMapLeaves::new(HashMap::from([
            (String::from("g"), engine_google),
            (String::from("google"), engine_google),
            (String::from("b"), engine_bing),
            (String::from("bing"), engine_bing),
            (String::from("bd"), engine_baidu),
            (String::from("baidu"), engine_baidu),
            (String::from("sg"), engine_sogou),
            (String::from("sogou"), engine_sogou),
            (String::from("d"), engine_ddg),
            (String::from("ddg"), engine_ddg),
            (String::from("w"), engine_wikipedia),
            (String::from("wiki"), engine_wikipedia),
        ]));

        let fallback = move |query: &Query| {
            if query.content().starts_with('!') {
                engine_ddg_bang
            } else if query.content().starts_with('\\') {
                engine_ddg_direct
            } else if query.content().chars().any(|c| c.is_cjk()) {
                engine_bing_china
            } else {
                engine_google
            }
        };

        let fallback = RouterFn(move |query| Ok(Some(router::Route::new_static(fallback(query)))));

        (map, fallback)
    }
}

#[derive(Clone, Copy)]
struct EngineGoogle;

static URL_GOOGLE_SEARCH: &str = "https://www.google.com/search";

impl Engine for EngineGoogle {
    fn execute(&self, query: &Query) -> Result<ExecuteAction> {
        Ok(ExecuteAction::redirect_to_query(
            URL_GOOGLE_SEARCH,
            &[("q", query.content())],
        ))
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
struct EngineDuckDuckGo {
    is_bang: bool,
    is_direct: bool,
}

static URL_DUCKDUCKGO_SEARCH: &str = "https://duckduckgo.com";

impl Engine for EngineDuckDuckGo {
    fn execute(&self, query: &Query) -> Result<ExecuteAction> {
        Ok(ExecuteAction::redirect_to_query(
            URL_DUCKDUCKGO_SEARCH,
            &[("q", query.content())],
        ))
    }
}

#[derive(Clone, Copy)]
struct EngineBing {
    is_china: bool,
}

static URL_BING_SEARCH: &str = "https://www.bing.com/search";
static URL_BING_SEARCH_CHINA: &str = "https://cn.bing.com/search";

impl Engine for EngineBing {
    fn execute(&self, query: &Query) -> Result<ExecuteAction> {
        if self.is_china || query.residue().get(1) == Some(&"cn") {
            return Ok(ExecuteAction::redirect_to_query(
                URL_BING_SEARCH_CHINA,
                &[("q", query.content())],
            ));
        }

        Ok(ExecuteAction::redirect_to_query(
            URL_BING_SEARCH,
            &[("q", query.content())],
        ))
    }
}

#[derive(Clone, Copy)]
enum EngineChina {
    Baidu,
    Sogou,
}

impl Engine for EngineChina {
    fn execute(&self, query: &Query) -> Result<ExecuteAction> {
        match self {
            EngineChina::Baidu => Ok(ExecuteAction::redirect_to_query(
                "https://www.baidu.com/s",
                &[("wd", query.content())],
            )),
            EngineChina::Sogou => Ok(ExecuteAction::redirect_to_query(
                "https://www.sogou.com/web",
                &[("query", query.content())],
            )),
        }
    }
}

#[derive(Clone, Copy)]
struct EngineWikipedia;

static URL_WIKIPEDIA_SEARCH: &str = "https://en.wikipedia.org/w/index.php";
static URL_WIKIPEDIA_SEARCH_ZH: &str = "https://zh.wikipedia.org/w/index.php";
static URL_WIKIPEDIA_SEARCH_JA: &str = "https://ja.wikipedia.org/w/index.php";

impl Engine for EngineWikipedia {
    fn execute(&self, query: &Query) -> Result<ExecuteAction> {
        let lang = match query.residue().first() {
            Some(&"zh") => URL_WIKIPEDIA_SEARCH_ZH,
            Some(&"ja") => URL_WIKIPEDIA_SEARCH_JA,
            _ => {
                if query.content().chars().any(|c| c.is_cjk()) {
                    URL_WIKIPEDIA_SEARCH_ZH
                } else {
                    URL_WIKIPEDIA_SEARCH
                }
            }
        };

        Ok(ExecuteAction::redirect_to_query(
            lang,
            &[("search", query.content())],
        ))
    }
}
