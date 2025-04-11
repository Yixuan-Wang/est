use std::{default::Default, str::FromStr};

use smallvec::SmallVec;

mod parse;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Query {
    pub mention: SmallVec<[String; 1]>,
    pub content: String,
    pub scope: Option<String>,
}

impl Query {
    #[inline]
    pub fn content(&self) -> &str {
        &self.content
    }

    #[inline]
    pub fn mention_head(&self) -> &str {
        self.mention.first().map(String::as_str).unwrap_or("")
    }

    #[inline]
    pub fn mention_tail(&self) -> &[String] {
        self.mention.get(1..).unwrap_or(&[])
    }

    #[inline]
    pub fn with_mention(&self, mention: SmallVec<[String; 1]>) -> Self {
        Self {
            mention,
            content: self.content.clone(),
            scope: self.scope.clone(),
        }
    }
}

impl FromStr for Query {
    type Err = winnow::error::ContextError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use winnow::{error::ParseError, Parser};
        parse::parse_query.parse(s).map_err(ParseError::into_inner)
    }
}

#[cfg(test)]
mod test {
    use super::Query;

    #[test]
    fn test_parse_query() {
        let input = "@mention";
        let reference = Query {
            mention: vec!["mention".to_string()].into(),
            content: "".to_string(),
            scope: None,
        };
        let target = input.parse();
        assert_eq!(target, Ok(reference));
    }
}
