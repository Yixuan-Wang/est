//! Search engines.
//!
//! Each search engine should be capable of
//! - [`Search`]: transform a query into a

use crate::{common::Result, query::Query};

use slotmap::{new_key_type, SlotMap};
use std::{
    borrow::Borrow,
    ops::{Deref, DerefMut},
    sync::Arc,
};
use tokio::sync::RwLock;
use url::Url;

pub trait Engine {
    /// Accepts a query and returns a SearchAction.
    fn execute(&self, query: &Query) -> Result<ExecuteAction>;
}

new_key_type! {
    pub struct HandleEngine;
}

pub struct ArenaEngine {
    arena: SlotMap<HandleEngine, RefEngine>,
}

pub struct RefEngine {
    pub handle: HandleEngine,
    pub engine: Arc<RwLock<dyn Engine + Send + Sync>>,
}

impl Deref for RefEngine {
    type Target = Arc<RwLock<dyn Engine + Send + Sync>>;

    fn deref(&self) -> &Self::Target {
        &self.engine
    }
}

impl DerefMut for RefEngine {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.engine
    }
}

impl Default for ArenaEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ArenaEngine {
    pub fn new() -> Self {
        ArenaEngine {
            arena: SlotMap::with_key(),
        }
    }

    pub fn insert<E: Engine + Send + Sync + 'static>(&mut self, engine: E) -> HandleEngine {
        self.arena.insert_with_key(|h| RefEngine {
            handle: h,
            engine: Arc::new(RwLock::new(engine)),
        })
    }

    pub fn insert_iter<I, E>(
        &mut self,
        engines: I,
    ) -> impl Iterator<Item = HandleEngine> + use<'_, I, E>
    where
        I: IntoIterator<Item = E>,
        E: Engine + 'static + Send + Sync,
    {
        engines.into_iter().map(|engine| self.insert(engine))
    }

    pub fn pop(&mut self, handle: HandleEngine) -> Option<RefEngine> {
        self.arena.remove(handle)
    }

    pub fn remove(&mut self, handle: HandleEngine) {
        let Some(_) = self.arena.remove(handle) else {
            return;
        };
    }

    pub fn get(&self, handle: HandleEngine) -> Option<Arc<RwLock<dyn Engine + Send + Sync>>> {
        let engine = self.arena.get(handle)?;
        Some(Arc::clone(&engine.engine))
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum ExecuteAction {
    /// Redirect to a URL.
    /// The constructor must guarantee the URL is valid.
    /// Use [`SearchAction::redirect_to`] or [`SearchAction::redirect_to_query`] to build this action.
    Redirect(String),
}

impl ExecuteAction {
    /// A helper function to build a [`SearchAction::Redirect`] action with a string.
    #[inline]
    pub fn redirect_to(url: &str) -> Self {
        ExecuteAction::Redirect(Url::parse(url).unwrap().to_string())
    }

    /// A helper function to build a [`SearchAction::Redirect`] action with a base URL and an iterator of queries.
    #[inline]
    pub fn redirect_to_query<I, K, V>(base: &str, queries: I) -> Self
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        ExecuteAction::Redirect(
            url::Url::parse_with_params(base, queries)
                .unwrap()
                .to_string(),
        )
    }
}
