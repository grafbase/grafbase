use engine::Positioned;
use engine_parser::types::{FieldDefinition, TypeDefinition};
use engine_value::ConstValue;

use super::{
    directive::Directive,
    visitor::{Visitor, VisitorContext},
};

pub const LENGTH_DIRECTIVE: &str = "length";

pub const MIN_ARGUMENT: &str = "min";
pub const MAX_ARGUMENT: &str = "max";

pub struct LengthDirective;

impl Directive for LengthDirective {
    fn definition() -> String {
        r"
        directive @length(min: Int, max: Int) on FIELD_DEFINITION
        "
        .to_string()
    }
}

impl<'a> Visitor<'a> for LengthDirective {
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
            if !crate::utils::is_type_with_length(&field.node.ty.node) {
                return ctx.report_error(
                    vec![directive.pos],
                    "The @length directive is only accepted on Strings and Lists",
                );
            }

            if let Ok(mut arguments) = super::directive::extract_arguments(
                ctx,
                directive,
                &[&[MIN_ARGUMENT], &[MAX_ARGUMENT], &[MAX_ARGUMENT, MIN_ARGUMENT]],
                Some("`max` and `min`"),
            ) {
                let min_value = arguments.remove(MIN_ARGUMENT);
                let max_value = arguments.remove(MAX_ARGUMENT);

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
}

#[cfg(test)]
mod tests {
    use engine_parser::parse_schema;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::rules::visitor::visit;

    #[rstest::rstest]
    #[case(r"
        type Product @model {
            id: ID!
            name: String @length(foo: 10)
        }
        ", &[
        "Unexpected argument foo, @length directive only supports the following arguments: `max` and `min`"
    ])]
    #[case(r"
        type Product @model {
            id: ID!
            name: String @length
        }
        ", &[
        "The @length directive expects at least one of the `max` and `min` arguments"
    ])]
    #[case(r"
        type Product @model {
            id: ID! @length(min: 0, max: 100)
            name: String
        }
        ", &[
        "The @length directive is only accepted on Strings and Lists"
    ])]
    #[case(r"
        type Product @model {
            id: ID!
            name: String!
            category: Int @length(min:0)
        }
        ", &[
        "The @length directive is only accepted on Strings and Lists"
    ])]
    #[case(r"
        type Category @model {
            id: ID!
            name: String!
        }

        type Product @model {
            id: ID!
            name: String!
            category: Category @length(min: 0)
        }
        ", &[
        "The @length directive is only accepted on Strings and Lists"
    ])]
    #[case(r#"
        type Product @model {
            id: ID!
            name: String! @length(min: "10")
        }
        "#, &[
        "The @length directive's min argument must be a positive number"
    ])]
    #[case(r"
        type Product @model {
            id: ID!
            name: String! @length(value: 10)
        }
        ", &[
        "Unexpected argument value, @length directive only supports the following arguments: `max` and `min`"
    ])]
    #[case(r"
        type Product @model {
            id: ID!
            name: String! @length(min: 0, value: 10)
        }
        ", &[
        "Unexpected argument value, @length directive only supports the following arguments: `max` and `min`"
    ])]
    #[case(r"
        type Product @model {
            id: ID!
            name: String! @length(min:10, max: 1)
        }
        ",
        &["The `max` must be greater than the `min`"]
    )]
    #[case(r"
        type Product @model {
            id: ID!
            name: String! @length(min: 10, max: 100)
        }
        ", &[]
    )]
    #[case(r"
        type Product @model {
            id: ID!
            name: String! @length(min: 10)
        }
        ", &[]
    )]
    #[case(r"
        type Product @model {
            id: ID!
            name: String! @length(max: 10)
        }
        ", &[]
    )]

    fn test_parse_result(#[case] schema_string: &str, #[case] expected_messages: &[&str]) {
        let schema = parse_schema(schema_string).unwrap();
        let mut ctx = VisitorContext::new_for_tests(&schema);
        visit(&mut LengthDirective, &mut ctx, &schema);

        let actual_messages: Vec<_> = ctx.errors.iter().map(|error| error.message.as_str()).collect();
        assert_eq!(actual_messages.as_slice(), expected_messages, "for {schema_string}");
    }
}
