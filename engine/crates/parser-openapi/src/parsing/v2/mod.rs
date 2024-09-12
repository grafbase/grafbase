//! This module handles parsing a v2 OpenAPI spec into our intermediate graph

mod components;

use std::collections::BTreeMap;

use inflector::Inflector;
use openapi::v2::{Operation, PathItem};
use petgraph::{graph::NodeIndex, visit::EdgeRef};
use regex::Regex;
use registry_v2::resolvers::http::{ExpectedStatusCode, QueryParameterEncodingStyle, RequestBodyContentType};
use std::sync::LazyLock;
use url::Url;

use self::components::Components;
use super::grouping;
use crate::{
    graph::{construction::ParentNode, Edge, FieldName, HttpMethod, Node, ScalarKind, SchemaDetails},
    parsing::{Context, Ref},
    Error,
};

pub fn parse(spec: openapi::v2::Spec) -> Context {
    let mut ctx = Context {
        url: Some(url_from_spec(&spec)),
        ..Context::default()
    };

    let mut components = Components::default();
    components.extend(&spec);

    if let Some(definitions) = &spec.definitions {
        extract_definitions(&mut ctx, definitions)
    }

    extract_operations(&mut ctx, &components, &spec.paths);

    grouping::determine_resource_relationships(&mut ctx);

    ctx
}

fn extract_definitions(ctx: &mut Context, definitions: &BTreeMap<String, openapi::v2::Schema>) {
    for (name, schema) in definitions {
        // There's a title property on schemas that we _could_ use for a name,
        // but the spec doesn't enforce that it's unique and (certainly in stripes case) it is not.
        // Might do some stuff to work around htat, but for now it's either "x-resourceId"
        // which stripe use or the name of the schema in components.
        let resource_id = schema.other.get("x-resourceId").map(|value| value.to_string());

        let index = ctx
            .graph
            .add_node(Node::Schema(Box::new(SchemaDetails::new(name.clone(), resource_id))));

        ctx.schema_index.insert(Ref::v2_definition(name), index);
    }

    // Now we want to extract the schema for each of these into our graph
    for (name, schema) in definitions {
        extract_types(
            ctx,
            schema,
            ParentNode::Schema(ctx.schema_index[&Ref::v2_definition(name)]),
        );
    }
}

fn extract_operations(ctx: &mut Context, components: &Components, paths: &BTreeMap<String, PathItem>) {
    for (path, operations) in paths {
        for (http_method, operation) in operations_iter(operations) {
            let operation_index = ctx
                .graph
                .add_node(Node::Operation(Box::new(crate::graph::OperationDetails {
                    path: path.clone(),
                    http_method,
                    operation_id: operation.operation_id.clone(),
                })));

            for parameter_or_ref in operation.parameters.as_deref().unwrap_or(&[]) {
                let parameter = match &parameter_or_ref {
                    openapi::v2::ParameterOrRef::Parameter(parameter) => parameter,
                    openapi::v2::ParameterOrRef::Ref { ref_path } => {
                        let parameter_ref = Ref::absolute(ref_path);
                        let Some(parameter) = components.parameters.get(&parameter_ref) else {
                            ctx.errors.push(parameter_ref.to_unresolved_error());
                            continue;
                        };
                        parameter
                    }
                };
                match parameter.location.as_str() {
                    "path" => {
                        let parent = ParentNode::PathParameter {
                            name: parameter.name.clone(),
                            operation_index,
                        };
                        match &parameter.schema {
                            Some(schema) => extract_types(ctx, schema, parent),
                            None => {
                                // If the parameter has no schema we just assume it's a string.
                                ctx.add_type_node(parent, Node::Scalar(ScalarKind::String), false);
                            }
                        }
                    }
                    "query" => {
                        let parent = ParentNode::QueryParameter {
                            name: parameter.name.clone(),
                            operation_index,
                            encoding_style: QueryParameterEncodingStyle::Form,
                            required: parameter.required.unwrap_or(false),
                        };
                        match &parameter.schema {
                            Some(schema) => extract_types(ctx, schema, parent),
                            None => {
                                // If the parameter has no schema we just assume it's a string.
                                ctx.add_type_node(parent, Node::Scalar(ScalarKind::String), false);
                            }
                        }
                    }
                    "body" => {
                        let Some(schema) = &parameter.schema else {
                            ctx.errors.push(Error::OperationMissingRequestSchema(
                                operation
                                    .operation_id
                                    .clone()
                                    .unwrap_or_else(|| format!("HTTP {http_method:?} {path}")),
                            ));
                            continue;
                        };

                        extract_types(
                            ctx,
                            schema,
                            ParentNode::OperationRequest {
                                content_type: RequestBodyContentType::Json, // Just assuming JSON for now
                                operation_index,
                                required: parameter.required.unwrap_or(false),
                            },
                        );
                    }
                    _ => {}
                }
            }

            for (status_code, response_or_ref) in &operation.responses {
                let Ok(status_code) = status_code.parse::<u16>() else {
                    continue;
                };

                let response = match &response_or_ref {
                    openapi::v2::ResponseOrRef::Response(response) => response,
                    openapi::v2::ResponseOrRef::Ref { ref_path } => {
                        let response_ref = Ref::absolute(ref_path);
                        let Some(response) = components.responses.get(&response_ref) else {
                            ctx.errors.push(response_ref.to_unresolved_error());
                            continue;
                        };
                        response
                    }
                };

                let parent_node = ParentNode::OperationResponse {
                    status_code: ExpectedStatusCode::Exact(status_code),
                    content_type: "application/json".into(), // Just assuming JSON for now
                    operation_index,
                };

                let Some(schema) = &response.schema else {
                    // This happens all the time in the planetscale schema.  Often just for errors,
                    // but most of the delete operations also have no schema so lets just mark them
                    // as JSON
                    ctx.add_type_node(parent_node, Node::Scalar(ScalarKind::Json), true);
                    continue;
                };

                extract_types(ctx, schema, parent_node);
            }

            ctx.operation_indices.push(operation_index);
        }
    }
}

fn extract_types(ctx: &mut Context, schema: &openapi::v2::Schema, parent: ParentNode) {
    if let Some(reference) = &schema.ref_path {
        // Ideally these are just references to a top level definition
        // like `#/definitions/blah` - but they might also have nested paths
        // on the end we need to handle like `#/definitions/blah/properties/data`
        let segments = reference.split('/').collect::<Vec<_>>();
        if segments.len() > 3 {
            let schema_reference = Ref::absolute(&segments[0..3].join("/"));
            let Some(schema) = ctx.schema_index.get(&schema_reference) else {
                ctx.errors.push(schema_reference.to_unresolved_error());
                return;
            };

            let Some(destination_index) = resolve_nested_ref(*schema, &segments[3..], ctx) else {
                ctx.errors.push(Ref::absolute(reference).to_unresolved_error());
                return;
            };

            ctx.add_type_edge(parent, destination_index, false);
            return;
        }
        let reference = Ref::absolute(reference);
        let Some(schema) = ctx.schema_index.get(&reference) else {
            ctx.errors.push(reference.to_unresolved_error());
            return;
        };

        ctx.add_type_edge(parent, *schema, false);
        return;
    }

    match schema.schema_type.as_deref() {
        Some("string") => {
            let enum_values = schema.enum_values.clone().unwrap_or_default();
            if enum_values.is_empty() || !enum_values.iter().all(is_valid_enum_value) {
                ctx.add_type_node(parent, Node::Scalar(ScalarKind::String), false)
                    .add_possible_values(enum_values.as_slice());
            } else {
                ctx.add_type_node(parent, Node::Enum, false)
                    .add_possible_values(enum_values.as_slice());
            }
        }
        Some("boolean") => {
            ctx.add_type_node(parent, Node::Scalar(ScalarKind::Boolean), false)
                .add_possible_values(schema.enum_values.as_ref().unwrap_or(&vec![]));
        }
        Some("integer") => {
            ctx.add_type_node(parent, Node::Scalar(ScalarKind::Integer), false)
                .add_possible_values(schema.enum_values.as_ref().unwrap_or(&vec![]));
        }
        Some("number") => {
            ctx.add_type_node(parent, Node::Scalar(ScalarKind::Float), false)
                .add_possible_values(schema.enum_values.as_ref().unwrap_or(&vec![]));
        }
        Some("object") => {
            let no_properties = BTreeMap::new();
            let no_required = Vec::new();
            extract_object(
                ctx,
                parent,
                schema.properties.as_ref().unwrap_or(&no_properties),
                schema.required.as_ref().unwrap_or(&no_required),
            );
        }
        Some("array") => {
            let Some(items) = schema.items.as_ref() else {
                // We don't support array without items, so error if it's missing
                ctx.errors.push(Error::ArrayWithoutItems);
                return;
            };

            extract_types(
                ctx,
                items,
                ParentNode::List {
                    nullable: false,
                    parent: Box::new(parent),
                },
            );
        }
        None if schema.properties.as_ref().map(|p| !p.is_empty()).unwrap_or_default() => {
            // Presumably this is an unlabelled object
            let no_properties = BTreeMap::new();
            let no_required = Vec::new();
            extract_object(
                ctx,
                parent,
                schema.properties.as_ref().unwrap_or(&no_properties),
                schema.required.as_ref().unwrap_or(&no_required),
            );
        }
        None if schema.items.is_some() => {
            let items = schema.items.as_ref().unwrap();

            extract_types(
                ctx,
                items,
                ParentNode::List {
                    nullable: false,
                    parent: Box::new(parent),
                },
            );
        }
        ty @ None | ty @ Some(_) => {
            tracing::warn!("Unknown schema type: {ty:?}, skipping");
            // Not sure what (if anything) to do with these.
            // Going to skip them for now and we can look into it if anyone complains
        }
    }
}

fn resolve_nested_ref(mut current_node: NodeIndex, path_segments: &[&str], ctx: &Context) -> Option<NodeIndex> {
    for pair in path_segments.chunks_exact(2) {
        let [kind_selector, index] = pair else { unreachable!() };
        loop {
            match (*kind_selector, &ctx.graph[current_node]) {
                ("properties", Node::Schema(_)) => {
                    current_node = ctx.graph.edges(current_node).find_map(|edge| match edge.weight() {
                        Edge::HasType { .. } => Some(edge.target()),
                        _ => None,
                    })?;
                }
                ("properties", Node::Object) => {
                    current_node = ctx.graph.edges(current_node).find_map(|edge| match edge.weight() {
                        Edge::HasField { name, .. } if name == index => Some(edge.target()),
                        _ => None,
                    })?;
                    break;
                }
                _ => {
                    // The above deal with objects, we could maybe need to deal with array types
                    // or similar.  But for now I don't need to so nope.
                    return None;
                }
            }
        }
    }

    Some(current_node)
}

fn extract_object(
    ctx: &mut Context,
    parent: ParentNode,
    properties: &BTreeMap<String, openapi::v2::Schema>,
    required_fields: &[String],
) {
    if properties.is_empty() {
        // If the object is empty we don't bother emiting an object for it.
        // Not sure if this is a good idea - could be some APIs
        // that _require_ an empty object.  But lets see what happens
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

    let object_index = ctx.add_type_node(parent, Node::Object, false).node_index();

    for (field_name, field_schema) in properties {
        let required = required_fields.contains(field_name);
        extract_types(
            ctx,
            field_schema,
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
fn is_valid_enum_value(value: &serde_json::Value) -> bool {
    let serde_json::Value::String(string) = value else {
        return false;
    };

    static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new("^[A-Z_][A-Z0-9_]*$").unwrap());
    let string = string.to_screaming_snake_case();

    REGEX.is_match(&string)
}

// OpenAPI field names can be basically any string, but we're much more limited
/// in GraphQL.  This checks if this name is valid in GraphQL or not.
fn is_valid_field_name(value: &str) -> bool {
    FieldName::from_openapi_name(value).will_be_valid_graphql()
}

fn operations_iter(operations: &PathItem) -> impl Iterator<Item = (HttpMethod, &Operation)> {
    [
        (HttpMethod::Get, &operations.get),
        (HttpMethod::Post, &operations.post),
        (HttpMethod::Put, &operations.put),
        (HttpMethod::Patch, &operations.patch),
        (HttpMethod::Delete, &operations.delete),
    ]
    .into_iter()
    .filter_map(|(method, maybe_operation)| Some((method, maybe_operation.as_ref()?)))
}

fn url_from_spec(spec: &openapi::v2::Spec) -> Result<Url, Error> {
    let url_string = format!(
        "https://{}{}",
        spec.host.as_deref().ok_or(Error::MissingUrl)?,
        spec.base_path.as_deref().unwrap_or("")
    );

    let url = Url::parse(&url_string).map_err(|_| Error::InvalidUrl(url_string))?;

    Ok(url)
}

impl Ref {
    fn v2_response(name: &str) -> Ref {
        Ref(format!("#/responses/{name}"))
    }

    fn v2_path(name: &str) -> Ref {
        Ref(format!("#/paths/{name}"))
    }

    fn v2_parameter(name: &str) -> Ref {
        Ref(format!("#/parameters/{name}"))
    }

    fn v2_definition(name: &str) -> Ref {
        Ref(format!("#/definitions/{name}"))
    }
}
