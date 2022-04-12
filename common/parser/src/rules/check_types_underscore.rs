//! ### What it does
//!
//! Check than the types inputed by the user doesn't begin by an underscore.
//!
//! ### Why?
//!
//! We keep those types as internal types.

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
    ) {
        if_chain! {
            let name = &field.node.name.node;
            if name.starts_with('_');
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
            type Product {
                id: ID!
                name: String!
                """
                The product's price in $
                """
                _price: Int!
            }
            "#;

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new(&schema);
        visit(&mut CheckBeginWithUnderscore, &mut ctx, &schema);

        assert!(!ctx.errors.is_empty(), "should be empty");
        assert_eq!(
            ctx.errors.get(0).unwrap().message,
            "Field _price shouldn't start with an underscore.",
            "should be empty"
        );
    }
}
