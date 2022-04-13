//! ### What it does
//!
//! Check that types input by the user don't begin with double underscores
//! on @models only.
//!
//! ### Why?
//!
//! We keep those types as internal types.

use super::model_directive::MODEL_DIRECTIVE;
use super::visitor::{Visitor, VisitorContext};
use if_chain::if_chain;

pub struct CheckBeginWithUnderscore;

impl<'a> Visitor<'a> for CheckBeginWithUnderscore {
    fn directives(&self) -> String {
        String::new()
    }

    fn enter_field(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        field: &'a async_graphql::Positioned<async_graphql_parser::types::FieldDefinition>,
        parent: &'a async_graphql::Positioned<async_graphql_parser::types::TypeDefinition>,
    ) {
        if_chain! {
            let name = &field.node.name.node;
            if name.starts_with("__");
            if parent.node.directives.iter().any(|directive| directive.node.name.node == MODEL_DIRECTIVE);
            then {
                ctx.report_error(
                    vec![field.pos],
                    format!("Field {name} shouldn't start with an underscore.", name = name),
                );

            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::rules::check_types_underscore::CheckBeginWithUnderscore;
    use crate::rules::visitor::{visit, VisitorContext};
    use async_graphql_parser::parse_schema;
    use serde_json as _;

    #[test]
    fn should_error_on_underscore() {
        let schema = r#"
            type Product @model {
                id: ID!
                _name: String!
                """
                The product's price in $
                """
                __price: Int!
            }
            "#;

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new(&schema);
        visit(&mut CheckBeginWithUnderscore, &mut ctx, &schema);

        assert!(!ctx.errors.is_empty(), "shouldn't be empty");
        assert_eq!(ctx.errors.len(), 1, "should have one error");
        assert_eq!(
            ctx.errors.get(0).unwrap().message,
            "Field __price shouldn't start with an underscore.",
            "should match"
        );
    }

    #[test]
    fn should_allow_underscore_on_non_model() {
        let schema = r#"
            type Product {
                id: ID!
                _name: String!
                """
                The product's price in $
                """
                __price: Int!
            }
            "#;

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new(&schema);
        visit(&mut CheckBeginWithUnderscore, &mut ctx, &schema);

        assert!(ctx.errors.is_empty(), "should not have any error");
    }
}
