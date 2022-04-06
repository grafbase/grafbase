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

use crate::registry::add_create_mutation;
use crate::registry::add_remove_query;
use crate::utils::is_id_type_and_non_nullable;
use crate::utils::is_type_primitive;

use super::visitor::{Visitor, VisitorContext};
use async_graphql::indexmap::IndexMap;
use async_graphql::registry::resolvers::dynamo_querying::DynamoResolver;
use async_graphql::registry::MetaField;
use async_graphql::registry::MetaInputValue;
use async_graphql::registry::MetaType;
use async_graphql::registry::{
    resolvers::Resolver, resolvers::ResolverType, transformers::Transformer, variables::VariableResolveDefinition,
};
use async_graphql_parser::types::TypeKind;
use if_chain::if_chain;

pub struct ModelDirective;

const MODEL_DIRECTIVE: &str = "model";

impl<'a> Visitor<'a> for ModelDirective {
    fn directives(&self) -> String {
        r#"
        directive @model on OBJECT
        "#
        .to_string()
    }

    fn enter_type_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        type_definition: &'a async_graphql::Positioned<async_graphql_parser::types::TypeDefinition>,
    ) {
        let directives = &type_definition.node.directives;
        if_chain! {
            if directives.iter().find(|directive| directive.node.name.node == MODEL_DIRECTIVE).is_some();
            if let TypeKind::Object(object) = &type_definition.node.kind;
            if object.fields.iter().find(|x| !is_type_primitive(&x.node)).is_none();
            if let Some(id_field) = object.fields.iter().find(|x| is_id_type_and_non_nullable(&x.node));
            then {
                let type_name = type_definition.node.name.node.to_string();
                // If it's a modeled Type, we create the associated type into the registry.
                // Without more data, we infer it's from our modelization.
                ctx.registry.get_mut().create_type(&mut |_| MetaType::Object {
                    name: type_name.clone(),
                    description: type_definition.node.description.clone().map(|x| x.node),
                    fields: {
                        let mut fields = IndexMap::new();
                        for field in &object.fields {
                            let name = field.node.name.node.to_string();
                            fields.insert(name.clone(), MetaField {
                                name: name.clone(),
                                description: field.node.description.clone().map(|x| x.node),
                                args: Default::default(),
                                ty: field.node.ty.clone().node.to_string(),
                                deprecation: Default::default(),
                                cache_control: Default::default(),
                                external: false,
                                requires: None,
                                provides: None,
                                visible: None,
                                compute_complexity: None,
                                resolve: None,
                                transforms: Some(vec![Transformer::DynamoSelect {
                                    property: if name == "id" {
                                        "pk".to_string()
                                    } else {
                                        name
                                    },
                                }]),
                            });
                        };
                        fields
                    },
                    cache_control: async_graphql::CacheControl {
                        public: true,
                        max_age: 0usize,
                    },
                    extends: false,
                    keys: None,
                    visible: None,
                    is_subscription: false,
                    rust_typename: type_name.clone(),
                }, &type_name, &type_name);

                ctx.queries.push(MetaField {
                    name: format!("unstable_{}byID", type_name.to_lowercase()),
                    description: Some(format!("Get a {} by his ID", type_name)),
                    args: {
                        let mut args = IndexMap::new();
                        args.insert(
                            "id".to_owned(),
                            MetaInputValue {
                                name: "id".to_owned(),
                                ty: "ID!".to_string(),
                                visible: None,
                                description: Some(format!("{} id", type_name)),
                                is_secret: false,
                                default_value: None,
                            },
                        );
                        args
                    },
                    ty: type_name.clone(),
                    deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                    cache_control: async_graphql::CacheControl {
                        public: true,
                        max_age: 0usize,
                    },
                    external: false,
                    provides: None,
                    requires: None,
                    visible: None,
                    compute_complexity: None,
                    resolve: Some(Resolver {
                        id: Some(format!("{}_resolver", type_name.to_lowercase())),
                        // TODO: Should be defined as a ResolveNode
                        r#type: ResolverType::DynamoResolver(DynamoResolver::QueryPKSK {
                            pk: VariableResolveDefinition::InputTypeName("id".to_owned()),
                            sk: VariableResolveDefinition::InputTypeName("id".to_owned()),
                        })
                    }),
                    transforms: None,
                });

                add_create_mutation(ctx, object, &id_field.node, &type_name);
                add_remove_query(ctx, &id_field.node, &type_name)
            }
        }
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    pub fn factory() -> DefaultValuesOfCorrectType {
        DefaultValuesOfCorrectType
    }

    #[test]
    fn variables_with_no_default_values() {
        expect_passes_rule!(
            factory,
            r#"
          query NullableValues($a: Int, $b: String, $c: ComplexInput) {
            dog { name }
          }
        "#,
        );
    }

    #[test]
    fn required_variables_without_default_values() {
        expect_passes_rule!(
            factory,
            r#"
          query RequiredValues($a: Int!, $b: String!) {
            dog { name }
          }
        "#,
        );
    }

    #[test]
    fn variables_with_valid_default_values() {
        expect_passes_rule!(
            factory,
            r#"
          query WithDefaultValues(
            $a: Int = 1,
            $b: String = "ok",
            $c: ComplexInput = { requiredField: true, intField: 3 }
          ) {
            dog { name }
          }
        "#,
        );
    }

    #[test]
    fn no_required_variables_with_default_values() {
        expect_fails_rule!(
            factory,
            r#"
          query UnreachableDefaultValues($a: Int! = 3, $b: String! = "default") {
            dog { name }
          }
        "#,
        );
    }

    #[test]
    fn variables_with_invalid_default_values() {
        expect_fails_rule!(
            factory,
            r#"
          query InvalidDefaultValues(
            $a: Int = "one",
            $b: String = 4,
            $c: ComplexInput = "notverycomplex"
          ) {
            dog { name }
          }
        "#,
        );
    }

    #[test]
    fn complex_variables_missing_required_field() {
        expect_fails_rule!(
            factory,
            r#"
          query MissingRequiredField($a: ComplexInput = {intField: 3}) {
            dog { name }
          }
        "#,
        );
    }

    #[test]
    fn list_variables_with_invalid_item() {
        expect_fails_rule!(
            factory,
            r#"
          query InvalidItem($a: [String] = ["one", 2]) {
            dog { name }
          }
        "#,
        );
    }
}
*/
