use std::{collections::HashMap, sync::LazyLock};

use crate::{
    common::Result,
    engine::{ArenaEngine, Engine, ExecuteAction},
    query::Query,
    router::Router,
};

pub struct PluginJavascript;

impl PluginJavascript {
    pub fn router(&self, arena: &mut ArenaEngine) -> impl Router {
        arena.insert(EngineJavascript)
    }
}

#[derive(Clone, Copy)]
struct EngineJavascript;

// impl Introduce for PluginJavascript {
//     fn introduce(&self, query: &Query) -> IntroduceResult {
//         let Some(root) = query.mention().first() else {
//             return Err(IntroduceError::Backtrack);
//         };

//         match root {
//             &"js" | &"javascript" => Ok(Box::new(EngineJavascript)),
//             _ => Err(IntroduceError::Backtrack),
//         }
//     }
// }

static MAP_JAVASCRIPT_HANDLE_TO_SITE: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
    let mut map = HashMap::new();

    // Frameworks
    map.insert("react", "reactjs.org");
    map.insert("vue", "vuejs.org");
    map.insert("svelte", "svelte.dev");
    map.insert("solid", "solidjs.com");
    map.insert("alpine", "alpinejs.dev");

    // Bundlers
    map.insert("rollup", "rollupjs.org");
    map.insert("vite", "vitejs.dev");

    // Suites
    map.insert("next", "nextjs.org");
    map.insert("nuxt", "nuxtjs.org");
    map.insert("astro", "astro.build");

    // Runtime
    map.insert("deno", "deno.land");
    map.insert("bun", "bun.sh");

    // Vue Ecosystem
    map.insert("vue-router", "router.vuejs.org");
    map.insert("pinia", "pinia.vuejs.org");
    map.insert("vueuse", "vueuse.org");

    // Pupeeter
    map.insert("puppeteer", "pptr.dev");
    map.insert("playwright", "playwright.dev");

    // CSS
    map.insert("tailwind", "tailwindcss.com");
    map.insert("pico", "picocss.com");

    // Testing
    map.insert("vitest", "vitest.dev");

    map
});

impl EngineJavascript {
    fn search_fallback(&self, query: &Query) -> Result<ExecuteAction> {
        if let Some(handle) = query.residue().first() {
            if MAP_JAVASCRIPT_HANDLE_TO_SITE.contains_key(handle) {
                return Ok(ExecuteAction::redirect_to_query(
                    "https://google.com/search",
                    &[(
                        "q",
                        format!(
                            "site:{} {}",
                            MAP_JAVASCRIPT_HANDLE_TO_SITE[handle],
                            query.content()
                        ),
                    )],
                ));
            }
        };

        let handle = query
            .residue()
            .get(0..)
            .map(|parts| parts.join(" "))
            .map(|s| format!("{} ", s))
            .unwrap_or_default();

        Ok(ExecuteAction::redirect_to_query(
            "https://www.google.com/search",
            &[("q", format!("javascript {}{}", handle, query.content()))],
        ))
    }
}

impl Engine for EngineJavascript {
    fn execute(&self, query: &Query) -> Result<ExecuteAction> {
        match query.residue().get(0..) {
            None | Some([]) | Some(["mdn"]) => Ok(ExecuteAction::redirect_to_query(
                "https://developer.mozilla.org/en-US/search",
                &[("q", query.content())],
            )),
            Some(["npm"]) => Ok(ExecuteAction::redirect_to_query(
                "https://www.npmjs.com/search",
                &[("q", query.content())],
            )),
            Some(["jsr"]) => Ok(ExecuteAction::redirect_to_query(
                "https://jsr.io/packages",
                &[("search", query.content())],
            )),
            Some(["node"]) => Ok(ExecuteAction::redirect_to_query(
                "https://nodejs.org/en/search",
                &[("q", query.content())],
            )),
            Some(["uno"]) => Ok(ExecuteAction::redirect_to_query(
                "https://unocss.dev/interactive",
                &[("s", query.content())],
            )),
            Some(["icones"]) => Ok(ExecuteAction::redirect_to_query(
                "https://icones.netlify.app/collection/all",
                &[("s", query.content())],
            )),
            Some(["caniuse"]) => Ok(ExecuteAction::redirect_to_query(
                "https://caniuse.com/",
                &[("search", query.content())],
            )),
            _ => self.search_fallback(query),
        }
    }
}
