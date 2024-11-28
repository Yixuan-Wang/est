//! Built-in plugin for Python search.


use std::{collections::HashMap, sync::{Arc, LazyLock}};

use crate::{engine::{Engine, ArenaEngine, SearchAction, SearchError, SearchResult}, query::Query, router::{Router, RouterMapLeaf, Terminal}};

pub struct PluginPython;

impl PluginPython {
    pub fn new() -> Self {
        PluginPython
    }

    pub fn router(
        &self,
        arena: &mut ArenaEngine,
    ) -> impl Router {
        let (
            handle_std,
            handle_pypi,
            handle_pkg,
        ) = (
            arena.insert(EnginePythonStdDocs),
            arena.insert(EnginePypi),
            arena.insert(EngineProminentPackages),
        );

        (
            Terminal(handle_std),
            ("pypi", handle_pypi),
            handle_pkg,
        )
    }
}

// impl Introduce for PluginPython {
//     fn introduce(
//         &self,
//         query: &Query,
//     ) -> IntroduceResult
//     {
//         let Some(root) = query.mention().first() else {
//             return Err(IntroduceError::Backtrack);
//         };

//         if root != &self.prefix {
//             // if root is a prefix of self.prefix, then we should prompt the user to use the full prefix
//             if self.prefix.starts_with(root) {
//                 print!("Did you mean @{}?", self.prefix);
//                 return Err(IntroduceError::Incomplete(format!("Did you mean @{}?", self.prefix)));
//             }
//             return Err(IntroduceError::Backtrack);
//         }

//         match query.mention() {
//             [_] => Ok(Box::new(self.engine_python_std_docs)),
//             [_, "pypi"] => Ok(Box::new(self.engine_pypi)),
//             _ => Ok(Box::new(self.engine_prominent_packages)),
//         }
//     }
// }


#[derive(Clone, Copy)]
struct EnginePypi;
static URL_PYTHON_PYPI_SEARCH: &str = "https://pypi.org/search";

impl Engine for EnginePypi {
    fn search(&self, query: &Query) -> SearchResult
    {
        Ok(SearchAction::redirect_to_query(
            URL_PYTHON_PYPI_SEARCH,
            &[("q", query.content())],
        ))
    }
}

#[derive(Clone, Copy)]
struct EnginePythonStdDocs;
static URL_PYTHON_STD_DOCS_SEARCH: &str = "https://docs.python.org/3/search.html";

impl Engine for EnginePythonStdDocs {
    fn search(&self, query: &Query) -> SearchResult
    {
        Ok(SearchAction::redirect_to_query(
            URL_PYTHON_STD_DOCS_SEARCH,
            &[("q", query.content())],
        ))
    }
}

static URL_PYTHON_PROMINENT_PACKAGES: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("np", "https://numpy.org/doc/stable/search.html");
    map.insert("pd", "https://pandas.pydata.org/docs/search.html");
    map.insert("plt", "https://matplotlib.org/stable/search.html");
    map.insert("sns", "https://seaborn.pydata.org/search.html");
    map.insert("torch", "https://pytorch.org/docs/stable/search.html");
    map.insert("ipy", "https://ipython.readthedocs.io/en/stable/search.html");
    map.insert("sk", "https://scikit-learn.org/stable/search.html");
    map.insert("keras", "https://keras.io/search.html");
    map.insert("sqla", "https://docs.sqlalchemy.org/en/stable/search.html");
    map
});

#[derive(Clone, Copy)]
struct EngineProminentPackages;

impl EngineProminentPackages {
    fn search_fallback(&self, query: &Query) -> SearchResult {
        let handle = query.mention().get(1..).map(|parts| parts.join(" ")).map(|s| format!("{} ", s)).unwrap_or_default();

        Ok(SearchAction::redirect_to_query(
            "https://www.google.com/search",
            &[("q", format!("python {}{}", handle, query.content()))],
        ))
    }
}

impl Engine for EngineProminentPackages {
    fn search(&self, query: &Query) -> SearchResult
    {
        let package = query.mention().get(1).ok_or(SearchError::Incomplete)?;
        if let Some(base) = URL_PYTHON_PROMINENT_PACKAGES.get(package) {
            Ok(SearchAction::redirect_to_query(
                base,
                &[("q", query.content())],
            ))
        } else {
            self.search_fallback(query)
        }
    }
}

#[test]
fn test_url() {
    let engine = EnginePypi;
    let query = Query::from_content("test");
    let action = engine.search(&query).unwrap();
    dbg!(action);
}
