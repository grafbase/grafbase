use std::collections::HashMap;

use super::visitor::{Visitor, VisitorContext};
use dynaql::Positioned;
use dynaql_parser::types::{FieldDefinition, TypeDefinition};
use dynaql_value::ConstValue;

pub const LENGTH_DIRECTIVE: &str = "length";

pub const MIN_ARGUMENT: &str = "min";
pub const MAX_ARGUMENT: &str = "max";

pub struct LengthDirective;

impl<'a> Visitor<'a> for LengthDirective {
    fn directives(&self) -> String {
        r#"
        directive @length(min: Int, max: Int) on FIELD_DEFINITION
        "#
        .to_string()
    }

    fn enter_field(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        field: &'a Positioned<FieldDefinition>,
        _parent_type: &'a Positioned<TypeDefinition>,
    ) {
        if let Some(directive) = field
            .node
            .directives
            .iter()
            .find(|d| d.node.name.node == super::length_directive::LENGTH_DIRECTIVE)
        {
            use itertools::Itertools;

            if !crate::utils::is_type_with_length(&field.node.ty.node) {
                return ctx.report_error(
                    vec![directive.pos],
                    "The @length directive is only accepted on Strings and Lists",
                );
            }

            // Extract and group args
            let arguments: HashMap<_, _> = directive
                .node
                .arguments
                .iter()
                .into_group_map_by(move |(key, _)| key.node.as_str().to_string())
                .into_iter()
                .map(move |(key, values)| {
                    (
                        key,
                        values.into_iter().map(|val| val.1.node.clone()).collect::<Vec<_>>(),
                    )
                })
                .collect();

            if arguments.is_empty() {
                ctx.report_error(
                    vec![directive.pos],
                    format!("The @length directive expects at least one of the `{MIN_ARGUMENT}` and `{MAX_ARGUMENT}` arguments"),
                );
            }

            let mut deduplicated_arguments: HashMap<_, _> = arguments
                .into_iter()
                .map(|(key, mut values)| {
                    if values.len() > 1 {
                        ctx.report_error(
                            vec![directive.pos],
                            "The @length directive expects the `key` argument only once".to_string(),
                        );
                    }
                    (key, values.pop())
                })
                .collect();

            let min_value = deduplicated_arguments.remove(MIN_ARGUMENT).flatten();
            let max_value = deduplicated_arguments.remove(MAX_ARGUMENT).flatten();

            for (key, _) in deduplicated_arguments {
                ctx.report_error(
                    vec![directive.pos],
                    format!("Unexpected argument {key}, @length directive expects at most 2 arguments; `{MIN_ARGUMENT}` and `{MAX_ARGUMENT}`"),
                );
            }

            // Parse the successfully extracted args
            let mut value_as_number = |key, value: Option<_>| {
                value.as_ref().and_then(|value| {
                    match value {
                        ConstValue::Number(ref min) => Some(min.as_u64().unwrap(/* Infallible */)),
                        _ => None,
                    }
                    .or_else(|| {
                        ctx.report_error(
                            vec![directive.pos],
                            format!("The @length directive's {key} argument must be a positive number"),
                        );
                        None
                    })
                })
            };

            let min_value = value_as_number(MIN_ARGUMENT, min_value);
            let max_value = value_as_number(MAX_ARGUMENT, max_value);
            if let Some((min_value, max_value)) = min_value.zip(max_value) {
                if min_value > max_value {
                    ctx.report_error(
                        vec![directive.pos],
                        format!("The `{MAX_ARGUMENT}` must be greater than the `{MIN_ARGUMENT}`"),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::visitor::visit;
    use dynaql_parser::parse_schema;
    use pretty_assertions::assert_eq;

    #[rstest::rstest]
    #[case(r#"
            type Product @model {
                id: ID!
                name: String @length(foo: 10)
            }
            "#, 1, &[
            "Unexpected argument foo, @length directive expects at most 2 arguments; `min` and `max`"
    ])]
    #[case(r#"
            type Product @model {
                id: ID!
                name: String @length
            }
            "#, 1, &[
            "The @length directive expects at least one of the `min` and `max` arguments"
    ])]
    #[case(r#"
            type Product @model {
                id: ID! @length(min: 0, max: 100)
                name: String
            }
            "#, 1, &[
            "The @length directive is only accepted on Strings and Lists"
    ])]
    #[case(r#"
            type Product @model {
                id: ID!
                name: String!
                category: Int @length(min:0)
            }
            "#, 1, &[
            "The @length directive is only accepted on Strings and Lists"
    ])]
    #[case(r#"
            type Category @model {
                id: ID!
                name: String!
            }

            type Product @model {
                id: ID!
                name: String!
                category: Category @length(min: 0)
            }
            "#, 1, &[
            "The @length directive is only accepted on Strings and Lists"
    ])]
    #[case(r#"
            type Product @model {
                id: ID!
                name: String! @length(min: "10")
            }
            "#, 1, &[
            "The @length directive's min argument must be a positive number"
    ])]
    #[case(r#"
            type Product @model {
                id: ID!
                name: String! @length(value: 10)
            }
            "#, 1, &[
            "Unexpected argument value, @length directive expects at most 2 arguments; `min` and `max`"
    ])]
    #[case(r#"
            type Product @model {
                id: ID!
                name: String! @length(min: 0, value: 10)
            }
            "#, 1, &[
            "Unexpected argument value, @length directive expects at most 2 arguments; `min` and `max`"
    ])]
    #[case(r#"
            type Product @model {
                id: ID!
                name: String! @length(min:10, max: 1)
            }
            "#, 1,
            &["The `max` must be greater than the `min`"])]
    #[case(r#"
            type Product @model {
                id: ID!
                name: String! @length(min: 10, max: 100)
            }
            "#, 0, &[])]
    #[case(r#"
            type Product @model {
                id: ID!
                name: String! @length(min: 10)
            }
            "#, 0, &[])]
    #[case(r#"
            type Product @model {
                id: ID!
                name: String! @length(max: 10)
            }
            "#, 0, &[ ])]

    fn test_parse_result(#[case] schema: &str, #[case] error_count: usize, #[case] error_messages: &[&str]) {
        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut LengthDirective, &mut ctx, &schema);

        assert_eq!(ctx.errors.len(), error_count);

        assert_eq!(
            ctx.errors.len(),
            error_messages.len(),
            "Did you forget an error_message example case?"
        );
        for (error, expected) in ctx.errors.iter().zip(error_messages) {
            assert_eq!(&&error.message, expected);
        }
    }
}
