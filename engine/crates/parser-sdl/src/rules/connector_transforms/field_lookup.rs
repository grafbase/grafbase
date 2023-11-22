use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1},
    combinator::{recognize, success},
    error::{convert_error, VerboseError},
    multi::{many0_count, many1, separated_list1},
    sequence::{delimited, pair},
    Finish, IResult, Parser,
};
use regex::Regex;

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(try_from = "String")]
/// Path to lookup one or more fields in one or more types.
///
/// This is parsed from a string like: `Query.{user,account}.*.email`
pub struct FieldLookup {
    /// The type(s) in the registry to start lookup at
    pub starting_type: PathSegment,
    /// The path from starting_type to our target type(s)
    pub path: Vec<PathSegment>,
    /// The field(s) on our target type(s)
    pub field: PathSegment,
}

impl TryFrom<String> for FieldLookup {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let (_, output) = parse_field_lookup(&value)
            .finish()
            .map_err(|error| convert_error(value.as_str(), error))?;

        Ok(FieldLookup {
            starting_type: output
                .starting_type
                .ok_or_else(|| {
                    format!("field lookups should always have at least two dot separated components.   found only one: {value}")
                })?
                .try_into()?,
            path: output
                .path
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            field: output.field.try_into()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PathSegment(Regex);

impl PathSegment {
    pub fn is_match(&self, name: &str) -> bool {
        self.0.is_match(name)
    }
}

impl TryFrom<AstPathSegment> for PathSegment {
    type Error = String;

    fn try_from(value: AstPathSegment) -> Result<Self, Self::Error> {
        Ok(PathSegment(
            Regex::new(&format!("{value}")).map_err(|error| error.to_string())?,
        ))
    }
}

/// An intermediate AST struct for FieldLookup.
///
/// This type is parsed out of a string then compiled into a FieldLookup
struct AstFieldLookup {
    /// The type(s) in the registry to start lookup at
    pub starting_type: Option<AstPathSegment>,
    /// The path from starting_type to our target type(s)
    pub path: Vec<AstPathSegment>,
    /// The field(s) on our target type(s)
    pub field: AstPathSegment,
}

/// An intermediate AST struct for PathSegment
///
/// This type is parsed out of a string then compiled into a PathSegment
#[derive(Clone, Debug)]
struct AstPathSegment(Vec<AstMatcher>);

/// Another intermediate AST struct
#[derive(Clone, Debug)]
enum AstMatcher {
    Literal(String),
    Wildcard,
    Choice(Vec<AstPathSegment>),
}

// We use Display to convert our ast into regular expressions to use for matching.
//
// Bit of an odd use of Display, but being able to do `write!` & `format!` was just too
// useful.
impl std::fmt::Display for AstPathSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for matcher in &self.0 {
            write!(f, "{matcher}")?;
        }
        Ok(())
    }
}

impl std::fmt::Display for AstMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AstMatcher::Literal(literal) => write!(f, "{literal}")?,
            AstMatcher::Wildcard => write!(f, ".*?")?,
            AstMatcher::Choice(choices) => {
                for (i, choice) in choices.iter().enumerate() {
                    if i != 0 {
                        write!(f, "|")?;
                    }
                    write!(f, "({choice})")?;
                }
            }
        }
        Ok(())
    }
}

fn parse_field_lookup(input: &str) -> IResult<&str, AstFieldLookup, VerboseError<&str>> {
    separated_list1(tag("."), parse_segment)
        .map(|mut segments| {
            let field = segments.pop().expect("should always be at least one segment");
            let path = segments.drain(1..).collect();
            let starting_type = segments.pop();
            AstFieldLookup {
                starting_type,
                path,
                field,
            }
        })
        .parse(input)
}

fn parse_segment(input: &str) -> IResult<&str, AstPathSegment, VerboseError<&str>> {
    many1(alt((
        tag("*").and_then(success(AstMatcher::Wildcard)),
        parse_choice,
        parse_literal,
    )))
    .map(AstPathSegment)
    .parse(input)
}

fn parse_choice(input: &str) -> IResult<&str, AstMatcher, VerboseError<&str>> {
    delimited(
        tag("{"),
        separated_list1(tag(","), parse_segment).map(AstMatcher::Choice),
        tag("}"),
    )(input)
}

fn parse_literal(input: &str) -> IResult<&str, AstMatcher, VerboseError<&str>> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))
    .map(|literal: &str| AstMatcher::Literal(literal.to_string()))
    .parse(input)
}

#[cfg(test)]
mod tests {
    use nom::Finish;
    use serde_json::json;

    use super::*;

    fn parse(input: &str) -> serde_json::Value {
        let (
            remainder,
            AstFieldLookup {
                starting_type,
                path,
                field,
            },
        ) = parse_field_lookup(input).finish().unwrap();

        assert!(remainder.is_empty());
        let starting_type = starting_type.unwrap();

        json!({
            "starting_type": format!("{starting_type}"),
            "path":
                path
                .into_iter()
                .map(|segment| format!("{segment}"))
                .collect::<Vec<_>>(),
            "field": format!("{field}")
        })
    }

    #[test]
    fn test_type_path_parsing() {
        insta::assert_json_snapshot!(parse("Query.whatever"), @r###"
        {
          "field": "whatever",
          "path": [],
          "starting_type": "Query"
        }
        "###);
        insta::assert_json_snapshot!(parse("Query.{customer,invoice}"), @r###"
        {
          "field": "(customer)|(invoice)",
          "path": [],
          "starting_type": "Query"
        }
        "###);
        insta::assert_json_snapshot!(parse("Query.*.{customer,invoice}"), @r###"
        {
          "field": "(customer)|(invoice)",
          "path": [
            ".*?"
          ],
          "starting_type": "Query"
        }
        "###);
        insta::assert_json_snapshot!(parse("Query.*.{customer,invoice}"), @r###"
        {
          "field": "(customer)|(invoice)",
          "path": [
            ".*?"
          ],
          "starting_type": "Query"
        }
        "###);
        insta::assert_json_snapshot!(parse("Query.customer*"), @r###"
        {
          "field": "customer.*?",
          "path": [],
          "starting_type": "Query"
        }
        "###);
        insta::assert_json_snapshot!(parse("Query.*customer"), @r###"
        {
          "field": ".*?customer",
          "path": [],
          "starting_type": "Query"
        }
        "###);
        insta::assert_json_snapshot!(parse("Query.{custom*,inv*}"), @r###"
        {
          "field": "(custom.*?)|(inv.*?)",
          "path": [],
          "starting_type": "Query"
        }
        "###);
    }
}
