use thiserror::Error;
use url::Url;

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum AcceptanceErr {
    #[error("No such specified engine.")]
    NoEngine,
}

#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum ReactionVerb {
    Navigate(Navigate),
    Forward(Forward),
}

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum ReactionErr {
    #[error("Not found.")]
    Nothing,

    #[error("Invalid configuration: {0}")]
    BadConfig(String),

    #[error("Query not accepted: {0}")]
    NotAccepted(#[from] AcceptanceErr),

    #[error("Est core failure: {0}")]
    Panic(String),

    #[error("Too many forwards before deciding on an engine to process the query.")]
    TooManyForward,
}

pub type Reaction = Result<ReactionVerb, ReactionErr>;

#[derive(Clone, Debug)]
pub struct Navigate {
    url: Url,
}

impl From<Navigate> for ReactionVerb {
    fn from(navigate: Navigate) -> Self {
        ReactionVerb::Navigate(navigate)
    }
}

impl Navigate {
    pub fn from_str(string: impl AsRef<str>, blame_config: bool) -> Reaction {
        let url = Url::parse(string.as_ref()).map_err(|err| {
            if blame_config {
                ReactionErr::BadConfig(format!("Invalid url: {}", err))
            } else {
                ReactionErr::Nothing
            }
        })?;

        Ok(ReactionVerb::Navigate(Navigate { url }))
    }

    pub fn url(&self) -> &Url {
        &self.url
    }
}

#[derive(Clone, Debug)]
pub enum Forward {
    /// Prepend a new mention segment to the query, and optionally drop the first-n mention segments.
    ///
    /// Given a query `@a.b.c`, the effect will be:
    /// - `Mention(d, 0)` -> `@d.a.b.c`
    /// - `Mention(d, 1)` -> `@d.b.c`
    /// - `Mention(d, 2)` -> `@d.c`
    /// - `Mention(d, 3)` -> `@d`
    /// - `Mention(d, 4)` -> `@d`
    Mention(String, usize),
}

impl From<Forward> for ReactionVerb {
    fn from(forward: Forward) -> Self {
        ReactionVerb::Forward(forward)
    }
}
