//! This module handles parsing a v2 OpenAPI spec into our intermediate graph

mod components;

use std::collections::BTreeMap;

use grafbase_engine::registry::resolvers::http::{
    ExpectedStatusCode, QueryParameterEncodingStyle, RequestBodyContentType,
};
use inflector::Inflector;
use once_cell::sync::Lazy;
use openapi::v2::{Operation, PathItem};
use regex::Regex;
use url::Url;

use self::components::Components;
use super::grouping;
use crate::{
    graph::{construction::ParentNode, FieldName, HttpMethod, Node, ScalarKind},
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

    extract_operations(&mut ctx, &components, &spec.paths);

    grouping::determine_resource_relationships(&mut ctx);

    ctx
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

            for (status_code, response) in &operation.responses {
                let Ok(status_code) = status_code.parse::<u16>() else {
                    continue;
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
        None | Some(_) => {
            // Not sure what (if anything) to do with these.
            // Going to skip them for now and we can look into it if anyone complains
        }
    }
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

    static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("^[A-Z_][A-Z0-9_]*$").unwrap());
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
}
