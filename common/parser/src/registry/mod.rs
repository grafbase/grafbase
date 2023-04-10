//! TODO:
//!
//! -> Split each of the creation and add tests with SDL
//!
use case::CaseExt;

use dynaql::indexmap::IndexMap;
use dynaql::registry::enums::DynaqlEnum;
use dynaql::registry::relations::MetaRelation;
use dynaql::registry::transformers::Transformer;
use dynaql::registry::{
    resolvers::dynamo_mutation::DynamoMutationResolver, resolvers::Resolver, resolvers::ResolverType,
    variables::VariableResolveDefinition, MetaField, MetaInputValue, MetaType,
};
use dynaql::registry::{MetaEnumValue, Registry};
use dynaql::validation::dynamic_validators::DynValidator;
use dynaql::AuthConfig;
use dynaql_parser::types::{BaseType, FieldDefinition, ObjectType, TypeDefinition};
use grafbase::auth::Operations;
use std::fmt::Display;

use crate::registry::names::MetaNames;
use crate::rules::length_directive::{LENGTH_DIRECTIVE, MAX_ARGUMENT, MIN_ARGUMENT};
use crate::rules::visitor::VisitorContext;
use crate::utils::{to_input_type, to_lower_camelcase};

mod mutations;
pub mod names;
mod pagination;
mod relations;
mod search;
pub use mutations::{add_mutation_create, add_mutation_update, NumericFieldKind};
pub use pagination::{add_query_paginated_collection, generate_pagination_args};
pub use search::add_query_search;

fn register_dynaql_enum<T: DynaqlEnum>(registry: &mut Registry) -> BaseType {
    let type_name = T::ty().to_string();
    registry.create_type(
        |_| MetaType::Enum {
            name: type_name.clone(),
            description: None,
            enum_values: IndexMap::from_iter(
                T::values()
                    .into_iter()
                    .map(|value| {
                        (
                            value.clone(),
                            MetaEnumValue {
                                name: value,
                                description: None,
                                deprecation: dynaql::registry::Deprecation::NoDeprecated,
                                visible: None,
                                value: None,
                            },
                        )
                    })
                    .collect::<Vec<_>>(),
            ),
            visible: None,
            rust_typename: type_name.clone(),
        },
        &type_name,
        &type_name,
    );
    BaseType::named(&type_name)
}

/// Create an input type for a non_primitive Type.
pub fn add_input_type_non_primitive(ctx: &mut VisitorContext<'_>, object: &ObjectType, type_name: &str) -> String {
    let type_name = type_name.to_string();
    let input_type = format!("{}Input", type_name.to_camel());

    // Input
    ctx.registry.get_mut().create_type(
        |_| MetaType::InputObject {
            name: input_type.clone(),
            description: Some(format!("{type_name} input type.")),
            oneof: false,
            input_fields: {
                let mut input_fields = IndexMap::new();
                for field in &object.fields {
                    let name = &field.node.name.node;

                    input_fields.insert(
                        name.clone().to_string(),
                        MetaInputValue {
                            description: field.node.description.clone().map(|x| x.node),
                            ..MetaInputValue::new(
                                name.to_string(),
                                to_input_type(&ctx.types, field.node.ty.clone().node),
                            )
                        },
                    );
                }
                input_fields
            },
            visible: None,
            rust_typename: input_type.clone(),
        },
        &input_type,
        &input_type,
    );

    input_type
}

/// Add the remove mutation for a given Object
pub fn add_remove_mutation(ctx: &mut VisitorContext<'_>, type_name: &str, auth: Option<&AuthConfig>) {
    let type_name = type_name.to_string();
    let delete_payload_name = format!("{}DeletePayload", type_name.to_camel());

    // DeletePayload
    ctx.registry.get_mut().create_type(
        |_| MetaType::Object {
            name: delete_payload_name.clone(),
            description: None,
            fields: {
                let mut fields = IndexMap::new();
                let name = "deletedId".to_string();
                fields.insert(
                    name.clone(),
                    MetaField {
                        name,
                        description: None,
                        args: Default::default(),
                        // TODO: Should be infered from the entity depending on the directives
                        ty: "ID!".to_string(),
                        deprecation: Default::default(),
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None,
                        visible: None,
                        compute_complexity: None,
                        edges: Vec::new(),
                        relation: None,
                        resolve: None,
                        transformer: Some(Transformer::JSONSelect {
                            property: "id".to_string(),
                        }),
                        plan: None,
                        required_operation: Some(Operations::DELETE),
                        auth: auth.cloned(),
                    },
                );
                fields
            },
            cache_control: Default::default(),
            extends: false,
            keys: None,
            is_node: false,
            visible: None,
            is_subscription: false,
            rust_typename: delete_payload_name.clone(),
            constraints: vec![],
        },
        &delete_payload_name,
        &delete_payload_name,
    );

    // deleteMutation
    ctx.mutations.push(MetaField {
        name: format!("{}Delete", to_lower_camelcase(&type_name)),
        description: Some(format!("Delete a {type_name} by ID or unique field")),
        args: {
            let mut args = IndexMap::new();
            args.insert(
                "by".to_owned(),
                MetaInputValue::new("by", format!("{type_name}ByInput!")),
            );
            args
        },
        ty: delete_payload_name,
        deprecation: dynaql::registry::Deprecation::NoDeprecated,
        cache_control: Default::default(),
        external: false,
        provides: None,
        requires: None,
        visible: None,
        edges: Vec::new(),
        relation: None,
        compute_complexity: None,
        resolve: Some(Resolver {
            id: Some(format!("{}_delete_resolver", type_name.to_lowercase())),
            r#type: ResolverType::DynamoMutationResolver(DynamoMutationResolver::DeleteNode {
                ty: type_name,
                by: VariableResolveDefinition::InputTypeName("by".to_owned()),
            }),
        }),
        plan: None,
        transformer: None,
        required_operation: Some(Operations::DELETE),
        auth: auth.cloned(),
    });
}

fn get_length_validator(field: &FieldDefinition) -> Option<DynValidator> {
    use tuple::Map;
    field
        .directives
        .iter()
        .find(|directive| directive.node.name.node == LENGTH_DIRECTIVE)
        .map(|directive| {
            let (min_value, max_value) = (MIN_ARGUMENT, MAX_ARGUMENT).map(|argument_name| {
                directive.node.get_argument(argument_name).and_then(|argument| {
                    if let dynaql_value::ConstValue::Number(ref min) = argument.node {
                        min.as_u64().and_then(|min| min.try_into().ok())
                    } else {
                        None
                    }
                })
            });
            DynValidator::length(min_value, max_value)
        })
}

/// Used to keep track of the parent relation when created nested input types
/// TODO: Merge it with MetaRelation?
pub struct ParentRelation<'a> {
    /// TypeDefinition of @model type
    model_type_definition: &'a TypeDefinition,
    meta: &'a MetaRelation,
}

impl<'a> Display for ParentRelation<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} relation of {}",
            self.meta.name,
            MetaNames::model(self.model_type_definition)
        )
    }
}
