//! ### What it does
//!
//! The user defined scalars can be hydrated to the generated API only if those scalars belongs to
//! the list of PossibleScalar from engine for now.
//!
use engine::{Positioned, Value};
use engine_parser::types::TypeDefinition;
use engine_scalars::{DynamicScalar, PossibleScalar};

use super::visitor::{Visitor, VisitorContext};

pub struct ScalarHydratation;

const SPECIFIED_BY_DIRECTIVE: &str = "specifiedBy";
const SPECIFIED_BY_ARGUMENT_URL: &str = "url";

impl<'a> Visitor<'a> for ScalarHydratation {
    fn enter_scalar_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        type_definition: &'a Positioned<TypeDefinition>,
    ) {
        let name = type_definition.node.name.node.as_str().to_string();

        if PossibleScalar::test_scalar_name_recursive(name.as_str()) {
            ctx.registry.get_mut().create_type(
                |_| {
                    let specified_by_url = type_definition
                        .node
                        .directives
                        .iter()
                        .find(|directive| directive.node.name.node.as_str() == SPECIFIED_BY_DIRECTIVE)
                        .and_then(|directive| directive.node.get_argument(SPECIFIED_BY_ARGUMENT_URL))
                        .and_then(|x| match &x.node {
                            Value::String(s) => Some(s.clone()),
                            _ => None,
                        });

                    engine::registry::MetaType::Scalar(engine::registry::ScalarType {
                        name: name.clone(),
                        description: type_definition
                            .node
                            .description
                            .clone()
                            .map(|x| x.node.as_str().to_string()),
                        is_valid: None,
                        specified_by_url,
                        parser: engine::registry::ScalarParser::BestEffort,
                    })
                },
                name.as_str(),
                name.as_str(),
            );
        } else {
            ctx.report_error(vec![type_definition.pos], format!("\"{name}\" is not a proper scalar"));
        }
    }
}

#[cfg(test)]
mod tests {
    use engine_parser::parse_schema;
    use serde_json as _;

    use super::ScalarHydratation;
    use crate::rules::visitor::{visit, VisitorContext};

    #[test]
    fn should_error_when_defining_a_invalid_scalar() {
        let schema = r"
            scalar DateInvalid

            type Product @model {
                id: ID!
                test: DateInvalid!
            }
            ";

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new_for_tests(&schema);
        visit(&mut ScalarHydratation, &mut ctx, &schema);

        assert!(!ctx.errors.is_empty(), "shouldn't be empty");
        assert_eq!(ctx.errors.len(), 1, "should have one error");
        assert_eq!(
            ctx.errors.first().unwrap().message,
            "\"DateInvalid\" is not a proper scalar",
            "should match"
        );
    }

    #[test]
    fn should_work_with_a_valid_scalar() {
        let schema = r"
            scalar DateTime

            type Product @model {
                id: ID!
                test: DateTime!
            }
            ";

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new_for_tests(&schema);
        visit(&mut ScalarHydratation, &mut ctx, &schema);

        let scalar_ty = ctx.registry.get_mut().types.get("DateTime");

        assert!(ctx.errors.is_empty(), "should be empty");
        assert!(scalar_ty.is_some(), "should have the scalar definition");
        insta::assert_debug_snapshot!(scalar_ty.unwrap());
    }
}
