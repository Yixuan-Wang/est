use std::ops::Deref;

use crate::engine::Engine;
mod parse;

pub struct QueryMention<'i> {
    seq: Vec<&'i str>,
    resolved: usize,
}

impl<'i> From<Vec<&'i str>> for QueryMention<'i> {
    fn from(seq: Vec<&'i str>) -> Self {
        QueryMention {
            seq,
            resolved: 0,
        }
    }
}

impl<'i> Deref for QueryMention<'i> {
    type Target = [&'i str];

    fn deref(&self) -> &Self::Target {
        &self.seq
    }
}

impl<'i> QueryMention<'i> {
    pub fn resolve_one(&mut self) {
        if self.resolved < self.seq.len() {
            self.resolved += 1;
        }
    }

    pub fn unresolve_one(&mut self) {
        if self.resolved > 0 {
            self.resolved -= 1;
        }
    }

    pub fn resolved(&self) -> &[&'i str] {
        &self.seq[..self.resolved]
    }

    pub fn residue(&self) -> &[&'i str] {
        if self.resolved < self.seq.len() {
            &self.seq[self.resolved..]
        } else {
            &[]
        }
    }
}


pub struct Query<'i> {
    pub(crate) mention: QueryMention<'i>,
    pub(crate) content: &'i str,
}

impl<'i> Query<'i> {
    pub fn from_content(content: &'i str) -> Self {
        Query {
            mention: Vec::default().into(),
            content,
        }
    }

    pub fn mention(&self) -> &QueryMention<'i> {
        &self.mention
    }

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

