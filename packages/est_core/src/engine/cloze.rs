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

impl From<Cloze> for EngineNode {
    fn from(cloze: Cloze) -> Self {
        Self::Cloze(cloze)
    }
}

pub(crate) mod compose {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Debug)]
    pub(crate) struct Cloze {
        pub template: String,
    }

    impl Cloze {
        pub(crate) fn build(self, identifier: String) -> crate::engine::EngineNode {
            super::Cloze {
                identifier,
                template: self.template,
            }
            .into()
        }
    }
}
