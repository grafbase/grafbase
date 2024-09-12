//! This module handles parsing a v3.1 OpenAPI spec into our intermediate graph

use inflector::Inflector;
use openapiv3::{
    schemars::schema::{InstanceType, ObjectValidation, Schema, SchemaObject, SingleOrVec},
    v3_1::{self as openapiv3_1},
};
use regex::Regex;
use serde_json::Value;
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

pub fn parse(spec: openapiv3_1::OpenApi) -> Context {
    let mut ctx = Context {
        url: Some(url_from_spec(&spec)),
        ..Context::default()
    };

    let mut components = Components::default();
    if let Some(spec_components) = &spec.components {
        components.extend(&mut ctx, spec_components);
        extract_components(&mut ctx, spec_components);
    }

    extract_operations(&mut ctx, spec.paths.as_ref(), components);

    grouping::determine_resource_relationships(&mut ctx);

    ctx
}

fn extract_components(ctx: &mut Context, components: &openapiv3_1::Components) {
    for (name, schema) in &components.schemas {
        // I'm just going to assume that a top-level schema won't be a reference for now.
        // I think the only case where a user would do that is to reference a remote schema,
        // which is a PITA to support so lets not do that right now.
        if schema.json_schema.is_ref() {
            ctx.errors.push(Error::TopLevelSchemaWasReference(name.clone()));
            continue;
        }

        let Schema::Object(schema) = &schema.json_schema else {
            ctx.errors.push(Error::TopLevelSchemaWasBoolean(name.clone()));
            continue;
        };

        // There's a title property on schemas that we _could_ use for a name,
        // but the spec doesn't enforce that it's unique and (certainly in stripes case) it is not.
        // Might do some stuff to work around that, but for now it's either "x-resourceId"
        // which stripe use or the name of the schema in components.
        let resource_id = schema.extensions.get("x-resourceId").map(|value| value.to_string());

        let index = ctx
            .graph
            .add_node(Node::Schema(Box::new(SchemaDetails::new(name.clone(), resource_id))));

        ctx.schema_index.insert(Ref::v3_schema(name), index);
    }

    // Now we want to extract the spec for each of these schemas into our graph
    for (name, schema) in &components.schemas {
        tracing::trace!("Extracting schema {name}");
        extract_types(
            ctx,
            &schema.json_schema,
            ParentNode::Schema(ctx.schema_index[&Ref::v3_schema(name)]),
        );
    }
}

fn extract_operations(ctx: &mut Context, paths: Option<&openapiv3_1::Paths>, components: Components) {
    let Some(paths) = paths else { return };

    for (path, path_item) in &paths.paths {
        // Also going to assume that paths can't be references for now
        let Some(path_item) = path_item.as_item() else {
            ctx.errors.push(Error::TopLevelPathWasReference(path.clone()));
            continue;
        };

        for (method, operation) in path_item.iter() {
            tracing::trace!("Parsing operation: {:?}", operation.operation_id);

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
                tracing::trace!("Parsing path parameter {}", parameter.name);

                let parent = ParentNode::PathParameter {
                    name: parameter.name,
                    operation_index,
                };

                match parameter.schema {
                    Some(schema) => extract_types(ctx, &schema.json_schema, parent),
                    None => {
                        // If the parameter has no schema we just assume it's a string.
                        ctx.add_type_node(parent, Node::Scalar(ScalarKind::String), false);
                    }
                }
            }

            for parameter in operation.query_parameters {
                tracing::trace!("Parsing query parameter {}", parameter.name);
                let parent = ParentNode::QueryParameter {
                    name: parameter.name,
                    operation_index,
                    encoding_style: parameter.encoding_style,
                    required: parameter.required,
                };
                match parameter.schema {
                    Some(schema) => extract_types(ctx, &schema.json_schema, parent),
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
                tracing::trace!(
                    "Parsing response for {:?} {}",
                    response.status_code,
                    response.content_type
                );

                extract_types(
                    ctx,
                    &schema.json_schema,
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

                tracing::trace!("Parsing request for {:?}", request.content_type);

                extract_types(
                    ctx,
                    &schema.json_schema,
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

fn extract_types(ctx: &mut Context, schema: &Schema, parent: ParentNode) {
    let schema = match &schema {
        Schema::Bool(_) => {
            ctx.add_type_node(parent, Node::Scalar(ScalarKind::Json), false);
            return;
        }
        Schema::Object(schema_object) => schema_object,
    };

    if let Some(reference) = &schema.reference {
        tracing::trace!("Inserting schema reference {reference}");
        let reference = Ref::absolute(reference);
        let Some(schema) = ctx.schema_index.get(&reference) else {
            ctx.errors.push(reference.to_unresolved_error());
            return;
        };

        ctx.add_type_edge(parent, *schema, false);
        return;
    }

    match &schema.instance_type {
        None => {}
        Some(SingleOrVec::Single(inner)) => {
            extract_instance_type(ctx, parent, **inner, schema, false);
            return;
        }
        Some(SingleOrVec::Vec(types)) => {
            let nullable = types.contains(&InstanceType::Null);
            let other_types = types.iter().filter(|ty| **ty != InstanceType::Null).collect::<Vec<_>>();
            if other_types.len() == 1 {
                tracing::trace!("Found a nullable instance type");
                extract_instance_type(ctx, parent, **other_types.last().unwrap(), schema, true);
            } else {
                tracing::trace!("Found a complex union, converting to JSON");
                ctx.add_type_node(parent, Node::Scalar(ScalarKind::Json), nullable);
            }
            return;
        }
    }

    if let Some(subschemas) = &schema.subschemas {
        match (&subschemas.one_of, &subschemas.any_of) {
            (Some(schemas), _) | (_, Some(schemas)) => {
                if schemas.len() == 1 {
                    // The Stripe API has anyOfs containing a single item.
                    // For ease of use we simplify this to just point direct at the underlying type.
                    tracing::trace!("Simplifying one_or/any_of");
                    extract_types(ctx, schemas.first().unwrap(), parent);
                    return;
                }
                tracing::trace!("Extracting union from oneOf/anyOf");

                let union_index = ctx
                    .add_type_node(parent, Node::Union, false)
                    .add_default(schema.metadata.as_ref().and_then(|metadata| metadata.default.as_ref()))
                    .node_index();

                for schema in schemas {
                    extract_types(ctx, schema, ParentNode::Union(union_index));
                }
                return;
            }
            _ => {}
        }
        if let Some(all_of) = &subschemas.all_of {
            let node_index = ctx.graph.add_node(Node::AllOf);
            ctx.add_type_edge(parent, node_index, false);

            tracing::trace!("Extracting allOf");
            for schema in all_of {
                extract_types(ctx, schema, ParentNode::AllOf(node_index));
            }
            return;
        }
        tracing::info!("Encountered an unhandled subschema");
        return;
    }

    // If we've got this far we might have to infer type from what other properties are present
    let default = schema.metadata.as_ref().and_then(|metadata| metadata.default.as_ref());
    if schema.object.is_some() {
        tracing::trace!("Extracting unlabelled object");
        extract_object(ctx, parent, schema.object.as_deref(), default, false);
        return;
    }
    if schema.array.is_some() {
        tracing::trace!("Extracting unlabelled array");
        extract_array_type(ctx, parent, schema, false);
        return;
    }

    // We can't determine what type this is, so insert a placeholder for now
    // incase this is one branch of an allOf schema.  If it's not the placeholder will just be ignored
    // in our output.
    tracing::trace!("Defaulting to PlaceholderType");
    ctx.add_type_node(parent, Node::PlaceholderType, false);
}

fn extract_instance_type(
    ctx: &mut Context,
    parent: ParentNode,
    instance_type: InstanceType,
    schema: &SchemaObject,
    nullable: bool,
) {
    let default = schema.metadata.as_ref().and_then(|metadata| metadata.default.as_ref());
    match instance_type {
        InstanceType::Null => {}
        InstanceType::Boolean => {
            tracing::trace!("Extracting boolean");
            ctx.add_type_node(parent, Node::Scalar(ScalarKind::Boolean), nullable)
                .add_default(default);
        }
        InstanceType::Object => {
            extract_object(ctx, parent, schema.object.as_deref(), default, nullable);
        }
        InstanceType::Array => {
            extract_array_type(ctx, parent, schema, nullable);
        }
        InstanceType::String => {
            let enum_values = schema.enum_values.clone().unwrap_or_default();
            if enum_values.is_empty() || !enum_values.iter().all(is_valid_enum_value) {
                ctx.add_type_node(parent, Node::Scalar(ScalarKind::String), nullable)
                    .add_default(default)
                    .add_possible_values(&enum_values);
            } else {
                ctx.add_type_node(parent, Node::Enum, nullable)
                    .add_default(default)
                    .add_possible_values(&enum_values);
            }
        }
        InstanceType::Integer => {
            ctx.add_type_node(parent, Node::Scalar(ScalarKind::Integer), nullable)
                .add_default(default)
                .add_possible_values(&schema.enum_values.clone().unwrap_or_default());
        }
        InstanceType::Number => {
            ctx.add_type_node(parent, Node::Scalar(ScalarKind::Float), nullable)
                .add_default(default)
                .add_possible_values(&schema.enum_values.clone().unwrap_or_default());
        }
    }
}

fn extract_array_type(ctx: &mut Context, parent: ParentNode, schema: &SchemaObject, nullable: bool) {
    let Some(SingleOrVec::Single(items)) = schema.array.as_ref().and_then(|array| array.items.as_ref()) else {
        // Arrays with no item spec _or_ with multiple are hard to deal with properly, so
        // make this a JSON
        tracing::trace!("Extracting array as JSON");
        ctx.add_type_node(parent, Node::Scalar(ScalarKind::Json), nullable);
        return;
    };

    extract_types(
        ctx,
        items.as_ref(),
        ParentNode::List {
            nullable,
            parent: Box::new(parent),
        },
    );
}

fn extract_object(
    ctx: &mut Context,
    parent: ParentNode,
    object: Option<&ObjectValidation>,
    default: Option<&Value>,
    nullable: bool,
) {
    let default_object = ObjectValidation::default();
    let object = object.unwrap_or(&default_object);

    tracing::trace!("Extracting object");

    if object.properties.is_empty() {
        // If the object is empty _and_ there's no additionalProperties we don't bother
        // emiting an object for it.  Not sure if this is a good idea - could be some APIs
        // that _require_ an empty object.  But lets see what happens
        if object.additional_properties != Some(Box::new(Schema::Bool(false))) {
            tracing::trace!("Extracting object as JSON");
            ctx.add_type_node(parent, Node::Scalar(ScalarKind::Json), nullable);
        }
        tracing::trace!("Skipping object explicitly empty object");
        return;
    }

    if object
        .properties
        .iter()
        .any(|(field_name, _)| !is_valid_field_name(field_name))
    {
        // There's an edge case where field names are made up entirely of symbols and numbers,
        // making it tricky to generate a good GQL name for those fields.
        // For now, I'm just making those objects JSON.
        tracing::trace!("Extracting object as JSON because of unsupported names");
        ctx.add_type_node(parent, Node::Scalar(ScalarKind::Json), nullable);
        return;
    }

    let object_index = ctx
        .add_type_node(parent, Node::Object, nullable)
        .add_default(default)
        .node_index();

    for (field_name, field_schema) in &object.properties {
        tracing::trace!("Extracting field {field_name}");
        let required = object.required.contains(field_name);
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
fn is_valid_enum_value(value: &Value) -> bool {
    static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new("^[A-Z_][A-Z0-9_]*$").unwrap());

    value
        .as_str()
        .map(|value| value.to_screaming_snake_case())
        .filter(|value| REGEX.is_match(value))
        .is_some()
}

// OpenAPI field names can be basically any string, but we're much more limited
/// in GraphQL.  This checks if this name is valid in GraphQL or not.
fn is_valid_field_name(value: &str) -> bool {
    FieldName::from_openapi_name(value).will_be_valid_graphql()
}

fn url_from_spec(spec: &openapiv3_1::OpenApi) -> Result<Url, Error> {
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
