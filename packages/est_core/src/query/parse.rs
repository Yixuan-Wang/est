use super::Query;

use icu_properties::sets::{blank, id_continue, id_start, CodePointSetDataBorrowed};
use winnow::{
    combinator::{delimited, opt, separated},
    prelude::*,
    token::{one_of, rest, take_while},
};

static UNICODE_ID_START: CodePointSetDataBorrowed<'_> = id_start();
static UNICODE_ID_CONTINUE: CodePointSetDataBorrowed<'_> = id_continue();
static UNICODE_ID_BLANK: CodePointSetDataBorrowed<'_> = blank();

macro_rules! contains {
    ($set:expr) => {
        |c| $set.contains(c)
    };
}

const SIGIL_MENTION: (char, char) = ('@', '\u{FF20}');
const SIGIL_PATH_SEP: (char, char) = ('.', '\u{3002}');

fn parse_identifier<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
    (
        one_of(contains!(UNICODE_ID_START)),
        take_while(0.., contains!(UNICODE_ID_CONTINUE)),
    )
        .take()
        .parse_next(input)
}

fn parse_mention<'i>(input: &mut &'i str) -> ModalResult<Vec<&'i str>> {
    delimited(
        one_of(SIGIL_MENTION),
        separated(1.., parse_identifier, one_of(SIGIL_PATH_SEP)),
        take_while(0.., contains!(UNICODE_ID_BLANK)),
    )
    .parse_next(input)
}

pub fn parse_query(input: &mut &str) -> ModalResult<Query> {
    let mention = opt(parse_mention).parse_next(input)?;
    let content = rest.parse_next(input)?;

    Ok(Query {
        mention: mention
            .unwrap_or_default()
            .into_iter()
            .map(String::from)
            .collect(),
        content: content.to_owned(),
    })
}

#[cfg(test)]
mod test {
    use winnow::Parser;

    #[test]
    fn test_parse_identifier() {
        for input in ["hello", "你好", "hello_world", "hello1"] {
            let result = super::parse_identifier.parse(input).unwrap();
            assert_eq!(result, input);
        }
    }

    #[test]
    fn test_parse_mention() {
        let input = "@mention";
        let result = super::parse_mention.parse(input).unwrap();
        assert_eq!(result, vec!["mention"]);
    }

    #[test]
    fn test_parse_query() {
        use super::Query;
        let input = "@mention.hello content hi there";
        let reference = Query {
            mention: vec!["mention".to_string(), "hello".to_string()].into(),
            content: "content hi there".to_string(),
        };
        let target = super::parse_query.parse(input).unwrap();
        assert_eq!(target, reference);
    }
}
