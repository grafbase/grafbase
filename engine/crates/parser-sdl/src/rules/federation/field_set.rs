use std::str::FromStr;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1},
    combinator::recognize,
    error::{convert_error, ParseError, VerboseError},
    multi::{many0_count, many1},
    sequence::{delimited, pair},
    AsChar, Finish, IResult, InputTakeAtPosition, Parser,
};
use registry_v2::Selection;
use serde::de::Error;

/// Newtype wrapper around registry::FieldSet that Deserializes from a String
#[derive(Clone, Debug)]
pub struct FieldSet(pub registry_v2::FieldSet);

impl<'de> serde::Deserialize<'de> for FieldSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse::<FieldSet>()
            .map_err(D::Error::custom)
    }
}

impl FromStr for FieldSet {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let (remainder, output) = parse_field_set(input)
            .finish()
            .map_err(|error| convert_error(input, error))?;

        if !remainder.is_empty() {
            return Err(format!(
                "Couldn't parse all of `{input}` as a FieldSet. `{remainder}` did not parse correctly"
            ));
        }

        Ok(output)
    }
}

fn parse_field_set(input: &str) -> IResult<&str, FieldSet, VerboseError<&str>> {
    many1(parse_selection)
        .map(|selections| FieldSet(registry_v2::FieldSet::new(selections)))
        .parse(input)
}

fn parse_selection(input: &str) -> IResult<&str, Selection, VerboseError<&str>> {
    alt((
        pair(ws(parse_name), ws(parse_selection_set)),
        ws(parse_name).map(|name| (name, vec![])),
    ))
    .map(|(field, selections)| Selection {
        field: field.to_string(),
        selections,
    })
    .parse(input)
}

fn parse_selection_set(input: &str) -> IResult<&str, Vec<Selection>, VerboseError<&str>> {
    delimited(tag("{"), many1(parse_selection), tag("}"))(input)
}

fn parse_name(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))
    .parse(input)
}

/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and
/// trailing whitespace, returning the output of `inner`.
fn ws<'a, F, O, E: ParseError<&'a str>>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Parser<&'a str, O, E>,
{
    delimited(ignored_chars, inner, ignored_chars)
}

/// nom parser for characters we ignore in GraphQL (commas & whitespace mostly)
pub fn ignored_chars<T, E: ParseError<T>>(input: T) -> IResult<T, T, E>
where
    T: InputTakeAtPosition,
    <T as InputTakeAtPosition>::Item: AsChar + Clone,
{
    input.split_at_position_complete(|item| {
        let c = item.as_char();
        !(c == ' ' || c == '\t' || c == '\r' || c == '\n' || c == ',')
    })
}

#[cfg(test)]
mod tests {
    use engine::registry::field_set::FieldSetDisplay;

    use super::*;

    fn roundtrip(input: &str) -> String {
        FieldSetDisplay(&input.parse::<FieldSet>().unwrap().0).to_string()
    }

    #[test]
    fn test_parsing_fieldset() {
        assert_eq!(roundtrip("foo"), "foo");
        assert_eq!(roundtrip("foo { bar }"), "foo { bar }");
        assert_eq!(roundtrip("foo { bar baz }"), "foo { bar baz }");
        assert_eq!(roundtrip("foo { bar baz { bun }}"), "foo { bar baz { bun } }");
        assert_eq!(
            roundtrip("foo { bar baz { bun }} bleep"),
            "foo { bar baz { bun } } bleep"
        );

        // Make sure we ignore whitespace & commas
        assert_eq!(roundtrip("   foo { bar baz { bun }}"), "foo { bar baz { bun } }");
        assert_eq!(roundtrip("   foo {   bar baz { bun }}"), "foo { bar baz { bun } }");
        assert_eq!(
            roundtrip("  , foo {,,, bar ,, baz ,,{ bun }, , }, "),
            "foo { bar baz { bun } }"
        );
    }

    fn expect_error(input: &str) {
        input
            .parse::<FieldSet>()
            .expect_err(&format!("Expected parsing `{input}` to fail"));
    }

    #[test]
    fn test_failures() {
        expect_error("foo {{");
        expect_error("foo {");
        expect_error("foo {}");
        expect_error("{ foo }");
        expect_error("{{");
        expect_error("1foo {");
    }
}
