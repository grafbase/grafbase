//! ### What it does
//!
//! Check that fields are in camelcase
//!
//! ### Why?
//!
//! Due to the implementation right now (especially on constraint), we shouldn't
//! allow it

use case::CaseExt;
use if_chain::if_chain;

use super::visitor::{Visitor, VisitorContext};

pub struct CheckFieldCamelCase;

impl<'a> Visitor<'a> for CheckFieldCamelCase {
    fn enter_field(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        field: &'a engine::Positioned<engine_parser::types::FieldDefinition>,
        _parent: &'a engine::Positioned<engine_parser::types::TypeDefinition>,
    ) {
        if_chain! {
            let name = &field.node.name.node;
            let sanitized_name = name.to_camel_lowercase();
            if sanitized_name != *name;
            then {
                ctx.report_error(
                    vec![field.pos],
                    format!("Field \"{name}\" is not in Camel lowercase, please use \"{sanitized_name}\" instead."),
                );

            }
        }
    }
}

#[cfg(test)]
mod tests {
    use engine_parser::parse_schema;
    use serde_json as _;

    use super::CheckFieldCamelCase;
    use crate::rules::visitor::{visit, VisitorContext};

    #[test]
    fn should_error_when_not_camel_lowercase() {
        let schema = r"
            type Product @model {
                id: ID!
                _name: String!
            }
            ";

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new_for_tests(&schema);
        visit(&mut CheckFieldCamelCase, &mut ctx, &schema);

        assert!(!ctx.errors.is_empty(), "shouldn't be empty");
        assert_eq!(ctx.errors.len(), 1, "should have one error");
        assert_eq!(
            ctx.errors.first().unwrap().message,
            "Field \"_name\" is not in Camel lowercase, please use \"name\" instead.",
            "should match"
        );
    }
}
