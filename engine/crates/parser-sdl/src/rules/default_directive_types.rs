use engine::Positioned;
use engine_parser::types::{FieldDefinition, TypeDefinition};

use super::{
    model_directive::ModelDirective,
    visitor::{Visitor, VisitorContext},
};

pub const VALUE_ARGUMENT: &str = "value";

pub struct DefaultDirectiveTypes;

const FIELDS_NOT_ALLOWED: &[&str] = &[engine::names::OUTPUT_FIELD_ID];

impl<'a> Visitor<'a> for DefaultDirectiveTypes {
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
            .find(|d| d.node.name.node == super::default_directive::DEFAULT_DIRECTIVE)
        {
            if ModelDirective::is_model(ctx, &field.node.ty.node) {
                ctx.report_error(
                    vec![directive.pos],
                    "The @default directive is not accepted on fields referring to other models".to_string(),
                );
            }

            if let Some(field) = FIELDS_NOT_ALLOWED
                .iter()
                .copied()
                .find(|field_name| field.node.name.node == *field_name)
            {
                ctx.report_error(
                    vec![directive.pos],
                    format!("The @default directive is not accepted on the `{field}` field"),
                );
            }

            if let Ok(mut arguments) = super::directive::extract_arguments(ctx, directive, &[&[VALUE_ARGUMENT]], None) {
                let _default_value = arguments.remove(VALUE_ARGUMENT).unwrap();

                // let error = {
                //     let ctx_registry = ctx.registry.borrow();
                //     engine::validation::utils::is_valid_input_value(
                //         &ctx_registry,
                //         &field.node.ty.node.to_string(),
                //         &default_value,
                //         QueryPath::empty().child(field.node.name.node.as_str()),
                //     )
                // };
                // if let Some(err) = error {
                //     ctx.report_error(
                //         vec![directive.pos],
                //         format!("The @default value is of a wrong type: {err}"),
                //     );
                // }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use engine_parser::parse_schema;
    use engine_scalars::{PossibleScalar, SDLDefinitionScalar};
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::rules::visitor::visit;

    #[test]
    fn test_default_with_enum_variant() {
        let schema = r"
            type Product {
                id: ID!
                price: Int! @default(value: 0)
                currency: Currency @default(value: USD)
            }

            enum Currency {
                EUR
                USD
                GBP
            }
        ";

        let mut rules = crate::rules::visitor::VisitorNil
            .with(crate::BasicType)
            .with(crate::EnumType)
            .with(crate::ScalarHydratation);

        let schema = format!("{}\n{schema}", PossibleScalar::sdl());
        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new_for_tests(&schema);

        visit(&mut rules, &mut ctx, &schema);
        visit(&mut crate::DefaultDirectiveTypes, &mut ctx, &schema);

        assert_eq!(ctx.errors, vec![]);
    }

    #[rstest::rstest]
    #[case(r#"
        type Product @model {
            id: ID!
            name: String @default(foo: "default")
        }
    "#, &[
        "The @default directive takes a single `value` argument"
    ])]
    #[case(r"
        type Product @model {
            id: ID!
            name: String @default
        }
    ", &[
        "The @default directive takes a single `value` argument"
    ])]
    #[case(r#"
        type Product @model {
            id: ID! @default(value: "default")
            name: String
        }
    "#, &[
        "The @default directive is not accepted on the `id` field"
    ])]
    #[case(r"
        type Category @model {
            id: ID!
            name: String!
        }

        type Product @model {
            id: ID!
            name: String!
            category: Category @default(value: null)
        }
    ", &[
        "The @default directive is not accepted on fields referring to other models"
    ])]
    #[case(r"
        type Product @model {
            id: ID!
            name: String! @default(value: 10)
        }
    ", &[
        "The @default value is of a wrong type: \"name\", expected type \"String\""
    ])]
    #[case(r#"
        type Product @model {
            id: ID!
            name: String @default(value: "foo")
        }
    "#, &[])]
    fn test_parse_result(#[case] schema: &str, #[case] expected_messages: &[&str]) {
        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new_for_tests(&schema);
        visit(&mut DefaultDirectiveTypes, &mut ctx, &schema);

        let actual_messages: Vec<_> = ctx.errors.iter().map(|error| error.message.as_str()).collect();
        assert_eq!(actual_messages.as_slice(), expected_messages);
    }
}
