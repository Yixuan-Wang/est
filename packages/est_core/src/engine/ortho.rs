//! An engine that forwards to specified engines based on Unicode script type.
use super::{Engine, EngineNode};
use crate::{reaction::Forward, Instance, Query, Reaction};
use icu_properties::{script, Script};
use std::{future::Future, sync::LazyLock};

static UNICODE_SCRIPT: LazyLock<script::ScriptWithExtensionsBorrowed<'_>> =
    LazyLock::new(|| script::script_with_extensions());

pub enum Ortho {
    Single {
        identifier: String,
        script: Script,
        to: String,
        default: String,
    },
    Hierarchical {
        identifier: String,
        default: String,
        scripts: Vec<(Script, String)>,
    },
}

impl Engine for Ortho {
    fn identifier(&self) -> &str {
        match self {
            Self::Single { identifier, .. } => identifier,
            Self::Hierarchical { identifier, .. } => identifier,
        }
    }

    fn react<'e, 'q: 'e, 'i: 'e>(
        &'e self,
        query: &'q Query,
        _instance: &'i Instance,
    ) -> impl Future<Output = Reaction> + Send + 'e {
        let engine_id = match self {
            Self::Single {
                script,
                to,
                default,
                ..
            } => {
                let has_script = query
                    .content
                    .chars()
                    .any(|c| UNICODE_SCRIPT.has_script(c as u32, *script));
                if has_script {
                    to
                } else {
                    default
                }
            }
            Self::Hierarchical {
                default, scripts, ..
            } => scripts
                .iter()
                .find(|(script, _)| {
                    query
                        .content
                        .chars()
                        .any(|c| UNICODE_SCRIPT.has_script(c as u32, *script))
                })
                .map(|(_, to)| to)
                .unwrap_or(default),
        };

        let reaction = Forward::Mention(engine_id.to_owned(), 1);

        async move { Ok(reaction.into()) }
    }
}

impl From<Ortho> for EngineNode {
    fn from(ortho: Ortho) -> Self {
        Self::Ortho(ortho)
    }
}

pub(crate) mod compose {
    use icu_properties::Script;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Debug)]
    pub(crate) struct OrthoScript {
        pub script: String,
        pub to: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(untagged)]
    pub(crate) enum Ortho {
        Single {
            default: String,
            #[serde(flatten)]
            script: OrthoScript,
        },
        Hierarchical {
            default: String,
            scripts: Vec<OrthoScript>,
        },
    }

    fn get_script(script: OrthoScript) -> (Script, String) {
        let OrthoScript { script, to } = script;
        let script = Script::name_to_enum_mapper()
            .get_loose(script.as_str())
            .unwrap_or_else(|| panic!("Invalid script name in ortho engine: {}", script));
        (script, to)
    }

    impl Ortho {
        pub(crate) fn build(self, identifier: String) -> crate::engine::EngineNode {
            match self {
                Self::Single { default, script } => {
                    let (script, to) = get_script(script);

                    super::Ortho::Single {
                        identifier,
                        script,
                        to,
                        default,
                    }
                }
                Self::Hierarchical { default, scripts } => {
                    let scripts = scripts.into_iter().map(get_script).collect();

                    super::Ortho::Hierarchical {
                        identifier,
                        default,
                        scripts,
                    }
                }
            }
            .into()
        }
    }
}
