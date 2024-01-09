//! Implement the model directive
//!
//! When a @model directive is present for a type, we generate the associated type into the
//! registry and generate the CRUDL configuration for this type.
//!
//! Flow:
//!  -> When there is a @model directive on a type
//!  -> Must be an ObjectType
//!  -> Must have primitives
//!  -> Must have a non_nullable ID type
//!
//! Then:
//!  -> Create the ObjectType
//!  -> Create the ReadById Query
//!  -> Create the Create Mutation
//!
//! TODO: Should have either: an ID or a PK

pub mod types;

use std::borrow::Cow;

use engine::Positioned;
use engine_parser::types::{BaseType, Type, TypeDefinition, TypeKind};
use if_chain::if_chain;

use super::{
    directive::Directive,
    visitor::{Visitor, VisitorContext},
};

pub struct ModelDirective;

pub const MODEL_DIRECTIVE: &str = "model";

impl ModelDirective {
    pub fn is_model(ctx: &'_ VisitorContext<'_>, ty: &Type) -> bool {
        Self::get_model_type_definition(ctx, &ty.base).is_some()
    }

    pub fn get_model_type_definition<'a, 'b>(
        ctx: &'a VisitorContext<'b>,
        base: &BaseType,
    ) -> Option<&'a Cow<'b, Positioned<TypeDefinition>>> {
        match base {
            BaseType::Named(name) => ctx.types.get(name.as_ref()).and_then(|ty| {
                if_chain!(
                    if let TypeKind::Object(_) = &ty.node.kind;
                    if ty.node.directives.iter().any(|directive| {
                        let has_no_attributes = directive.node.arguments.is_empty();
                        directive.is_model() && has_no_attributes
                    });
                    then { Some(ty) }
                    else { None }
                )
            }),
            BaseType::List(list) => Self::get_model_type_definition(ctx, &list.base),
        }
    }
}

impl Directive for ModelDirective {
    fn definition() -> String {
        r"
        directive @model on OBJECT
        "
        .to_string()
    }
}

impl<'a> Visitor<'a> for ModelDirective {
    fn enter_type_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        type_definition: &'a engine::Positioned<engine_parser::types::TypeDefinition>,
    ) {
        if !&type_definition
            .node
            .directives
            .iter()
            .filter(|directive| directive.is_model())
            .any(|directive| directive.node.arguments.is_empty())
        {
            return;
        }

        ctx.report_error(
            vec![type_definition.node.name.pos],
            "The connector-less `@model` directive is no longer supported.",
        );
    }
}

#[cfg(test)]
mod tests {
    use engine_parser::parse_schema;
    use serde_json as _;

    use super::ModelDirective;
    use crate::rules::visitor::{visit, VisitorContext};

    #[test]
    fn should_reject_model() {
        let schema = r"
            type Product @model {
                id: ID!
                test: String!
            }
            ";

        let schema = parse_schema(schema).expect("");

        let variables = Default::default();
        let mut ctx = VisitorContext::new(&schema, false, &variables);
        visit(&mut ModelDirective, &mut ctx, &schema);

        insta::assert_debug_snapshot!(ctx.errors, @r###"
        [
            RuleError {
                locations: [
                    Pos(2:18),
                ],
                message: "The connector-less `@model` directive is no longer supported.",
            },
        ]
        "###);
    }
}
