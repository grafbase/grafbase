//! ### What it does
//!
//! Check if there are collision with Types
//!
//! ### Why?
//!
//! To avoid having an invalid schema
use std::collections::HashSet;

use engine::Positioned;
use engine_parser::types::TypeDefinition;

use super::visitor::{Visitor, VisitorContext};

#[derive(Default)]
pub struct CheckTypeCollision {
    /// Type we encounter in parsing the initial schema. If there is a collision for Object & Enum
    /// throw a Validation Error
    type_pokedex: HashSet<String>,
}

impl<'a> Visitor<'a> for CheckTypeCollision {
    fn enter_type_definition(&mut self, ctx: &mut VisitorContext<'a>, type_definition: &'a Positioned<TypeDefinition>) {
        let ty = &type_definition.node;
        let name = ty.name.node.to_string();

        if ty.extend {
            return;
        }

        if !self.type_pokedex.insert(name) {
            ctx.report_error(
                vec![type_definition.pos],
                format!("Type `{name}` is present multiple times.", name = &ty.name.node),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use engine_parser::parse_schema;
    use serde_json as _;

    use crate::rules::{
        check_type_collision::CheckTypeCollision,
        visitor::{visit, VisitorContext},
    };

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

            enum Product {
              PRODUCT_A
              PRODUCT_B
            }

            "#;

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new_for_tests(&schema);
        visit(&mut CheckTypeCollision::default(), &mut ctx, &schema);

        assert!(!ctx.errors.is_empty(), "shouldn't be empty");
        assert_eq!(ctx.errors.len(), 1, "should have one error");
        assert_eq!(
            ctx.errors.first().unwrap().message,
            "Type `Product` is present multiple times.",
            "should match"
        );
    }
}
