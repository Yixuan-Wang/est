//! A namespace.
//! It may optionally act as an alias.

use super::{Engine, EngineNode};
use crate::{reaction::Forward, AcceptanceErr, Instance, Query, Reaction, ReactionErr};
use std::{collections::HashMap, future::Future};

pub struct Namespace {
    identifier: String,
    default: Option<String>,
    children: HashMap<String, String>,
}

impl Engine for Namespace {
    fn identifier(&self) -> &str {
        &self.identifier
    }

    #[allow(clippy::manual_async_fn)]
    fn react<'e, 'q: 'e, 'i: 'e>(
        &'e self,
        query: &'q Query,
        _instance: &'i Instance,
    ) -> impl Future<Output = Reaction> + Send + 'e {
        let mention = query.mention_tail();
        let mut next_engine = mention
            .first()
            .and_then(|f| self.children.get(f))
            .map(String::as_str);
        let is_next_engine_child = next_engine.is_some();

        if let Some(default) = &self.default {
            let _ = next_engine.get_or_insert(default);
        }

        let reaction = if let Some(engine_id) = next_engine {
            Ok(Forward::Mention(
                engine_id.to_string(),
                if is_next_engine_child { 2 } else { 1 },
            )
            .into())
        } else {
            Err(ReactionErr::Nothing)
        };

        async move { reaction }
    }

    fn accept(&self, _query: &Query, _instance: &Instance) -> Result<(), AcceptanceErr> {
        if self.default.is_none() {
            Err(AcceptanceErr::NoEngine)
        } else {
            Ok(())
        }
    }
}

impl From<Namespace> for EngineNode {
    fn from(namespace: Namespace) -> Self {
        Self::Namespace(namespace)
    }
}

pub(crate) mod compose {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Deserialize, Serialize, Debug)]
    pub(crate) struct Namespace {
        pub default: Option<String>,
        #[serde(default)]
        pub children: HashMap<String, String>,
    }

    impl Namespace {
        pub(crate) fn build(self, identifier: String) -> crate::engine::EngineNode {
            super::Namespace {
                identifier,
                default: self.default,
                children: self.children,
            }
            .into()
        }
    }
}
