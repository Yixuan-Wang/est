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
        let engine_scholar = arena.insert(EngineYixuanFavorite::GoogleScholar);
        let engine_arxiv = arena.insert(EngineYixuanFavorite::Arxiv);
        let engine_openreview = arena.insert(EngineYixuanFavorite::OpenReview);
        let engine_github = arena.insert(EngineYixuanFavorite::Github);
        let engine_wiktionary = arena.insert(EngineYixuanFavorite::Wiktionary);
        let engine_merriamwebster = arena.insert(EngineYixuanFavorite::MerriamWebster);
        let engine_oed = arena.insert(EngineYixuanFavorite::OED);
        let engine_perplexity = arena.insert(EngineYixuanFavorite::Perplexity);
        let engine_zitools = arena.insert(EngineYixuanFavorite::ZiTools);
        let engine_jisho = arena.insert(EngineYixuanFavorite::Jisho);
        let engine_zhihu = arena.insert(EngineYixuanFavorite::Zhihu);
        let engine_douban = arena.insert(EngineYixuanFavorite::Douban);
        let engine_bilibili = arena.insert(EngineYixuanFavorite::Bilibili);
        let engine_weibo = arena.insert(EngineYixuanFavorite::Weibo);
        let engine_xiaohongshu = arena.insert(EngineYixuanFavorite::Xiaohongshu);
        let engine_youtube = arena.insert(EngineYixuanFavorite::YouTube);

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
            (String::from("gs"), engine_scholar),
            (String::from("scholar"), engine_scholar),
            (String::from("ax"), engine_arxiv),
            (String::from("arxiv"), engine_arxiv),
            (String::from("or"), engine_openreview),
            (String::from("openreview"), engine_openreview),
            (String::from("gh"), engine_github),
            (String::from("github"), engine_github),
            (String::from("wd"), engine_wiktionary),
            (String::from("wikt"), engine_wiktionary),
            (String::from("mw"), engine_merriamwebster),
            (String::from("merriamwebster"), engine_merriamwebster),
            (String::from("oed"), engine_oed),
            (String::from("ppl"), engine_perplexity),
            (String::from("perplexity"), engine_perplexity),
            (String::from("zi"), engine_zitools),
            (String::from("zitools"), engine_zitools),
            (String::from("ji"), engine_jisho),
            (String::from("jisho"), engine_jisho),
            (String::from("zh"), engine_zhihu),
            (String::from("zhihu"), engine_zhihu),
            (String::from("db"), engine_douban),
            (String::from("douban"), engine_douban),
            (String::from("bl"), engine_bilibili),
            (String::from("bilibili"), engine_bilibili),
            (String::from("wb"), engine_weibo),
            (String::from("weibo"), engine_weibo),
            (String::from("xhs"), engine_xiaohongshu),
            (String::from("xiaohongshu"), engine_xiaohongshu),
            (String::from("yt"), engine_youtube),
            (String::from("youtube"), engine_youtube),
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

#[derive(Clone, Copy)]
enum EngineYixuanFavorite {
    GoogleScholar,
    Arxiv,
    OpenReview,
    Github,
    Wiktionary,
    MerriamWebster,
    OED,
    Perplexity,
    ZiTools,
    Jisho,
    Zhihu,
    Douban,
    Bilibili,
    Weibo,
    Xiaohongshu,
    YouTube,
}

impl Engine for EngineYixuanFavorite {
    fn execute(&self, query: &Query) -> Result<ExecuteAction> {
        match self {
            EngineYixuanFavorite::GoogleScholar => Ok(ExecuteAction::redirect_to_query(
                "https://scholar.google.com/scholar",
                &[("q", query.content())],
            )),
            EngineYixuanFavorite::Arxiv => Ok(ExecuteAction::redirect_to_query(
                "https://arxiv.org/search",
                &[("query", query.content())],
            )),
            EngineYixuanFavorite::OpenReview => Ok(ExecuteAction::redirect_to_query(
                "https://openreview.net/search",
                &[("query", query.content())],
            )),
            EngineYixuanFavorite::Github => Ok(ExecuteAction::redirect_to_query(
                "https://github.com/search",
                &[("q", query.content())],
            )),
            EngineYixuanFavorite::Wiktionary => Ok(ExecuteAction::redirect_to_query(
                "https://en.wiktionary.org/w/index.php",
                &[("search", query.content())],
            )),
            EngineYixuanFavorite::MerriamWebster => Ok(ExecuteAction::redirect_to(
                &url::Url::parse(&format!(
                    "https://www.merriam-webster.com/dictionary/{}",
                    query.content()
                ))
                .map(|url| url.to_string())
                .unwrap_or_else(|_| "https://www.merriam-webster.com".to_string()),
            )),
            EngineYixuanFavorite::OED => Ok(ExecuteAction::redirect_to_query(
                "https://www.oed.com/search/dictionary",
                &[("scope", "Entries"), ("q", query.content())],
            )),
            EngineYixuanFavorite::Perplexity => Ok(ExecuteAction::redirect_to_query(
                "https://www.perplexity.ai/search/new",
                &[("q", query.content())],
            )),
            EngineYixuanFavorite::ZiTools => Ok(ExecuteAction::redirect_to(
                &url::Url::parse(&format!("https://zi.tools/zi/{}", query.content()))
                    .map(|url| url.to_string())
                    .unwrap_or_else(|_| "https://zi.tools".to_string()),
            )),
            EngineYixuanFavorite::Jisho => {
                let url = url::Url::parse(&format!("https://jisho.org/search/{}", query.content()))
                    .map(|url| url.to_string())
                    .unwrap_or_else(|_| "https://jisho.org".to_string());
                Ok(ExecuteAction::redirect_to(&url))
            }
            EngineYixuanFavorite::Zhihu => Ok(ExecuteAction::redirect_to_query(
                "https://www.zhihu.com/search",
                &[("q", query.content())],
            )),
            EngineYixuanFavorite::Douban => Ok(ExecuteAction::redirect_to_query(
                "https://www.douban.com/search",
                &[("q", query.content())],
            )),
            EngineYixuanFavorite::Bilibili => Ok(ExecuteAction::redirect_to_query(
                "https://search.bilibili.com/all",
                &[("keyword", query.content())],
            )),
            EngineYixuanFavorite::Weibo => Ok(ExecuteAction::redirect_to_query(
                "https://s.weibo.com/weibo",
                &[("q", query.content())],
            )),
            EngineYixuanFavorite::Xiaohongshu => Ok(ExecuteAction::redirect_to_query(
                "https://www.xiaohongshu.com/search_result",
                &[("keyword", query.content())],
            )),
            EngineYixuanFavorite::YouTube => Ok(ExecuteAction::redirect_to_query(
                "https://www.youtube.com/results",
                &[("search_query", query.content())],
            )),
        }
    }
}
