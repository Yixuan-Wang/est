//! A query against the router and engines.
//! A typical pipeline for a query is:
//!
//! 1. A query formulated in the form of a string is parsed into a [`Query`] struct.
//! 2. The [`Query`] is passed to the [`Router`](crate::router::Router), which routes the query to the appropriate [`Engine`](crate::engine::Engine).
//! 3. The [`Engine`](crate::engine::Engine) processes the result and returns an [`Action`].

use std::ops::Deref;
mod parse;

/// The [mention](https://en.wikipedia.org/wiki/Mention_(blogging)) in a query.
/// Though not enforced,
/// **the mention is the main source of information for the router to route the query**.
///
/// A mention can be represented as a sequence of strings, marked by the `@` sigil
/// and delimited by the `.` character.
/// The router will attempt to resolve the mention from left to right,
/// segment by segment, until it finds a match.
/// During resolution, the struct will be mutated to keep track of the resolved segments.
/// After routing, the mention will be split into two parts:
/// - The *resolved* part, which is the part of the mention that was successfully resolved and routed.
/// - The *residue* part, the remaining part of the mention.
///
/// The residue may be used by the engine to further refine the search.
pub struct QueryMention<'i> {
    seq: Vec<&'i str>,
    resolved: usize,
}

impl<'i> std::fmt::Display for QueryMention<'i> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.seq.split_at(self.resolved) {
            ([], []) => write!(f, ""),
            (resolved, []) => write!(f, "@{}", resolved.join(".")),
            (resolved, residue) => write!(f, "@{}({})", resolved.join("."), residue.join(".")),
        }
    }
}

impl<'i> From<Vec<&'i str>> for QueryMention<'i> {
    fn from(seq: Vec<&'i str>) -> Self {
        QueryMention { seq, resolved: 0 }
    }
}

impl<'i> Deref for QueryMention<'i> {
    type Target = [&'i str];

    fn deref(&self) -> &Self::Target {
        &self.seq
    }
}

impl<'i> QueryMention<'i> {
    /// Resolve one segment of the mention.
    /// This method should be called when the [`Router`](crate::router::Router) successfully routes a segment.
    pub fn resolve_one(&mut self) {
        if self.resolved < self.seq.len() {
            self.resolved += 1;
        }
    }

    /// Unresolve one segment of the mention.
    /// This method should be called when the [`Router`](crate::router::Router) backtracks.
    pub fn unresolve_one(&mut self) {
        if self.resolved > 0 {
            self.resolved -= 1;
        }
    }

    /// Get the resolved part of the mention, i.e. the part that was successfully routed.
    pub fn resolved(&self) -> &[&'i str] {
        &self.seq[..self.resolved]
    }

    /// Get the residue part of the mention, i.e. the part that was left unused by the router.
    pub fn residue(&self) -> &[&'i str] {
        if self.resolved < self.seq.len() {
            &self.seq[self.resolved..]
        } else {
            &[]
        }
    }
}

/// A query against the router and engines.
pub struct Query<'i> {
    pub mention: QueryMention<'i>,
    pub content: &'i str,
}

impl<'i> Query<'i> {
    /// A test-time helper to create a query from a content string.
    #[allow(dead_code)]
    pub(crate) fn from_content(content: &'i str) -> Self {
        Query {
            mention: Vec::default().into(),
            content,
        }
    }

    #[inline]
    /// See [`QueryMention::resolved`].
    pub fn resolved(&self) -> &[&'i str] {
        self.mention.resolved()
    }

    #[inline]
    /// See [`QueryMention::residue`].
    pub fn residue(&self) -> &[&'i str] {
        self.mention.residue()
    }

    #[inline]
    pub fn content(&self) -> &'i str {
        self.content
    }
}

impl<'i> TryFrom<&'i str> for Query<'i> {
    type Error = ();

    #[inline]
    fn try_from(content: &'i str) -> Result<Self, Self::Error> {
        parse::parse_query(content).map_err(|_| ())
    }
}
