//! ### What it does
//!
//! Check that fields are not reserved for Modelized types
//!
//! ### Why?
//!
//! We do not want Modelized type with created_at and updated_at fields as we add them
//! ourselves.
use super::model_directive::MODEL_DIRECTIVE;
use super::visitor::{Visitor, VisitorContext};
use if_chain::if_chain;

pub struct CheckModelizedFieldReserved;

const RESERVED_FIELDS: [&str; 4] = ["created_at", "updated_at", "createdAt", "updatedAt"];

impl<'a> Visitor<'a> for CheckModelizedFieldReserved {
    fn directives(&self) -> String {
        String::new()
    }

    fn enter_field(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        field: &'a dynaql::Positioned<dynaql_parser::types::FieldDefinition>,
        parent: &'a dynaql::Positioned<dynaql_parser::types::TypeDefinition>,
    ) {
        if_chain! {
            let name = &field.node.name.node;
            if RESERVED_FIELDS.contains(&name.as_str());
            if parent.node.directives.iter().any(|directive| directive.node.name.node == MODEL_DIRECTIVE);
            then {
                ctx.report_error(
                    vec![field.pos],
                    format!("Field {name} is reserved, you can't use it.", name = name),
                );

            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CheckModelizedFieldReserved;
    use crate::rules::visitor::{visit, VisitorContext};
    use dynaql_parser::parse_schema;
    use serde_json as _;

    #[test]
    fn should_error_on_reserved_keyword() {
        let schema = r#"
            type Product @model {
                id: ID!
                _name: String!
                """
                The product's price in $
                """
                __price: Int!
                created_at: DateTime!
            }
            "#;

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new(&schema);
        visit(&mut CheckModelizedFieldReserved, &mut ctx, &schema);

        assert!(!ctx.errors.is_empty(), "shouldn't be empty");
        assert_eq!(ctx.errors.len(), 1, "should have one error");
        assert_eq!(
            ctx.errors.get(0).unwrap().message,
            "Field created_at is reserved, you can't use it.",
            "should match"
        );
    }

    #[test]
    fn should_allow_reserved_keyword_on_non_model() {
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
        visit(&mut CheckModelizedFieldReserved, &mut ctx, &schema);

        assert!(ctx.errors.is_empty(), "should not have any error");
    }
}
