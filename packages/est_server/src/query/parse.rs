//! Parses a query string into a [`Query`](super::Query) struct.
//!
//! Syntax:
//! ```plaintext
//! @mention.mention.mention content
//! ```

use super::Query;

use winnow::{
    ascii::alphanumeric1,
    combinator::{delimited, opt, separated},
    prelude::*,
    stream::AsChar,
    token::take_while,
};

fn parse_mention<'i>(input: &mut &'i str) -> PResult<Vec<&'i str>> {
    delimited(
        "@",
        separated(1.., alphanumeric1, "."),
        take_while(0.., AsChar::is_space),
    )
    .parse_next(input)
}

pub(super) fn parse_query<'i>(input: &'i str) -> PResult<Query<'i>> {
    let (input, mention) = opt(parse_mention).parse_peek(&input)?;

    Ok(Query {
        mention: mention.unwrap_or_default().into(),
        content: input,
    })
}
