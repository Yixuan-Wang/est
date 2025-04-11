use super::Query;

use icu_properties::sets::{blank, id_continue, id_start, CodePointSetDataBorrowed};
use winnow::{
    combinator::{alt, dispatch, fail, repeat, separated},
    prelude::*,
    token::{any, one_of, take_while},
};

static UNICODE_ID_START: CodePointSetDataBorrowed<'_> = id_start();
static UNICODE_ID_CONTINUE: CodePointSetDataBorrowed<'_> = id_continue();
static UNICODE_ID_BLANK: CodePointSetDataBorrowed<'_> = blank();

macro_rules! contains {
    (not $set:expr) => {
        |c| !$set.contains(c)
    };
    ($set:expr) => {
        |c| $set.contains(c)
    };
}

const SIGIL_MENTION: char = '@';
const SIGIL_MENTION_ALT: char = '\u{FF20}';

const SIGIL_SCOPE: char = '!';
const SIGIL_SCOPE_ALT: char = '\u{FF01}';

const SIGIL_PATH_SEP: (char, char) = ('.', '\u{3002}');

#[derive(Debug, Clone, PartialEq, Eq)]
enum Segment<'i> {
    Whitespace(&'i str),
    Mention(Vec<&'i str>),
    Scope(&'i str),
    Content(&'i str),
}

fn parse_identifier<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
    (
        one_of(contains!(UNICODE_ID_START)),
        take_while(0.., contains!(UNICODE_ID_CONTINUE)),
    )
        .take()
        .parse_next(input)
}

fn parse_mention<'i>(input: &mut &'i str) -> ModalResult<Segment<'i>> {
    separated(1.., parse_identifier, one_of(SIGIL_PATH_SEP))
    .map(Segment::Mention)
    .parse_next(input)
}

fn parse_scope<'i>(input: &mut &'i str) -> ModalResult<Segment<'i>> {
    take_while(0.., contains!(not UNICODE_ID_BLANK))
        .map(Segment::Scope)
        .parse_next(input)
}

fn parse_whitespace<'i>(input: &mut &'i str) -> ModalResult<Segment<'i>> {
    take_while(1.., contains!(UNICODE_ID_BLANK))
        .map(Segment::Whitespace)
        .parse_next(input)
}

fn parse_content<'i>(input: &mut &'i str) -> ModalResult<Segment<'i>> {
    take_while(1.., contains!(not UNICODE_ID_BLANK))
        .map(Segment::Content)
        .parse_next(input)
}

fn parse_sigil<'i>(input: &mut &'i str) -> ModalResult<Segment<'i>> {
    dispatch! { any;
        SIGIL_MENTION | SIGIL_MENTION_ALT => parse_mention,
        SIGIL_SCOPE | SIGIL_SCOPE_ALT => parse_scope,
        _ => fail,
    }
    .parse_next(input)
}


pub fn parse_query(input: &mut &str) -> ModalResult<Query> {
    let segments: Vec<Segment> = repeat(0.., alt((
        parse_sigil,
        parse_whitespace,
        parse_content,
    )))
        .parse_next(input)?;

    let mut mention: Option<Vec<&str>> = None;
    let mut scope: Option<&str> = None;
    let mut content: String = String::new();
    let mut in_content = false;

    for segment in segments {
        if !matches!(segment, Segment::Whitespace(..)) {
            in_content = matches!(segment, Segment::Content(..));
        }

        match segment {
            Segment::Mention(m) => { let _ = mention.insert(m); }
            Segment::Whitespace(w) => { if in_content { content.push_str(w); } }
            Segment::Content(c) => { content.push_str(c); }
            Segment::Scope(s) => { let _ = scope.insert(s); }
        }
    }

    let mention = mention.map(|v| v.into_iter().map(String::from).collect()).unwrap_or_default();
    let scope = scope.map(String::from);

    let trimmed = content.trim();
    if content != trimmed {
        content = trimmed.to_string();
    }

    Ok(Query {
        mention,
        content,
        scope,
    })
}

#[cfg(test)]
mod test {
    use winnow::Parser;
    use super::Segment;

    #[test]
    fn test_parse_identifier() {
        for input in ["hello", "你好", "hello_world", "hello1"] {
            let result = super::parse_identifier.parse(input).unwrap();
            assert_eq!(result, input);
        }
    }

    #[test]
    fn test_parse_mention() {
        let input = "mention.hi.you";
        let result = super::parse_mention.parse(input).unwrap();
        assert_eq!(result, Segment::Mention(vec!["mention", "hi", "you"]));
    }

    #[test]
    fn test_parse_query() {
        use super::Query;
        let input = "@mention.hello content hi there ";
        let reference = Query {
            mention: vec!["mention".to_string(), "hello".to_string()].into(),
            content: "content hi there".to_string(),
            scope: None,
        };
        let target = super::parse_query.parse(input).unwrap();
        assert_eq!(target, reference);
    }
}
