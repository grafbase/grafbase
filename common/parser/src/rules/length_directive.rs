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

            let arguments: Vec<_> = directive
                .node
                .arguments
                .iter()
                .map(|(key, value)| (key.node.as_str(), value))
                .collect();

            let allowed_args = [MIN_ARGUMENT, MAX_ARGUMENT];

            let argument_names: Vec<_> = arguments
                .iter()
                .map(|(key, _)| key)
                .sorted()
                .dedup_with_count()
                .collect();
            let parsed_args = match &argument_names[..] {
                // One of each arg
                arg_names @ (&[(1, &MAX_ARGUMENT), (1, &MIN_ARGUMENT)] | [(1, &MIN_ARGUMENT | &MAX_ARGUMENT)] ) => {
                    Some(arg_names.iter().map(|(_,  arg_name)|{
                        (arg_name, arguments.iter().find(|(key, _)| &key == arg_name).map(|(_, value)| &value.node))
                    }).collect::<Vec<_>>())
                },
                &[] => {
                    ctx.report_error(
                        vec![directive.pos],
                        format!("The @length directive expects at least one of the `{MIN_ARGUMENT}` and `{MAX_ARGUMENT}` arguments"),
                    );
                    None
                }
                &[(_, &MAX_ARGUMENT), (_, &MIN_ARGUMENT)] => {
                    ctx.report_error(
                        vec![directive.pos],
                        format!("The @length directive expects the `{MIN_ARGUMENT}` and `{MAX_ARGUMENT}` arguments only once each"),
                    );
                    None
                },
                s => {
                    for (_, key) in s {
                        if !allowed_args.contains(key) {
                            ctx.report_error(
                                vec![directive.pos],
                                format!("Unexpected argument {key}, @length directive expects at most 2 arguments; `{MIN_ARGUMENT}` and `{MAX_ARGUMENT}`"),
                            );
                        }
                    }
                    None
                }
            }.map(|parsed_args| {
                parsed_args.into_iter().filter_map(|(key, value)|{
                    if let Some(ConstValue::Number(ref min)) = value {
                        min.as_u64().map(u64::try_from)
                    } else {
                        None
                    }.or_else (|| {
                        ctx.report_error(
                            vec![directive.pos],
                            format!("The @length directive's {key} argument must be a positive number")
                        );
                        None
                    })
                }).collect::<Result<Vec<_>, _>>()
            });
            match parsed_args.as_ref().map(|inner| inner.as_deref()) {
                Some(Ok(&[max, min])) => {
                    if max <= min {
                        ctx.report_error(
                            vec![directive.pos],
                            format!("The `{MAX_ARGUMENT}` must be greater than the `{MIN_ARGUMENT}`"),
                        );
                    }
                }
                Some(Err(e)) => {
                    ctx.report_error(
                        vec![directive.pos],
                        format!("Error {e} while parsing @length directive"),
                    );
                }
                Some(Ok(_)) | None => {
                    // All Good
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

    fn test_parse_result(
        #[case] schema: &str,
        #[case] error_count: usize,
        #[case] error_messages: &[&str],
    ) {
        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut LengthDirective, &mut ctx, &schema);

        assert_eq!(ctx.errors.len(), error_count);
        for (error, expected) in ctx.errors.iter().zip(error_messages) {
            assert_eq!(
                &&error.message,
                expected
            )
        }
    }
}
