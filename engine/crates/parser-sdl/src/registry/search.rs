use std::collections::HashMap;

use common_types::auth::Operations;
use engine::{
    names::{INPUT_FIELD_FILTER_REGEX, OUTPUT_EDGE_CURSOR},
    registry::{
        self,
        resolvers::{
            query::{QueryResolver, SEARCH_RESOLVER_EDGES, SEARCH_RESOLVER_EDGE_SCORE, SEARCH_RESOLVER_TOTAL_HITS},
            transformer::Transformer,
            Resolver,
        },
        variables::VariableResolveDefinition,
        InputObjectType, MetaField, MetaInputValue, MetaTypeName, NamedType, Registry,
    },
    AuthConfig, Positioned,
};
use engine_parser::types::{FieldDefinition, TypeDefinition};
use itertools::Itertools;
use runtime::search;

use crate::{
    registry::{
        generate_pagination_args,
        names::{
            MetaNames, INPUT_ARG_FIELDS, INPUT_ARG_FILTER, INPUT_ARG_QUERY, INPUT_FIELD_FILTER_ALL,
            INPUT_FIELD_FILTER_ANY, INPUT_FIELD_FILTER_EQ, INPUT_FIELD_FILTER_GT, INPUT_FIELD_FILTER_GTE,
            INPUT_FIELD_FILTER_IN, INPUT_FIELD_FILTER_IS_NULL, INPUT_FIELD_FILTER_LIST_INCLUDES,
            INPUT_FIELD_FILTER_LIST_INCLUDES_NONE, INPUT_FIELD_FILTER_LIST_IS_EMPTY, INPUT_FIELD_FILTER_LT,
            INPUT_FIELD_FILTER_LTE, INPUT_FIELD_FILTER_NEQ, INPUT_FIELD_FILTER_NONE, INPUT_FIELD_FILTER_NOT,
            INPUT_FIELD_FILTER_NOT_IN, PAGINATION_FIELD_EDGES, PAGINATION_FIELD_EDGE_CURSOR,
            PAGINATION_FIELD_EDGE_NODE, PAGINATION_FIELD_EDGE_SEARCH_SCORE, PAGINATION_FIELD_PAGE_INFO,
            PAGINATION_FIELD_SEARCH_INFO, PAGINATION_INPUT_ARG_AFTER, PAGINATION_INPUT_ARG_BEFORE,
            PAGINATION_INPUT_ARG_FIRST, PAGINATION_INPUT_ARG_LAST, SEARCH_INFO_FIELD_TOTAL_HITS, SEARCH_INFO_TYPE,
        },
    },
    rules::{
        cache_directive::CacheDirective,
        model_directive::{METADATA_FIELD_CREATED_AT, METADATA_FIELD_UPDATED_AT},
        search_directive::SEARCH_DIRECTIVE,
        visitor::VisitorContext,
    },
    type_names::TypeNameExt,
};

enum FilterKind {
    Single { scalar: String, is_nullable: bool },
    List { scalar: String },
}

impl From<&str> for FilterKind {
    fn from(ty: &str) -> Self {
        match MetaTypeName::create(ty) {
            MetaTypeName::List(ty) => FilterKind::List {
                scalar: MetaTypeName::concrete_typename(ty).to_string(),
            },
            MetaTypeName::NonNull(ty) => match FilterKind::from(ty) {
                FilterKind::Single { scalar: ty, .. } => FilterKind::Single {
                    scalar: ty,
                    is_nullable: false,
                },
                kind => kind,
            },
            MetaTypeName::Named(ty) => FilterKind::Single {
                scalar: ty.to_string(),
                is_nullable: true,
            },
        }
    }
}

fn convert_to_search_field_type(
    registry: &Registry,
    ty: &str,
    is_nullable: Option<bool>,
) -> Result<search::FieldType, String> {
    match MetaTypeName::create(ty) {
        MetaTypeName::NonNull(type_name) => {
            convert_to_search_field_type(registry, type_name, is_nullable.or(Some(false)))
        }
        MetaTypeName::List(type_name) => convert_to_search_field_type(registry, type_name, Some(true)),
        MetaTypeName::Named(type_name) => {
            let opts = search::FieldOptions {
                nullable: is_nullable.unwrap_or(true),
            };
            Ok(match type_name {
                "URL" => search::FieldType::URL(opts),
                "Email" => search::FieldType::Email(opts),
                "PhoneNumber" => search::FieldType::PhoneNumber(opts),
                "String" => search::FieldType::String(opts),
                "Date" => search::FieldType::Date(opts),
                "DateTime" => search::FieldType::DateTime(opts),
                "Timestamp" => search::FieldType::Timestamp(opts),
                "Int" => search::FieldType::Int(opts),
                "Float" => search::FieldType::Float(opts),
                "Boolean" => search::FieldType::Boolean(opts),
                "IPAddress" => search::FieldType::IPAddress(opts),
                _ => {
                    if registry
                        .types
                        .get(type_name)
                        .map(|meta_type| meta_type.is_enum())
                        .unwrap_or_default()
                    {
                        search::FieldType::String(opts)
                    } else {
                        return Err(type_name.to_string());
                    }
                }
            })
        }
    }
}

pub fn build_search_schema(
    ctx: &mut VisitorContext<'_>,
    model_type_definition: &TypeDefinition,
    fields: &[Positioned<FieldDefinition>],
) -> Option<search::Schema> {
    let search_fields = if model_type_definition
        .directives
        .iter()
        .any(|directive| directive.is_search())
    {
        let mut search_fields: HashMap<String, search::FieldEntry> = fields
            .iter()
            .filter_map(|field| {
                convert_to_search_field_type(&ctx.registry.borrow(), &field.node.ty.node.to_string(), None)
                    .ok()
                    .map(|ty| (field.node.name.node.to_string(), search::FieldEntry { ty }))
            })
            .collect();
        let ty = search::FieldType::DateTime(search::FieldOptions { nullable: false });
        search_fields.insert(
            METADATA_FIELD_CREATED_AT.to_string(),
            search::FieldEntry { ty: ty.clone() },
        );
        search_fields.insert(METADATA_FIELD_UPDATED_AT.to_string(), search::FieldEntry { ty });
        search_fields
    } else {
        let (search_fields, errors): (HashMap<_, _>, Vec<_>) = fields
            .iter()
            .filter_map(|field| {
                field
                    .node
                    .directives
                    .iter()
                    .find(|directive| directive.is_search())
                    .map(|directive| {
                        let field_type =
                            convert_to_search_field_type(&ctx.registry.borrow(), &field.node.ty.node.to_string(), None);
                        field_type
                            .map(|ty| (field.node.name.node.to_string(), search::FieldEntry { ty }))
                            .map_err(|unsupported_type_name| {
                                ctx.report_error(
                                    vec![directive.pos],
                                    format!("The @{SEARCH_DIRECTIVE} directive cannot be used with the {unsupported_type_name} type."),
                                );
                            })
                    })
            })
            .partition_result();
        if !errors.is_empty() {
            return None;
        }
        search_fields
    };

    if search_fields.is_empty() {
        None
    } else {
        Some(search::Schema { fields: search_fields })
    }
}

pub fn add_query_search(
    ctx: &mut VisitorContext<'_>,
    model_type_definition: &TypeDefinition,
    fields: &[Positioned<FieldDefinition>],
    model_auth: Option<&AuthConfig>,
) {
    let Some(schema) = build_search_schema(ctx, model_type_definition, fields) else {
        return;
    };
    let type_name = MetaNames::model(model_type_definition);
    let entity_type = MetaNames::entity_type(model_type_definition);
    let field_filters = {
        let mut field_filters: HashMap<String, FilterKind> = fields
            .iter()
            .filter_map(|field| {
                let name = field.node.name.node.to_string();
                if schema.fields.contains_key(&name) {
                    Some((name, FilterKind::from(field.node.ty.node.to_string().as_str())))
                } else {
                    None
                }
            })
            .collect();
        if schema.fields.contains_key(METADATA_FIELD_CREATED_AT) {
            field_filters.insert(
                METADATA_FIELD_CREATED_AT.to_string(),
                FilterKind::Single {
                    scalar: "DateTime".to_string(),
                    is_nullable: false,
                },
            );
        }
        if schema.fields.contains_key(METADATA_FIELD_UPDATED_AT) {
            field_filters.insert(
                METADATA_FIELD_UPDATED_AT.to_string(),
                FilterKind::Single {
                    scalar: "DateTime".to_string(),
                    is_nullable: false,
                },
            );
        }

        field_filters
    };

    ctx.registry
        .get_mut()
        .search_config
        .indices
        .insert(entity_type.clone(), search::IndexConfig { schema });

    let connection_type = register_connection_type(ctx.registry.get_mut(), model_type_definition, model_auth);
    ctx.queries.push(MetaField {
        name: MetaNames::query_search(model_type_definition),
        mapped_name: None,
        description: Some(format!("Search `{type_name}`")),
        args: {
            let mut pagination_args = generate_pagination_args(ctx.registry.get_mut(), model_type_definition);
            let args = vec![
                MetaInputValue::new(INPUT_ARG_QUERY, "String").with_description("Text to search."),
                MetaInputValue::new(INPUT_ARG_FIELDS, "[String!]").with_description(concat!(
                    "Fields used for searching. ",
                    "Restricted to String, URL, Email and PhoneNumber fields. ",
                    "If not specified it defaults to all @search fields with those types."
                )),
                MetaInputValue::new(
                    INPUT_ARG_FILTER,
                    register_model_filter(ctx.registry.get_mut(), model_type_definition, field_filters),
                ),
                pagination_args
                    .remove(PAGINATION_INPUT_ARG_FIRST)
                    .expect("Has to be present"),
                pagination_args
                    .remove(PAGINATION_INPUT_ARG_AFTER)
                    .expect("Has to be present"),
                pagination_args
                    .remove(PAGINATION_INPUT_ARG_LAST)
                    .expect("Has to be present"),
                pagination_args
                    .remove(PAGINATION_INPUT_ARG_BEFORE)
                    .expect("Has to be present"),
            ];

            args.into_iter().map(|input| (input.name.clone(), input)).collect()
        },
        ty: connection_type.into(),
        deprecation: engine::registry::Deprecation::NoDeprecated,
        cache_control: CacheDirective::parse(&model_type_definition.directives),
        external: false,
        r#override: None,
        provides: None,
        requires: None,
        visible: None,
        compute_complexity: None,
        edges: vec![],
        relation: None,
        resolver: Resolver::Query(QueryResolver::Search {
            query: VariableResolveDefinition::input_type_name(INPUT_ARG_QUERY),
            fields: VariableResolveDefinition::input_type_name(INPUT_ARG_FIELDS),
            filter: VariableResolveDefinition::input_type_name(INPUT_ARG_FILTER),
            first: VariableResolveDefinition::input_type_name(PAGINATION_INPUT_ARG_FIRST),
            after: VariableResolveDefinition::input_type_name(PAGINATION_INPUT_ARG_AFTER),
            before: VariableResolveDefinition::input_type_name(PAGINATION_INPUT_ARG_BEFORE),
            last: VariableResolveDefinition::input_type_name(PAGINATION_INPUT_ARG_LAST),
            type_name: type_name.into(),
            entity_type,
        }),
        required_operation: Some(Operations::LIST),
        auth: model_auth.cloned(),
        shareable: false,
        inaccessible: false,
        tags: vec![],
    });
}

fn register_connection_type(
    registry: &mut Registry,
    model_type_definition: &TypeDefinition,
    model_auth: Option<&AuthConfig>,
) -> String {
    let type_name = MetaNames::search_connection_type(model_type_definition);

    registry.create_type(
        |registry| {
            let edge_type = register_edge_type(registry, model_type_definition, model_auth);
            let search_info_type = register_search_info(registry);
            let page_info_type = super::pagination::register_page_info_type(registry)
                .as_non_null()
                .into();
            registry::ObjectType::new(
                type_name.clone(),
                [
                    MetaField {
                        name: PAGINATION_FIELD_PAGE_INFO.to_string(),
                        ty: page_info_type,
                        required_operation: Some(Operations::LIST),
                        auth: model_auth.cloned(),
                        ..Default::default()
                    },
                    MetaField {
                        name: PAGINATION_FIELD_SEARCH_INFO.to_string(),
                        ty: search_info_type.as_nullable().into(),
                        required_operation: Some(Operations::LIST),
                        auth: model_auth.cloned(),
                        ..Default::default()
                    },
                    MetaField {
                        name: PAGINATION_FIELD_EDGES.to_string(),
                        ty: format!("[{edge_type}!]!").into(),
                        required_operation: Some(Operations::LIST),
                        auth: model_auth.cloned(),
                        resolver: Transformer::select(SEARCH_RESOLVER_EDGES).into(),
                        ..Default::default()
                    },
                ],
            )
            .with_cache_control(CacheDirective::parse(&model_type_definition.directives))
            .into()
        },
        &type_name,
        &type_name,
    );

    type_name
}

fn register_search_info(registry: &mut Registry) -> NamedType<'static> {
    let type_name = SEARCH_INFO_TYPE.to_string();
    registry.create_type(
        |_| {
            registry::ObjectType::new(
                type_name.clone(),
                [MetaField {
                    name: SEARCH_INFO_FIELD_TOTAL_HITS.to_string(),
                    ty: "Int!".into(),
                    resolver: Transformer::select(SEARCH_RESOLVER_TOTAL_HITS).into(),
                    ..Default::default()
                }],
            )
            .into()
        },
        &type_name,
        &type_name,
    );

    type_name.into()
}

fn register_edge_type(
    registry: &mut Registry,
    model_type_definition: &TypeDefinition,
    model_auth: Option<&AuthConfig>,
) -> String {
    let type_name = MetaNames::search_edge_type(model_type_definition);
    let model_name = NamedType::from(MetaNames::model(model_type_definition));
    registry.create_type(
        |_| {
            registry::ObjectType::new(
                type_name.clone(),
                [
                    MetaField {
                        name: PAGINATION_FIELD_EDGE_NODE.to_string(),
                        ty: model_name.as_non_null().into(),
                        required_operation: Some(Operations::LIST),
                        auth: model_auth.cloned(),
                        ..Default::default()
                    },
                    MetaField {
                        name: PAGINATION_FIELD_EDGE_CURSOR.to_string(),
                        ty: "String!".into(),
                        required_operation: Some(Operations::LIST),
                        auth: model_auth.cloned(),
                        resolver: Transformer::select(OUTPUT_EDGE_CURSOR).into(),
                        ..Default::default()
                    },
                    MetaField {
                        name: PAGINATION_FIELD_EDGE_SEARCH_SCORE.to_string(),
                        ty: "Float!".into(),
                        required_operation: Some(Operations::LIST),
                        auth: model_auth.cloned(),
                        resolver: Transformer::select(SEARCH_RESOLVER_EDGE_SCORE).into(),
                        ..Default::default()
                    },
                ],
            )
            .with_cache_control(CacheDirective::parse(&model_type_definition.directives))
            .into()
        },
        &type_name,
        &type_name,
    );

    type_name
}

fn register_model_filter(
    registry: &mut Registry,
    model_type_definition: &TypeDefinition,
    filters: HashMap<String, FilterKind>,
) -> String {
    let input_type_name = MetaNames::search_filter_input(model_type_definition);
    registry.create_type(
        |registry| {
            let mut args = vec![
                MetaInputValue::new(INPUT_FIELD_FILTER_ALL, format!("[{input_type_name}!]")),
                MetaInputValue::new(INPUT_FIELD_FILTER_ANY, format!("[{input_type_name}!]")),
                MetaInputValue::new(INPUT_FIELD_FILTER_NONE, format!("[{input_type_name}!]")),
                MetaInputValue::new(INPUT_FIELD_FILTER_NOT, input_type_name.as_str()),
            ];
            args.extend({
                let mut field_args = filters
                    .into_iter()
                    .map(|(name, kind)| {
                        MetaInputValue::new(
                            name,
                            match kind {
                                FilterKind::Single { scalar, is_nullable } => {
                                    register_scalar_filter(registry, &scalar, is_nullable)
                                }
                                FilterKind::List { scalar } => register_scalar_list_filter(registry, &scalar),
                            },
                        )
                    })
                    .collect::<Vec<_>>();
                // Stable schema
                field_args.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());
                field_args
            });

            InputObjectType::new(input_type_name.clone(), args).into()
        },
        &input_type_name,
        &input_type_name,
    );

    input_type_name
}

fn register_scalar_list_filter(registry: &mut Registry, scalar: &str) -> String {
    // Whether the scalar is really nullable or not, doesn't matter, Tantivy cannot make the
    // difference
    let item_input_type_name = register_scalar_filter(registry, scalar, false);
    let list_input_type_name = MetaNames::search_scalar_list_filter_input(scalar);
    registry.create_type(
        |_| {
            InputObjectType::new(
                list_input_type_name.clone(),
                [
                    MetaInputValue::new(INPUT_FIELD_FILTER_LIST_INCLUDES, item_input_type_name.as_str()),
                    MetaInputValue::new(INPUT_FIELD_FILTER_LIST_INCLUDES_NONE, item_input_type_name.as_str()),
                    MetaInputValue::new(INPUT_FIELD_FILTER_LIST_IS_EMPTY, "Boolean"),
                ],
            )
            .with_oneof(scalar == "Boolean")
            .into()
        },
        &list_input_type_name,
        &list_input_type_name,
    );

    list_input_type_name
}

fn register_scalar_filter(registry: &mut Registry, scalar: &str, is_nullable: bool) -> String {
    let input_type_name = MetaNames::search_scalar_filter_input(scalar, is_nullable);
    registry.create_type(
        |registry| {
            let mut args = vec![];
            if scalar == "Boolean" {
                args.extend([
                    MetaInputValue::new(INPUT_FIELD_FILTER_EQ, scalar),
                    MetaInputValue::new(INPUT_FIELD_FILTER_NEQ, scalar),
                ]);
            } else if registry.types.get(scalar).map(|ty| ty.is_enum()).unwrap_or_default() {
                args.extend([
                    MetaInputValue::new(INPUT_FIELD_FILTER_EQ, scalar),
                    MetaInputValue::new(INPUT_FIELD_FILTER_NEQ, scalar),
                    MetaInputValue::new(INPUT_FIELD_FILTER_IN, format!("[{scalar}!]")),
                    MetaInputValue::new(INPUT_FIELD_FILTER_NOT_IN, format!("[{scalar}!]")),
                ]);
            } else {
                args.extend([
                    MetaInputValue::new(INPUT_FIELD_FILTER_ALL, format!("[{input_type_name}!]")),
                    MetaInputValue::new(INPUT_FIELD_FILTER_ANY, format!("[{input_type_name}!]")),
                    MetaInputValue::new(INPUT_FIELD_FILTER_NONE, format!("[{input_type_name}!]")),
                    MetaInputValue::new(INPUT_FIELD_FILTER_NOT, input_type_name.as_str()),
                    MetaInputValue::new(INPUT_FIELD_FILTER_EQ, scalar),
                    MetaInputValue::new(INPUT_FIELD_FILTER_NEQ, scalar),
                ]);
                let range_scalar = match scalar {
                    "Email" | "PhoneNumber" | "URL" => "String",
                    _ => scalar,
                };
                args.extend([
                    MetaInputValue::new(INPUT_FIELD_FILTER_GT, range_scalar),
                    MetaInputValue::new(INPUT_FIELD_FILTER_GTE, range_scalar),
                    MetaInputValue::new(INPUT_FIELD_FILTER_LTE, range_scalar),
                    MetaInputValue::new(INPUT_FIELD_FILTER_LT, range_scalar),
                    MetaInputValue::new(INPUT_FIELD_FILTER_IN, format!("[{scalar}!]")),
                    MetaInputValue::new(INPUT_FIELD_FILTER_NOT_IN, format!("[{scalar}!]")),
                ]);
                if range_scalar == "String" {
                    args.push(MetaInputValue::new(INPUT_FIELD_FILTER_REGEX, range_scalar));
                }
            }
            if is_nullable {
                args.push(MetaInputValue::new(INPUT_FIELD_FILTER_IS_NULL, "Boolean"));
            }

            InputObjectType::new(input_type_name.clone(), args)
                .with_oneof(scalar == "Boolean")
                .into()
        },
        &input_type_name,
        &input_type_name,
    );

    input_type_name
}
