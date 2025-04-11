//! Core definitions for `est`
pub mod compose;
pub mod engine;
pub mod query;
pub mod reaction;

pub(crate) use engine::EngineNode;
pub use query::Query;
pub use reaction::{AcceptanceErr, Reaction, ReactionErr, ReactionVerb};

const MAX_FORWARD_DEPTH: u8 = 16;

pub struct Instance {
    pub(crate) engine_registry: engine::EngineRegistry,
}

impl Instance {
    pub(crate) fn engine<'i>(&'i self, id: &str) -> Result<&'i EngineNode, ReactionErr> {
        self.engine_registry
            .get(id)
            .ok_or(ReactionErr::NotAccepted(AcceptanceErr::NoEngine))
    }

    pub async fn react(&self, mut query: Query) -> Reaction {
        let mut engine = self.engine(query.mention_head())?;

        let mut count = 0u8;
        let reaction = loop {
            count += 1;
            if count > MAX_FORWARD_DEPTH {
                return Err(ReactionErr::TooManyForward);
            }

            engine.accept(&query, self)?;
            let reaction = engine.react(&query, self).await?;
            if let ReactionVerb::Forward(fwd) = reaction {
                use reaction::Forward::*;
                match fwd {
                    Mention(prepend, skip) => {
                        engine = self.engine(prepend.as_str())?;
                        match (query.mention.len(), skip) {
                            (0, _) => query.mention.push(prepend),
                            (_, 0) => query.mention.insert(0, prepend),
                            (_, 1) => query.mention[0] = prepend,
                            (len, 2) => {
                                query.mention[0] = prepend;
                                if len > 1 {
                                    query.mention.remove(1);
                                }
                            }
                            _ => {
                                query.mention = std::iter::once(prepend)
                                    .chain(query.mention.into_iter().skip(skip))
                                    .collect();
                            }
                        }
                    }
                }
            } else {
                break reaction;
            }
        };

        Ok(reaction)
    }
}
