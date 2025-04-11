//! An alias to another engine.
//! Except for the identifier, it behaves exactly the same as the engine it forwards to.
use super::{Engine, EngineNode};
use crate::{reaction::Forward, Instance, Query, Reaction};
use std::future::Future;

pub struct Alias {
    identifier: String,
    to: String,
}

impl Engine for Alias {
    fn identifier(&self) -> &str {
        &self.identifier
    }

    #[allow(clippy::manual_async_fn)]
    fn react<'e, 'q: 'e, 'i: 'e>(
        &'e self,
        _query: &'q Query,
        _instance: &'i Instance,
    ) -> impl Future<Output = Reaction> + Send + 'e {
        async move { Ok(Forward::Mention(self.to.clone(), 1).into()) }
    }
}

impl From<Alias> for EngineNode {
    fn from(alias: Alias) -> Self {
        Self::Alias(alias)
    }
}

pub(crate) mod compose {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize)]
    pub(crate) struct Alias {
        pub to: String,
    }

    impl Alias {
        pub(crate) fn build(self, identifier: String) -> crate::engine::EngineNode {
            super::Alias {
                identifier,
                to: self.to,
            }
            .into()
        }
    }
}
