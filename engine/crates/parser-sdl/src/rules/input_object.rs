//! Pulls input objects into the registry
use engine::registry::{InputObjectType, MetaInputValue};
use engine_parser::types::TypeKind;

use super::visitor::{Visitor, VisitorContext};

pub struct InputObjectVisitor;

impl<'a> Visitor<'a> for InputObjectVisitor {
    fn enter_type_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        type_definition: &'a engine::Positioned<engine_parser::types::TypeDefinition>,
    ) {
        let TypeKind::InputObject(input_object) = &type_definition.node.kind else {
            return;
        };
        let type_name = type_definition.node.name.node.to_string();
        ctx.registry.get_mut().create_type(
            |_| {
                InputObjectType::new(
                    type_name.clone(),
                    input_object.fields.iter().map(|field| MetaInputValue {
                        description: field.node.description.clone().map(|description| description.node),
                        default_value: field.node.default_value.clone().map(|default| default.node),
                        ..MetaInputValue::new(field.node.name.node.to_string(), field.node.ty.node.to_string())
                    }),
                )
                .with_description(type_definition.node.description.clone().map(|x| x.node))
                .into()
            },
            &type_name,
            &type_name,
        );
    }
}

#[cfg(test)]
mod tests {
    use engine::registry::RegistrySdlExt;
    #[test]
    fn test_parsing_input_type() {
        let schema = r#"
            extend type Mutation {
                checkout(input: CheckoutInput!): CheckoutSession! @resolver(name: "checkout")
            }

            input CheckoutInput {
                price: String!
                quantity: Int = 1
            }

            type CheckoutSession {
                url: String!
            }
            "#;

        insta::assert_snapshot!(crate::parse_registry(schema).unwrap().export_sdl(false), @r###"
        input CheckoutInput {
        	price: String!
        	quantity: Int = 1
        }
        type CheckoutSession {
        	url: String!
        }
        type Mutation {
        	checkout(input: CheckoutInput!): CheckoutSession!
        }
        schema {
        	mutation: Mutation
        }
        "###);
    }
}
