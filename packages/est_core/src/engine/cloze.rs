//! A simple engine that fills the designated blank with the query content.
//! This is similar to Chrome and Firefox's custom search engine feature,
//!   excepts that we use `{}` as a placeholder for the query.
use super::{Engine, EngineNode};
use crate::reaction::Navigate;
use crate::{Instance, Query, Reaction};
use std::future::Future;

pub struct Cloze {
    identifier: String,
    template: String,
}

pub struct ClozeScoped {
    identifier: String,
    template_default: String,
    template_scoped: String,
}


impl Engine for Cloze {
    fn identifier(&self) -> &str {
        &self.identifier
    }

    fn react<'e, 'q: 'e, 'i: 'e>(
        &'e self,
        query: &'q Query,
        _instance: &'i Instance,
    ) -> impl Future<Output = Reaction> + Send + 'e {
        let url = self.template.replace("{}", query.content());

        async { Navigate::from_str(url, true) }
    }
}

impl Engine for ClozeScoped {
    fn identifier(&self) -> &str {
        &self.identifier
    }

    fn react<'e, 'q: 'e, 'i: 'e>(
        &'e self,
        query: &'q Query,
        _instance: &'i Instance,
    ) -> impl Future<Output = Reaction> + Send + 'e {
        dbg!(&query);
        let url = if let Some(scope) = query.scope.as_ref() {
            self.template_scoped.replace("{!}", scope).replace("{}", query.content())
        } else {
            self.template_default.replace("{}", query.content())
        };

        async { Navigate::from_str(url, true) }
    }
}

impl From<Cloze> for EngineNode {
    fn from(cloze: Cloze) -> Self {
        Self::Cloze(cloze)
    }
}

impl From<ClozeScoped> for EngineNode {
    fn from(cloze: ClozeScoped) -> Self {
        Self::ClozeScoped(cloze)
    }
}

pub(crate) mod compose {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Debug)]
    pub(crate) struct Cloze {
        pub template: ClozeTemplate,
    }

    #[derive(Deserialize, Serialize, Debug)]
    #[serde(untagged)]
    pub enum ClozeTemplate {
        Single(String),
        Scoped {
            default: String,
            scoped: String,
        },
    }

    impl Cloze {
        pub(crate) fn build(self, identifier: String) -> crate::engine::EngineNode {
            match self.template {
                ClozeTemplate::Single(template) => super::Cloze {
                    identifier,
                    template,
                }.into(),
                ClozeTemplate::Scoped { default, scoped } => super::ClozeScoped {
                    identifier,
                    template_default: default,
                    template_scoped: scoped,
                }.into(),
            }
        }
    }
}
