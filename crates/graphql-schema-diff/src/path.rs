//! A structured path to a specific location in a schema. See the docs on [Path].

mod display;
mod parse;

use std::fmt;

type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
pub struct ParseError;

/// A structured path to a specific location in a schema.
///
/// Paths have a structured string representation.
///
/// Each level in a path is separated by a '.' character. Directive uses, type definition extensions and schema definition extensions have an index to distinguish between them. The index is optional.
///
/// First level:
///
/// - Type definitions are unprefixed.
/// - Type extensions have an index, for example `Query[3]`.
/// - Directive definitions are prefixed with an `@`: `@authorized`.
/// - `:schema` for schema definitions, `:schema[1]` (index) for extensions.
///
/// Second level:
///
/// - Fields, union members, enum values and input object fields are unprefixed.
/// - Directives on types and schema definitions are prefixed with an `@` and followed by an index: `@key[0]`.
/// - Interface implementations are prefixed with an `&`: `&SomeInterface`.
///
/// Third level:
///
/// - Field arguments are unprefixed.
/// - Directives on fields and enum values are prefixed with an `@` and followed by an index: `@include[0]`.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Path<'a> {
    SchemaDefinition,
    SchemaExtension(usize),
    TypeDefinition(&'a str, Option<PathInType<'a>>),
    TypeExtension(&'a str, usize, Option<PathInType<'a>>),
    DirectiveDefinition(&'a str),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum PathInType<'a> {
    InField(&'a str, Option<PathInField<'a>>),
    InDirective(&'a str, usize),
    InterfaceImplementation(&'a str),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum PathInField<'a> {
    InArgument(&'a str),
    InDirective(&'a str, usize),
}

fn is_valid_graphql_name(s: &str) -> bool {
    let mut chars = s.chars();

    let Some(first_char) = chars.next() else {
        return false;
    };

    if !first_char.is_ascii_alphabetic() && first_char != '_' {
        return false;
    }

    for c in chars {
        if !c.is_ascii_alphanumeric() && c != '_' {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic)]

    use super::*;

    #[test]
    fn error_tests() {
        fn expect_error(path: &str) {
            match Path::parse(path) {
                Err(_) => (),
                Ok(found) => panic!("Expected error for path: {path}, got: {found:?}"),
            }
        }

        for case in [
            "",
            "s:",
            "s:meow",
            ":schema[-1]",
            ":schema.something",
            ":s",
            ":s[1]",
            ":something",
            "t:something",
            "something.:s",
            "10",
            "test.",
            "test[10].",
            "test.@siblings.",
            // Directive applications without index.
            "myObject._abc1.@requires",
            "_my_object.@key",
            // Empty directive name
            "@",
            "_my_object.@",
            // Index on a directive definition
            "@test[0]",
            "myObject.&MyInterface.a",
            "myObject.&MyInterface.",
        ] {
            expect_error(case);
        }
    }

    #[test]
    fn roundtrip_tests() {
        fn test(path: &str) {
            let Ok(parsed) = Path::parse(path) else {
                panic!("Failed to parse path: {path}")
            };
            let formatted = parsed.to_string();
            assert_eq!(path, formatted);
        }

        for case in [
            ":schema",
            ":schema[0]",
            ":schema[1]",
            ":schema[100]",
            "@meow",
            "@deprecated",
            "@something__else",
            "@join__type",
            "my_union",
            "__my_input_object[32]",
            "_my_object.id",
            "_my_object.@key[0]",
            "myObject[10].id",
            "myObject.@join__type[0]",
            "myObject._abc1",
            "myObject._abc1.@requires[0]",
            "myObject._abc1.@requires[100]",
            "myObject._abc1.arg1",
            "myObject.&MyInterface",
        ] {
            test(case);
        }
    }
}
