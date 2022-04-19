//! ### What it does
//!
//! Check the types for fields are valid.
//! Fields valid are: Primitives, Basic Type.
//!
//! ### Why?
//!
//! To avoid having an invalid schema
use super::model_directive::MODEL_DIRECTIVE;
use super::visitor::{Visitor, VisitorContext};
use crate::utils::is_type_primitive;
use crate::utils::to_base_type_str;

pub struct CheckTypeValidity;

impl<'a> Visitor<'a> for CheckTypeValidity {
    fn enter_field(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        field: &'a async_graphql::Positioned<async_graphql_parser::types::FieldDefinition>,
        _parent_type: &'a async_graphql::Positioned<async_graphql_parser::types::TypeDefinition>,
    ) {
        let base_type = to_base_type_str(&field.node.ty.node.base);
        if is_type_primitive(&field.node) {
            return;
        }

        match ctx.types.get(&base_type) {
            Some(ty) => {
                if ty.node.directives.iter().any(|d| d.node.name.node == MODEL_DIRECTIVE) {
                    ctx.report_error(
                        vec![field.pos],
                        format!(
                            "Field `{name}` got an undefined type: `{ty}`.",
                            name = field.node.name.node,
                            ty = base_type
                        ),
                    );
                }
            }
            None => {
                ctx.report_error(
                    vec![field.pos],
                    format!(
                        "Field `{name}` got an undefined type: `{ty}`.",
                        name = field.node.name.node,
                        ty = base_type
                    ),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::rules::check_type_validity::CheckTypeValidity;
    use crate::rules::visitor::{visit, VisitorContext};
    use async_graphql_parser::parse_schema;
    use serde_json as _;

    #[test]
    fn should_not_error_with_basic_type() {
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
        visit(&mut CheckTypeValidity, &mut ctx, &schema);

        assert!(ctx.errors.is_empty(), "should be empty");
    }

    #[test]
    fn should_not_error_with_custom_type() {
        let schema = r#"
            type Truc {
                name: String!
            }

            type Product {
                id: ID!
                _name: String!
                """
                The product's price in $
                """
                __price: Truc!
            }
            "#;

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new(&schema);
        visit(&mut CheckTypeValidity, &mut ctx, &schema);

        assert!(ctx.errors.is_empty(), "should be empty");
    }

    #[test]
    fn should_error_with_model_type() {
        let schema = r#"
            type Truc @model {
                name: String!
            }

            type Product {
                id: ID!
                _name: String!
                """
                The product's price in $
                """
                __price: Truc!
            }
            "#;

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new(&schema);
        visit(&mut CheckTypeValidity, &mut ctx, &schema);

        assert!(!ctx.errors.is_empty(), "shouldn't be empty");
        assert_eq!(ctx.errors.len(), 1, "should have one error");
        assert_eq!(
            ctx.errors.get(0).unwrap().message,
            "Field `__price` got a modelized type: `Truc` but modelized types are not valid as field yet.",
            "should match"
        );
    }

    #[test]
    fn should_error_with_invalid_types() {
        let schema = r#"
            type Product @model {
                id: ID!
                _name: String!
                """
                The product's price in $
                """
                __price: Url!
            }
            "#;

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new(&schema);
        visit(&mut CheckTypeValidity, &mut ctx, &schema);

        assert!(!ctx.errors.is_empty(), "shouldn't be empty");
        assert_eq!(ctx.errors.len(), 1, "should have one error");
        assert_eq!(
            ctx.errors.get(0).unwrap().message,
            "Field `__price` got an undefined type: `Url`.",
            "should match"
        );
    }
}
