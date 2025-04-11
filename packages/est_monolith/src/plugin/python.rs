//! Built-in plugin for Python search.

use std::{collections::HashMap, sync::LazyLock};

use crate::{
    common::{Fail, Result},
    engine::{ArenaEngine, Engine, ExecuteAction},
    query::Query,
    router::{Router, RouterTerminal},
};

pub struct PluginPython;

impl Default for PluginPython {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginPython {
    pub fn new() -> Self {
        PluginPython
    }

    pub fn router(&self, arena: &mut ArenaEngine) -> impl Router {
        let (handle_std, handle_pypi, handle_pkg) = (
            arena.insert(EnginePythonStdDocs),
            arena.insert(EnginePypi),
            arena.insert(EngineProminentPackages),
        );

        (
            RouterTerminal(handle_std),
            ("pypi", handle_pypi),
            handle_pkg,
        )
    }
}

#[derive(Clone, Copy)]
struct EnginePypi;
static URL_PYTHON_PYPI_SEARCH: &str = "https://pypi.org/search";

impl Engine for EnginePypi {
    fn execute(&self, query: &Query) -> Result<ExecuteAction> {
        Ok(ExecuteAction::redirect_to_query(
            URL_PYTHON_PYPI_SEARCH,
            &[("q", query.content())],
        ))
    }
}

#[derive(Clone, Copy)]
struct EnginePythonStdDocs;
static URL_PYTHON_STD_DOCS_SEARCH: &str = "https://docs.python.org/3/search.html";

impl Engine for EnginePythonStdDocs {
    fn execute(&self, query: &Query) -> Result<ExecuteAction> {
        Ok(ExecuteAction::redirect_to_query(
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
    map.insert(
        "ipy",
        "https://ipython.readthedocs.io/en/stable/search.html",
    );
    map.insert("sk", "https://scikit-learn.org/stable/search.html");
    map.insert("keras", "https://keras.io/search.html");
    map.insert("sqla", "https://docs.sqlalchemy.org/en/stable/search.html");
    map
});

#[derive(Clone, Copy)]
struct EngineProminentPackages;

impl EngineProminentPackages {
    fn search_fallback(&self, query: &Query) -> Result<ExecuteAction> {
        let handle = query
            .residue()
            .get(0..)
            .map(|parts| parts.join(" "))
            .map(|s| format!("{} ", s))
            .unwrap_or_default();

        Ok(ExecuteAction::redirect_to_query(
            "https://www.google.com/search",
            &[("q", format!("python {}{}", handle, query.content()))],
        ))
    }
}

impl Engine for EngineProminentPackages {
    fn execute(&self, query: &Query) -> Result<ExecuteAction> {
        let package = query.residue().first().ok_or(Fail::Incomplete)?;
        if let Some(base) = URL_PYTHON_PROMINENT_PACKAGES.get(package) {
            Ok(ExecuteAction::redirect_to_query(
                base,
                &[("q", query.content())],
            ))
        } else {
            self.search_fallback(query)
        }
    }
}
