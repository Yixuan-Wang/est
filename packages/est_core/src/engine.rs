use crate::{AcceptanceErr, Instance, Query, Reaction};
use cloze::ClozeScoped;
use futures::{future::BoxFuture, FutureExt};
use slotmap::{new_key_type, SlotMap};
use std::{collections::HashMap, future::Future};
use thiserror::Error;

pub mod alias;
pub mod cloze;
pub mod namespace;
pub mod ortho;

use self::{alias::Alias, cloze::Cloze, namespace::Namespace, ortho::Ortho};

pub trait Engine {
    /// Get the identifier of the engine.
    fn identifier(&self) -> &str;

    /// Process a query and generate a reaction to it.
    fn react<'e, 'q: 'e, 'i: 'e>(
        &'e self,
        query: &'q Query,
        instance: &'i Instance,
    ) -> impl Future<Output = Reaction> + Send + 'e;

    /// Whether the engine can further handle a query.
    /// If so, return the identifier.
    /// The default implementation will always accept.
    fn accept(&self, _query: &Query, _instance: &Instance) -> Result<(), AcceptanceErr> {
        Ok(())
    }
}

new_key_type! { pub(crate) struct EngineKey; }

#[non_exhaustive]
pub enum EngineNode {
    Alias(Alias),
    Namespace(Namespace),
    Cloze(Cloze),
    ClozeScoped(ClozeScoped),
    Ortho(Ortho),
}

impl EngineNode {
    pub fn accept(&self, query: &Query, instance: &Instance) -> Result<(), AcceptanceErr> {
        match self {
            Self::Alias(alias) => alias.accept(query, instance),
            Self::Namespace(namespace) => namespace.accept(query, instance),
            Self::Cloze(cloze) => cloze.accept(query, instance),
            Self::ClozeScoped(cloze_scoped) => cloze_scoped.accept(query, instance),
            Self::Ortho(ortho) => ortho.accept(query, instance),
        }
    }

    pub fn react<'e, 'q: 'e, 'i: 'e>(
        &'e self,
        query: &'q Query,
        instance: &'i Instance,
    ) -> BoxFuture<'e, Reaction> {
        match self {
            Self::Alias(alias) => alias.react(query, instance).boxed(),
            Self::Namespace(namespace) => namespace.react(query, instance).boxed(),
            Self::Cloze(cloze) => cloze.react(query, instance).boxed(),
            Self::ClozeScoped(cloze_scoped) => cloze_scoped.react(query, instance).boxed(),
            Self::Ortho(ortho) => ortho.react(query, instance).boxed(),
        }
    }
}

pub(crate) struct EngineRegistry {
    engines: SlotMap<EngineKey, EngineNode>,
    ids: HashMap<String, EngineKey>,
    description: HashMap<EngineKey, Option<String>>,
}

#[derive(Debug, Error)]
pub(crate) enum EngineRegistryModifyError {
    #[error("Engine with id {0} already exists.")]
    AlreadyExists(String),
    #[error("Engine with id {0} does not exist.")]
    NotFound(String),
}

impl EngineRegistry {
    pub(crate) fn get(&self, id: impl AsRef<str>) -> Option<&EngineNode> {
        let id = id.as_ref();
        self.ids.get(id).and_then(|key| self.engines.get(*key))
    }

    pub(crate) fn description(
        &self,
        id: impl AsRef<str>,
    ) -> Option<&str> {
        let id = id.as_ref();
        self.ids.get(id).and_then(|key| {
            self.description
                .get(key)
                .map(Option::as_deref)
                .flatten()
        })
    }

    pub(crate) fn iter_ids(&self) -> impl Iterator<Item = &String> {
        self.ids.keys()
    }

    pub(crate) fn alias(
        &mut self,
        id: impl AsRef<str>,
        to: impl AsRef<str>,
    ) -> Result<(), EngineRegistryModifyError> {
        let id = id.as_ref();
        if self.ids.contains_key(id) {
            return Err(EngineRegistryModifyError::AlreadyExists(id.to_string()));
        }

        let key = self
            .ids
            .get(to.as_ref())
            .ok_or_else(|| EngineRegistryModifyError::NotFound(to.as_ref().to_string()))?;
        self.ids.insert(id.to_string(), *key);
        Ok(())
    }
}

pub(crate) mod compose {
    use super::{
        alias::compose::Alias, cloze::compose::Cloze, namespace::compose::Namespace,
        ortho::compose::Ortho, EngineRegistry,
    };
    use serde::{Deserialize, Serialize};

    use slotmap::SlotMap;
    use std::collections::HashMap;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(untagged)]
    pub enum Shorthand {
        Single(String),
        Multiple(Vec<String>),
    }

    impl Default for Shorthand {
        fn default() -> Self {
            Self::Multiple(Vec::default())
        }
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Engine {
        #[serde(default)]
        pub(crate) id: String,
        #[serde(flatten)]
        pub(crate) engine: EngineType,
        #[serde(default)]
        pub(crate) shorthand: Shorthand,
        pub(crate) description: Option<String>,
    }

    #[non_exhaustive]
    #[derive(Debug, Deserialize, Serialize)]
    #[serde(tag = "type", rename_all = "kebab-case")]
    pub enum EngineType {
        Alias(Alias),
        Cloze(Cloze),
        Namespace(Namespace),
        Ortho(Ortho),
    }

    impl Engine {
        fn build(self, registry: &mut EngineRegistry) {
            let Engine {
                engine,
                id,
                shorthand,
                description,
            } = self;

            let identifier = id.clone();
            let engine = match engine {
                EngineType::Alias(alias) => alias.build(identifier),
                EngineType::Cloze(cloze) => cloze.build(identifier),
                EngineType::Namespace(namespace) => namespace.build(identifier),
                EngineType::Ortho(ortho) => ortho.build(identifier),
            };

            let key = registry.engines.insert(engine);
            registry.description.insert(key, description);
            registry.ids.insert(id, key);

            match shorthand {
                Shorthand::Single(s) => {
                    registry.ids.insert(s, key);
                }
                Shorthand::Multiple(shorthand) => {
                    for s in shorthand.into_iter() {
                        registry.ids.insert(s, key);
                    }
                }
            }
        }
    }

    impl FromIterator<Engine> for EngineRegistry {
        fn from_iter<T: IntoIterator<Item = Engine>>(iter: T) -> Self {
            let mut registry = EngineRegistry {
                engines: SlotMap::with_key(),
                ids: HashMap::new(),
                description: HashMap::new(),
            };

            for e in iter {
                e.build(&mut registry);
            }

            registry
        }
    }
}
