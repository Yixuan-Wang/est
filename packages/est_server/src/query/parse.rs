use crate::query::Query;

use winnow::{
    ascii::alphanumeric1, combinator::{delimited, opt, preceded, separated}, prelude::*, stream::AsChar, token::take_while
};

// Syntax:
// ```plaintext
// @mention.mention.mention content
// ```

fn parse_mention<'i>(input: &mut &'i str) -> PResult<Vec<&'i str>> {
    delimited(
        "@",
        separated(1.., alphanumeric1, "."),
        take_while(0.., AsChar::is_space),
    ).parse_next(input)
}

pub(super) fn parse_query<'i>(input: &'i str) -> PResult<Query<'i>> {
    let (input, mention) = opt(parse_mention).parse_peek(&input).unwrap();

    Ok(Query {
        mention: mention.unwrap_or_default().into() ,
        content: input,
    })
}
