//! This module handles parsing a v3 OpenAPI spec into our intermediate graph

use indexmap::IndexMap;
use inflector::Inflector;
use openapiv3::{AdditionalProperties, ReferenceOr, Type};
use regex::Regex;
use std::sync::LazyLock;
use url::Url;

use self::{components::Components, operations::OperationDetails};
use super::grouping;
use crate::{
    graph::{construction::ParentNode, FieldName, Node, ScalarKind, SchemaDetails},
    parsing::{Context, Ref},
    Error,
};

mod components;
mod operations;

pub fn parse(spec: openapiv3::OpenAPI) -> Context {
    let mut ctx = Context {
        url: Some(url_from_spec(&spec)),
        ..Context::default()
    };

    let mut components = Components::default();
    if let Some(spec_components) = &spec.components {
        components.extend(&mut ctx, spec_components);
        extract_components(&mut ctx, spec_components);
    }

    extract_operations(&mut ctx, &spec.paths, components);

    grouping::determine_resource_relationships(&mut ctx);

    ctx
}

fn extract_components(ctx: &mut Context, components: &openapiv3::Components) {
    for (name, schema) in &components.schemas {
        // I'm just going to assume that a top-level schema won't be a reference for now.
        // I think the only case where a user would do that is to reference a remote schema,
        // which is a PITA to support so lets not do that right now.
        let Some(schema) = schema.as_item() else {
            ctx.errors.push(Error::TopLevelSchemaWasReference(name.clone()));
            continue;
        };

        // There's a title property on schemas that we _could_ use for a name,
        // but the spec doesn't enforce that it's unique and (certainly in stripes case) it is not.
        // Might do some stuff to work around htat, but for now it's either "x-resourceId"
        // which stripe use or the name of the schema in components.
        let resource_id = schema
            .schema_data
            .extensions
            .get("x-resourceId")
            .map(|value| value.to_string());

        let index = ctx
            .graph
            .add_node(Node::Schema(Box::new(SchemaDetails::new(name.clone(), resource_id))));

        ctx.schema_index.insert(Ref::v3_schema(name), index);
    }

    // Now we want to extract the spec for each of these schemas into our graph
    for (name, schema) in &components.schemas {
        extract_types(ctx, schema, ParentNode::Schema(ctx.schema_index[&Ref::v3_schema(name)]));
    }
}

fn extract_operations(ctx: &mut Context, paths: &openapiv3::Paths, components: Components) {
    for (path, path_item) in &paths.paths {
        // Also going to assume that paths can't be references for now
        let Some(path_item) = path_item.as_item() else {
            ctx.errors.push(Error::TopLevelPathWasReference(path.clone()));
            continue;
        };

        for (method, operation) in path_item.iter() {
            let Ok(method) = method.parse() else {
                ctx.errors.push(Error::UnknownHttpMethod(method.to_string()));
                continue;
            };

            let operation =
                match OperationDetails::new(path.clone(), method, operation, &components, &path_item.parameters) {
                    Ok(operation) => operation,
                    Err(e) => {
                        ctx.errors.push(e);
                        continue;
                    }
                };
            let operation_index = ctx
                .graph
                .add_node(Node::Operation(Box::new(crate::graph::OperationDetails {
                    path: operation.path,
                    http_method: operation.http_method,
                    operation_id: operation.operation_id.clone(),
                })));

            for parameter in operation.path_parameters {
                let parent = ParentNode::PathParameter {
                    name: parameter.name,
                    operation_index,
                };
                match parameter.schema {
                    Some(schema) => extract_types(ctx, &schema, parent),
                    None => {
                        // If the parameter has no schema we just assume it's a string.
                        ctx.add_type_node(parent, Node::Scalar(ScalarKind::String), false);
                    }
                }
            }

            for parameter in operation.query_parameters {
                let parent = ParentNode::QueryParameter {
                    name: parameter.name,
                    operation_index,
                    encoding_style: parameter.encoding_style,
                    required: parameter.required,
                };
                match parameter.schema {
                    Some(schema) => extract_types(ctx, &schema, parent),
                    None => {
                        // If the parameter has no schema we just assume it's a string.
                        ctx.add_type_node(parent, Node::Scalar(ScalarKind::String), false);
                    }
                }
            }

            for response in operation.responses {
                let Some(schema) = &response.schema else {
                    ctx.errors.push(Error::OperationMissingResponseSchema(
                        operation
                            .operation_id
                            .clone()
                            .unwrap_or_else(|| format!("HTTP {method:?} {path}")),
                    ));
                    continue;
                };

                extract_types(
                    ctx,
                    schema,
                    ParentNode::OperationResponse {
                        status_code: response.status_code,
                        content_type: response.content_type,
                        operation_index,
                    },
                );
            }

            for request in operation.request_bodies.iter() {
                let Some(schema) = &request.schema else {
                    ctx.errors.push(Error::OperationMissingRequestSchema(
                        operation
                            .operation_id
                            .clone()
                            .unwrap_or_else(|| format!("HTTP {method:?} {path}")),
                    ));
                    continue;
                };
                extract_types(
                    ctx,
                    schema,
                    ParentNode::OperationRequest {
                        content_type: request.content_type.clone(),
                        operation_index,
                        required: request.required,
                    },
                );
            }

            ctx.operation_indices.push(operation_index);
        }
    }
}

fn extract_types(ctx: &mut Context, schema_or_ref: &ReferenceOr<openapiv3::Schema>, parent: ParentNode) {
    use openapiv3::SchemaKind;

    match schema_or_ref {
        ReferenceOr::Reference { reference } => {
            let reference = Ref::absolute(reference);
            let Some(schema) = ctx.schema_index.get(&reference) else {
                ctx.errors.push(reference.to_unresolved_error());
                return;
            };

            ctx.add_type_edge(parent, *schema, false);
        }
        ReferenceOr::Item(schema) => match &schema.schema_kind {
            SchemaKind::Type(Type::String(ty)) => {
                if ty.enumeration.is_empty() || !ty.enumeration.iter().all(is_valid_enum_value) {
                    ctx.add_type_node(parent, Node::Scalar(ScalarKind::String), schema.schema_data.nullable)
                        .add_default(schema.schema_data.default.as_ref())
                        .add_possible_values(&ty.enumeration);
                } else {
                    ctx.add_type_node(parent, Node::Enum, schema.schema_data.nullable)
                        .add_default(schema.schema_data.default.as_ref())
                        .add_possible_values(&ty.enumeration.iter().flatten().rev().cloned().collect::<Vec<_>>());
                }
            }
            SchemaKind::Type(Type::Boolean(_)) => {
                ctx.add_type_node(parent, Node::Scalar(ScalarKind::Boolean), schema.schema_data.nullable)
                    .add_default(schema.schema_data.default.as_ref());
            }
            SchemaKind::Type(Type::Integer(integer)) => {
                ctx.add_type_node(parent, Node::Scalar(ScalarKind::Integer), schema.schema_data.nullable)
                    .add_default(schema.schema_data.default.as_ref())
                    .add_possible_values(&integer.enumeration);
            }
            SchemaKind::Type(Type::Number(number)) => {
                ctx.add_type_node(parent, Node::Scalar(ScalarKind::Float), schema.schema_data.nullable)
                    .add_default(schema.schema_data.default.as_ref())
                    .add_possible_values(&number.enumeration);
            }
            SchemaKind::Type(Type::Object(obj)) => {
                extract_object(
                    ctx,
                    parent,
                    &schema.schema_data,
                    &obj.properties,
                    obj.additional_properties.as_ref(),
                    &obj.required,
                );
            }
            SchemaKind::Type(Type::Array(arr)) => {
                let Some(items) = arr.items.clone() else {
                    // We don't support array without items, so error if it's missing
                    ctx.errors.push(Error::ArrayWithoutItems);
                    return;
                };

                extract_types(
                    ctx,
                    &items.unbox(),
                    ParentNode::List {
                        nullable: schema.schema_data.nullable,
                        parent: Box::new(parent),
                    },
                );
            }
            SchemaKind::OneOf { one_of: schemas } | SchemaKind::AnyOf { any_of: schemas } => {
                if schemas.len() == 1 {
                    // The Stripe API has anyOfs containing a single item.
                    // For ease of use we simplify this to just point direct at the underlying type.
                    extract_types(ctx, schemas.first().unwrap(), parent);
                    return;
                }

                let union_index = ctx
                    .add_type_node(parent, Node::Union, schema.schema_data.nullable)
                    .add_default(schema.schema_data.default.as_ref())
                    .node_index();

                for schema in schemas {
                    extract_types(ctx, schema, ParentNode::Union(union_index));
                }
            }
            SchemaKind::AllOf { all_of } => {
                let node_index = ctx.graph.add_node(Node::AllOf);
                ctx.add_type_edge(parent, node_index, false);

                for schema in all_of {
                    extract_types(ctx, schema, ParentNode::AllOf(node_index));
                }
            }
            SchemaKind::Not { .. } => {
                ctx.errors.push(Error::NotSchema);
            }
            SchemaKind::Any(any) => {
                if any.properties.is_empty() && any.additional_properties.is_none() {
                    ctx.add_type_node(parent, Node::PlaceholderType, false);
                } else {
                    // For now we're assuming this is just an object that openapiv3 doesn't understand
                    extract_object(
                        ctx,
                        parent,
                        &schema.schema_data,
                        &any.properties,
                        any.additional_properties.as_ref(),
                        &any.required,
                    );
                }
            }
        },
    }
}

fn extract_object(
    ctx: &mut Context,
    parent: ParentNode,
    schema_data: &openapiv3::SchemaData,
    properties: &IndexMap<String, ReferenceOr<Box<openapiv3::Schema>>>,
    additional_properties: Option<&AdditionalProperties>,
    required_fields: &[String],
) {
    if properties.is_empty() {
        // If the object is empty _and_ there's no additionalProperties we don't bother
        // emiting an object for it.  Not sure if this is a good idea - could be some APIs
        // that _require_ an empty object.  But lets see what happens
        if additional_properties != Some(&AdditionalProperties::Any(false)) {
            ctx.add_type_node(parent, Node::Scalar(ScalarKind::Json), false);
        }
        return;
    }

    if properties
        .iter()
        .any(|(field_name, _)| !is_valid_field_name(field_name))
    {
        // There's an edge case where field names are made up entirely of symbols and numbers,
        // making it tricky to generate a good GQL name for those fields.
        // For now, I'm just making those objects JSON.
        ctx.add_type_node(parent, Node::Scalar(ScalarKind::Json), false);
        return;
    }

    let object_index = ctx
        .add_type_node(parent, Node::Object, schema_data.nullable)
        .add_default(schema_data.default.as_ref())
        .node_index();

    for (field_name, field_schema_or_ref) in properties {
        let required = required_fields.contains(field_name);
        extract_types(
            ctx,
            &field_schema_or_ref.clone().unbox(),
            ParentNode::Field {
                object_index,
                field_name: field_name.clone(),
                required,
            },
        );
    }
}

// OpenAPI enums can be basically any string, but we're much more limited
/// in GraphQL.  This checks if this value is valid in GraphQL or not.
fn is_valid_enum_value(value: &Option<String>) -> bool {
    static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new("^[A-Z_][A-Z0-9_]*$").unwrap());
    value
        .as_deref()
        .map(|value| value.to_screaming_snake_case())
        .filter(|value| REGEX.is_match(value))
        .is_some()
}

// OpenAPI field names can be basically any string, but we're much more limited
/// in GraphQL.  This checks if this name is valid in GraphQL or not.
fn is_valid_field_name(value: &str) -> bool {
    FieldName::from_openapi_name(value).will_be_valid_graphql()
}

impl Ref {
    pub(super) fn v3_schema(name: &str) -> Ref {
        Ref(format!("#/components/schemas/{name}"))
    }

    pub(super) fn v3_response(name: &str) -> Ref {
        Ref(format!("#/components/responses/{name}"))
    }

    pub(super) fn v3_request_body(name: &str) -> Ref {
        Ref(format!("#/components/requestBodies/{name}"))
    }

    pub(super) fn v3_parameter(name: &str) -> Ref {
        Ref(format!("#/components/parameters/{name}"))
    }
}

fn url_from_spec(spec: &openapiv3::OpenAPI) -> Result<Url, Error> {
    let url_str = spec
        .servers
        .first()
        .map(|server| server.url.as_ref())
        .ok_or(Error::MissingUrl)?;

    let url = Url::parse(url_str).map_err(|_| Error::InvalidUrl(url_str.to_string()))?;

    if !url.has_host() {
        return Err(Error::InvalidUrl(url_str.to_string()));
    }

    Ok(url)
}
