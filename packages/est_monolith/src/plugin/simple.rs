//! A plugin that loads simple rules into a single engine.
//!

use serde::Deserialize;
use serde_with::{formats::PreferOne, serde_as, OneOrMany};
use toml::Value as TomlValue;

use crate::{
    common,
    engine::{ArenaEngine, Engine, ExecuteAction},
    query,
    router::{Router, RouterMapLeaves},
};

#[serde_as]
#[derive(Deserialize, Debug, Clone)]
pub struct ReplaceRuleEntry {
    /// The name of search engine.
    pub name: String,

    /// The template string of search engine.
    ///
    /// Use `{}` to represent the query string.
    pub template: String,

    /// The shortcut of search engine.
    #[serde_as(as = "OneOrMany<_, PreferOne>")]
    pub shortcut: Vec<String>,

    /// The description of search engine.
    /// Optional.
    pub description: Option<String>,

    /// The OpenSearch suggestion URL of search engine.
    /// Optional.
    pub suggestion: Option<String>,
}

pub struct PluginSimple {
    rules: Vec<ReplaceRuleEntry>,
}

impl PluginSimple {
    pub fn from_config(config: &TomlValue) -> Self {
        let get_config = || -> Option<TomlValue> {
            let config = config.get("plugin")?;
            config.get("simple").cloned()
        };
        let config = get_config();

        let get_rules = || -> Option<Vec<ReplaceRuleEntry>> {
            let config = config.clone()?;
            let rules = config.get("rules")?;

            TomlValue::try_into::<Vec<ReplaceRuleEntry>>(rules.clone()).ok()
        };

        Self {
            rules: get_rules().unwrap_or_default(),
        }
    }

    pub fn router(&self, arena: &mut ArenaEngine) -> impl Router {
        let map = self
            .rules
            .iter()
            .flat_map(|rule| {
                let engine = EngineSimple { rule: rule.clone() };
                let handle_engine = arena.insert(engine);

                std::iter::zip(
                    rule.shortcut.iter().cloned(),
                    std::iter::repeat(handle_engine),
                )
            })
            .collect::<std::collections::HashMap<_, _>>();

        RouterMapLeaves::new(map)
    }
}

struct EngineSimple {
    rule: ReplaceRuleEntry,
}

impl Engine for EngineSimple {
    fn execute(&self, query: &query::Query) -> common::Result<ExecuteAction> {
        let url = self.rule.template.replace("{}", query.content());

        // FIXME: Incomplete is not the right error type here.
        //        The config + query does not form a correct URL.
        ExecuteAction::redirect_to(&url).ok_or(common::Fail::Incomplete)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_from_config() {
        let config_str = r#"
[[plugin.simple.rules]]
name = "Google"
template = "https://www.google.com/search?q={}"
shortcut = "g"
description = "Search with Google"
suggestion = "https://www.google.com/complete/search?output=firefox&q={searchTerms}"

[[plugin.simple.rules]]
name = "Bing"
template = "https://www.bing.com/search?q={}"
shortcut = ["b", "bing"]
description = "Search with Bing"
suggestion = "https://www.bing.com/osjson.aspx?form=OSDJAS&query={searchTerms}"
        "#;

        let plugin = super::PluginSimple::from_config(&toml::from_str(config_str).unwrap());

        assert_eq!(plugin.rules.len(), 2);

        println!("{:?}", plugin.rules);
    }
}
