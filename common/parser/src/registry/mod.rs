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
    self, resolvers::dynamo_mutation::DynamoMutationResolver, resolvers::Resolver, resolvers::ResolverType,
    variables::VariableResolveDefinition, MetaField, MetaInputValue,
};
use dynaql::registry::{MetaEnumValue, Registry};
use dynaql::validation::dynamic_validators::DynValidator;
use dynaql::{AuthConfig, CacheControl};
use dynaql_parser::types::{BaseType, FieldDefinition, ObjectType, TypeDefinition};
use grafbase::auth::Operations;
use std::fmt::Display;

use crate::registry::names::MetaNames;
use crate::rules::length_directive::{LENGTH_DIRECTIVE, MAX_ARGUMENT, MIN_ARGUMENT};
use crate::rules::visitor::VisitorContext;
use crate::utils::{to_input_type, to_lower_camelcase};

mod create_update;
pub mod names;
mod pagination;
mod relations;
mod search;
pub use create_update::{add_mutation_create, add_mutation_update, NumericFieldKind};
pub use pagination::{add_query_paginated_collection, generate_pagination_args};
pub use search::add_query_search;

fn register_dynaql_enum<T: DynaqlEnum>(registry: &mut Registry) -> BaseType {
    let type_name = T::ty().to_string();
    registry.create_type(
        |_| registry::EnumType::new(type_name.clone(), T::values().into_iter().map(MetaEnumValue::new)).into(),
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
        |_| {
            dynaql::registry::InputObjectType::new(
                input_type.clone(),
                object.fields.iter().map(|field| MetaInputValue {
                    description: field.node.description.clone().map(|x| x.node),
                    ..MetaInputValue::new(
                        field.name.node.to_string(),
                        to_input_type(&ctx.types, field.node.ty.clone().node),
                    )
                }),
            )
            .with_description(Some(format!("{type_name} input type.")))
            .into()
        },
        &input_type,
        &input_type,
    );

    input_type
}

/// Add the remove mutation for a given Object
pub fn add_remove_mutation(
    ctx: &mut VisitorContext<'_>,
    type_name: &str,
    auth: Option<&AuthConfig>,
    cache_control: CacheControl,
) {
    let delete_payload_name = dynaql::names::deletion_return_type_name(type_name);

    // DeletePayload
    ctx.registry.get_mut().create_type(
        |_| {
            registry::ObjectType::new(
                delete_payload_name.clone(),
                [MetaField {
                    name: names::OUTPUT_FIELD_DELETED_ID.to_string(),
                    description: None,
                    args: Default::default(),
                    // TODO: Should be infered from the entity depending on the directives
                    ty: "ID!".into(),
                    deprecation: Default::default(),
                    cache_control: cache_control.clone(),
                    external: false,
                    requires: None,
                    provides: None,
                    visible: None,
                    compute_complexity: None,
                    edges: Vec::new(),
                    relation: None,
                    resolve: None,
                    transformer: Some(Transformer::JSONSelect {
                        property: names::OUTPUT_FIELD_ID.to_string(),
                    }),
                    plan: None,
                    required_operation: Some(Operations::DELETE),
                    auth: auth.cloned(),
                }],
            )
            .into()
        },
        &delete_payload_name,
        &delete_payload_name,
    );

    // deleteMutation
    ctx.mutations.push(MetaField {
        name: format!("{}Delete", to_lower_camelcase(type_name)),
        description: Some(format!("Delete a {type_name} by ID or unique field")),
        args: {
            let mut args = IndexMap::new();
            args.insert(
                "by".to_owned(),
                MetaInputValue::new("by", format!("{type_name}ByInput!")),
            );
            args
        },
        ty: delete_payload_name.into(),
        deprecation: dynaql::registry::Deprecation::NoDeprecated,
        cache_control,
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
                ty: type_name.into(),
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
