//! TODO:
//!
//! -> Split each of the creation and add tests with SDL
//!
use std::fmt::Display;

use crate::registry::names::MetaNames;
use crate::rules::length_directive::{LENGTH_DIRECTIVE, MAX_ARGUMENT, MIN_ARGUMENT};
use crate::rules::visitor::VisitorContext;
use crate::utils::{pagination_arguments, to_input_type, to_lower_camelcase};
use case::CaseExt;
use dynamodb::{constant, ParentRelationId};
use dynaql::indexmap::IndexMap;
use dynaql::registry::relations::MetaRelation;
use dynaql::registry::resolvers::{
    PAGINATION_END_CURSOR, PAGINATION_HAS_NEXT_PAGE, PAGINATION_HAS_PREVIOUS_PAGE, PAGINATION_START_CURSOR,
};
use dynaql::registry::{
    resolvers::context_data::ParentDataResolver, resolvers::dynamo_mutation::MutationResolver,
    resolvers::dynamo_querying::QueryResolver, resolvers::Resolver, MetaField, MetaInputValue, MetaType,
};
use dynaql::validation::dynamic_validators::DynValidator;
use dynaql::{AuthConfig, Operations};
use dynaql_parser::types::{FieldDefinition, ObjectType, Type, TypeDefinition};

mod mutations;
pub mod names;
mod relations;

pub use mutations::{add_mutation_create, add_mutation_update, NumericFieldKind};

use self::names::{
    PAGINATION_INPUT_ARG_AFTER, PAGINATION_INPUT_ARG_BEFORE, PAGINATION_INPUT_ARG_FIRST, PAGINATION_INPUT_ARG_LAST,
};

/// Create an input type for a non_primitive Type.
pub fn add_input_type_non_primitive<'a>(ctx: &mut VisitorContext<'a>, object: &ObjectType, type_name: &str) -> String {
    let type_name = type_name.to_string();
    let input_type = format!("{}Input", type_name.to_camel());

    // Input
    ctx.registry.get_mut().create_type(
        &mut |_| MetaType::InputObject {
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
                            name: name.to_string(),
                            description: field.node.description.clone().map(|x| x.node),
                            ty: to_input_type(&ctx.types, field.node.ty.clone().node).to_string(),
                            validators: None,
                            visible: None,
                            default_value: None,
                            is_secret: false,
                        },
                    );
                }
                input_fields
            },
            visible: None,
            rust_typename: type_name.clone(),
        },
        &input_type,
        &input_type,
    );

    input_type
}

/// Add a query to list a Collection with Relay specification and pagination
///
/// ```graphql
/// type Query {
///   entityCollection(first: Int, last: Int, after: String, before: String): EntityCollection
/// }
/// ```
pub fn add_list_query_paginated<'a>(
    ctx: &mut VisitorContext<'a>,
    type_name: &str,
    connection_edges: Vec<String>,
    auth: Option<&AuthConfig>,
) {
    // Edge
    let edge = format!("{}Edge", type_name.to_camel());
    ctx.registry.get_mut().create_type(
        &mut |_| MetaType::Object {
            name: edge.clone(),
            description: None,
            fields: {
                let mut fields = IndexMap::new();
                let name = "node".to_string();
                let cursor = "cursor".to_string();
                fields.insert(
                    name.clone(),
                    MetaField {
                        name,
                        description: None,
                        args: Default::default(),
                        ty: format!("{}!", type_name.to_camel()),
                        deprecation: Default::default(),
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None,
                        visible: None,
                        compute_complexity: None,
                        edges: Vec::new(),
                        relation: None,
                        resolver: Some(Resolver::parent_object()),
                        required_operation: Some(Operations::LIST),
                        auth: auth.cloned(),
                    },
                );
                fields.insert(
                    cursor.clone(),
                    MetaField {
                        name: cursor,
                        description: None,
                        args: Default::default(),
                        ty: "String!".to_string(),
                        deprecation: Default::default(),
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None,
                        visible: None,
                        compute_complexity: None,
                        edges: Vec::new(),
                        relation: None,
                        resolver: Some(
                            Resolver::field(type_name)
                                .and_then(Resolver::dynamo_attr(constant::SK))
                                .and_then(Resolver::parent(ParentDataResolver::ConvertSkToCursor)),
                        ),
                        required_operation: Some(Operations::LIST),
                        auth: auth.cloned(),
                    },
                );
                fields
            },
            cache_control: dynaql::CacheControl {
                public: true,
                max_age: 0usize,
            },
            extends: false,
            keys: None,
            visible: None,
            is_subscription: false,
            is_node: false,
            rust_typename: edge.clone(),
            constraints: vec![],
        },
        &edge,
        &edge,
    );

    // PageInfo
    let page_info = "PageInfo";
    ctx.registry.get_mut().create_type(
        &mut |_| MetaType::Object {
            name: page_info.to_string(),
            description: None,
            fields: {
                let mut fields = IndexMap::new();
                let previous_page = "hasPreviousPage".to_string();
                let next_page = "hasNextPage".to_string();
                let start_cursor = "startCursor".to_string();
                let end_cursor = "endCursor".to_string();

                fields.insert(
                    previous_page.clone(),
                    MetaField {
                        name: previous_page,
                        description: None,
                        args: Default::default(),
                        ty: "Boolean!".to_string(),
                        deprecation: Default::default(),
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None,
                        visible: None,
                        compute_complexity: None,
                        edges: Vec::new(),
                        relation: None,
                        resolver: Some(Resolver::field(PAGINATION_HAS_PREVIOUS_PAGE)),
                        required_operation: Some(Operations::LIST),
                        auth: auth.cloned(),
                    },
                );
                fields.insert(
                    next_page.clone(),
                    MetaField {
                        name: next_page,
                        description: None,
                        args: Default::default(),
                        ty: "Boolean!".to_string(),
                        deprecation: Default::default(),
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None,
                        visible: None,
                        compute_complexity: None,
                        edges: Vec::new(),
                        relation: None,
                        resolver: Some(Resolver::field(PAGINATION_HAS_NEXT_PAGE)),
                        required_operation: Some(Operations::LIST),
                        auth: auth.cloned(),
                    },
                );
                fields.insert(
                    start_cursor.clone(),
                    MetaField {
                        name: start_cursor,
                        description: None,
                        args: Default::default(),
                        ty: "String".to_string(),
                        deprecation: Default::default(),
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None,
                        visible: None,
                        compute_complexity: None,
                        edges: Vec::new(),
                        relation: None,
                        resolver: Some(Resolver::field(PAGINATION_START_CURSOR)),
                        required_operation: Some(Operations::LIST),
                        auth: auth.cloned(),
                    },
                );
                fields.insert(
                    end_cursor.clone(),
                    MetaField {
                        name: end_cursor,
                        description: None,
                        args: Default::default(),
                        ty: "String".to_string(),
                        deprecation: Default::default(),
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None,
                        visible: None,
                        compute_complexity: None,
                        edges: Vec::new(),
                        relation: None,
                        resolver: Some(Resolver::field(PAGINATION_END_CURSOR)),
                        required_operation: Some(Operations::LIST),
                        auth: auth.cloned(),
                    },
                );
                fields
            },
            cache_control: dynaql::CacheControl {
                public: true,
                max_age: 0usize,
            },
            extends: false,
            keys: None,
            visible: None,
            is_subscription: false,
            is_node: false,
            rust_typename: page_info.to_string(),
            constraints: vec![],
        },
        page_info,
        page_info,
    );

    // Connection
    let connection = format!("{}Connection", type_name.to_camel());
    ctx.registry.get_mut().create_type(
        &mut |_| MetaType::Object {
            name: connection.clone(),
            description: None,
            fields: {
                let mut fields = IndexMap::new();
                let page_info = "pageInfo".to_string();
                let edges = "edges".to_string();
                fields.insert(
                    page_info.clone(),
                    MetaField {
                        name: page_info,
                        description: Some("Information to aid in pagination".to_string()),
                        args: Default::default(),
                        ty: "PageInfo!".to_string(),
                        deprecation: Default::default(),
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None,
                        visible: None,
                        compute_complexity: None,
                        edges: Vec::new(),
                        relation: None,
                        resolver: Some(Resolver::parent(ParentDataResolver::PageInfo)),
                        required_operation: Some(Operations::LIST),
                        auth: auth.cloned(),
                    },
                );
                fields.insert(
                    edges.clone(),
                    MetaField {
                        name: edges,
                        description: None,
                        args: Default::default(),
                        ty: format!("[{}]", &edge),
                        deprecation: Default::default(),
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None,
                        visible: None,
                        compute_complexity: None,
                        edges: connection_edges.clone(),
                        relation: None,
                        resolver: Some(Resolver::parent_object()),
                        required_operation: Some(Operations::LIST),
                        auth: auth.cloned(),
                    },
                );
                fields
            },
            cache_control: dynaql::CacheControl {
                public: true,
                max_age: 0usize,
            },
            extends: false,
            keys: None,
            visible: None,
            is_subscription: false,
            is_node: false,
            rust_typename: connection.clone(),
            constraints: vec![],
        },
        &connection,
        &connection,
    );

    ctx.queries.push(MetaField {
        name: format!("{}Collection", to_lower_camelcase(type_name)),
        description: Some(format!("Paginated query to fetch the whole list of `{type_name}`.")),
        args: pagination_arguments(),
        ty: connection,
        deprecation: dynaql::registry::Deprecation::NoDeprecated,
        cache_control: dynaql::CacheControl {
            public: true,
            max_age: 0usize,
        },
        external: false,
        provides: None,
        requires: None,
        visible: None,
        compute_complexity: None,
        edges: Vec::new(),
        relation: Some(MetaRelation::base_collection_relation(
            format!("{}Collection", to_lower_camelcase(type_name)),
            &Type::new(type_name).expect("Shouldn't fail"),
        )),
        resolver: Some(Resolver::query(QueryResolver::PaginatedByType {
            r#type: Resolver::constant(type_name),
            first: Resolver::input(PAGINATION_INPUT_ARG_FIRST),
            after: Resolver::input(PAGINATION_INPUT_ARG_AFTER),
            before: Resolver::input(PAGINATION_INPUT_ARG_BEFORE),
            last: Resolver::input(PAGINATION_INPUT_ARG_LAST),
            maybe_parent_relation: Resolver::constant::<Option<ParentRelationId>>(None),
        })),
        required_operation: Some(Operations::LIST),
        auth: auth.cloned(),
    });
}

/// Add the remove mutation for a given Object
pub fn add_remove_mutation<'a>(ctx: &mut VisitorContext<'a>, type_name: &str, auth: Option<&AuthConfig>) {
    let type_name = type_name.to_string();
    let delete_payload_name = format!("{}DeletePayload", type_name.to_camel());

    // DeletePayload
    ctx.registry.get_mut().create_type(
        &mut |_| MetaType::Object {
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
                        resolver: Some(Resolver::field("id")),
                        required_operation: Some(Operations::DELETE),
                        auth: auth.cloned(),
                    },
                );
                fields
            },
            cache_control: dynaql::CacheControl {
                public: true,
                max_age: 0usize,
            },
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
                MetaInputValue {
                    name: "by".to_owned(),
                    description: None,
                    ty: format!("{type_name}ByInput!"),
                    default_value: None,
                    validators: None,
                    visible: None,
                    is_secret: false,
                },
            );
            args
        },
        ty: delete_payload_name,
        deprecation: dynaql::registry::Deprecation::NoDeprecated,
        cache_control: dynaql::CacheControl {
            public: true,
            max_age: 0usize,
        },
        external: false,
        provides: None,
        requires: None,
        visible: None,
        edges: Vec::new(),
        relation: None,
        compute_complexity: None,
        resolver: Some(Resolver::mutation(MutationResolver::Delete {
            ty: type_name,
            by: Resolver::input("by"),
        })),
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
