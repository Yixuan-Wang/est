//! Declarative interface for configuring est cores.

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Compose {
    #[serde(default)]
    default: Option<String>,
    engines: Vec<crate::engine::compose::Engine>,
}

impl From<Compose> for crate::Instance {
    fn from(value: Compose) -> Self {
        let mut engine_registry: crate::engine::EngineRegistry =
            value.engines.into_iter().collect();

        if let Some(default) = value.default {
            engine_registry
                .alias("", &default)
                .expect("A default engine is already set by not appointing an id.");
        }

        Self { engine_registry }
    }
}
