use dynaql::registry::resolvers::http::{QueryParameterEncodingStyle, RequestBodyContentType};
use openapiv3::{ReferenceOr, StatusCode, Type};
use petgraph::graph::NodeIndex;

use crate::{
    graph::{ScalarKind, SchemaDetails, WrappingType},
    parsing::{
        components::{Components, Ref},
        operations::OperationDetails,
    },
    Error,
};

use super::{Context, Edge, Node};

pub fn extract_components(ctx: &mut Context, components: &openapiv3::Components) {
    for (name, schema) in &components.schemas {
        // I'm just going to assume that a top-level schema won't be a reference for now.
        // I think the only case where a user would do that is to reference a remote schema,
        // which is a PITA to support so lets not do that right now.
        let Some(schema) = schema.as_item() else {
            ctx.errors.push(Error::TopLevelSchemaWasReference(name.clone()));
            continue;
        };

        let index = ctx
            .graph
            .add_node(Node::Schema(SchemaDetails::new(name.clone(), schema.clone())));

        ctx.schema_index.insert(Ref::schema(name), index);
    }

    // Now we want to extract the spec for each of these schemas into our graph
    for (name, schema) in &components.schemas {
        extract_types(ctx, schema, ParentNode::Schema(ctx.schema_index[&Ref::schema(name)]));
    }
}

pub fn extract_operations(ctx: &mut Context, paths: &openapiv3::Paths, components: Components) {
    for (path, item) in &paths.paths {
        // Also going to assume that paths can't be references for now
        let Some(item) = item.as_item() else {
            ctx.errors.push(Error::TopLevelPathWasReference(path.clone()));
            continue;
        };

        for (method, operation) in item.iter() {
            let Ok(method) = method.parse() else {
                ctx.errors.push(Error::UnknownHttpMethod(method.to_string()));
                continue;
            };

            let operation = match OperationDetails::new(path.clone(), method, operation, &components) {
                Ok(operation) => operation,
                Err(e) => {
                    ctx.errors.push(e);
                    continue;
                }
            };
            let operation_index = ctx.graph.add_node(Node::Operation(operation.clone()));

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
                    ctx.errors.push(Error::OperationMissingResponseSchema(operation.operation_id.clone().unwrap_or_else(|| format!("HTTP {method:?} {path}"))));
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
                    ctx.errors.push(Error::OperationMissingRequestSchema(operation.operation_id.clone().unwrap_or_else(|| format!("HTTP {method:?} {path}"))));
                    continue;
                };
                extract_types(
                    ctx,
                    schema,
                    ParentNode::OperationRequest {
                        content_type: request.content_type.clone(),
                        operation_index,
                    },
                );
            }

            ctx.operation_indices.push(operation_index);
        }
    }
}

enum ParentNode {
    Schema(NodeIndex),
    OperationRequest {
        content_type: RequestBodyContentType,
        operation_index: NodeIndex,
    },
    OperationResponse {
        status_code: StatusCode,
        content_type: String,
        operation_index: NodeIndex,
    },
    Field {
        object_index: NodeIndex,
        field_name: String,
        // Whether the field is required (which is a separate concept from nullable)
        required: bool,
    },
    List {
        nullable: bool,
        parent: Box<ParentNode>,
    },
    Union(NodeIndex),
    PathParameter {
        name: String,
        operation_index: NodeIndex,
    },
    QueryParameter {
        name: String,
        operation_index: NodeIndex,
        encoding_style: QueryParameterEncodingStyle,
    },
}

impl Context {
    fn add_type_node(&mut self, parent: ParentNode, node: Node, nullable: bool) -> NodeIndex {
        let dest_index = self.graph.add_node(node);
        self.add_type_edge(parent, dest_index, nullable);
        dest_index
    }

    fn add_type_edge(&mut self, parent: ParentNode, dest_index: NodeIndex, nullable: bool) {
        let src_index = parent.node_index();
        let mut wrapping = WrappingType::Named;
        if !nullable {
            wrapping = wrapping.wrap_required();
        }
        self.graph
            .add_edge(src_index, dest_index, parent.create_edge_weight(wrapping));
    }
}

impl ParentNode {
    fn node_index(&self) -> NodeIndex {
        match self {
            ParentNode::Union(idx) | ParentNode::Schema(idx) => *idx,
            ParentNode::OperationResponse { operation_index, .. }
            | ParentNode::OperationRequest { operation_index, .. }
            | ParentNode::PathParameter { operation_index, .. }
            | ParentNode::QueryParameter { operation_index, .. } => *operation_index,
            ParentNode::Field { object_index, .. } => *object_index,
            ParentNode::List { parent, .. } => parent.node_index(),
        }
    }

    fn create_edge_weight(&self, wrapping: WrappingType) -> Edge {
        match self {
            ParentNode::Schema(_) => Edge::HasType { wrapping },
            ParentNode::OperationRequest { content_type, .. } => Edge::HasRequestType {
                content_type: content_type.clone(),
                wrapping,
            },
            ParentNode::OperationResponse {
                status_code,
                content_type,
                ..
            } => Edge::HasResponseType {
                content_type: content_type.clone(),
                status_code: status_code.clone(),
                wrapping,
            },
            ParentNode::Field {
                field_name, required, ..
            } => Edge::HasField {
                name: field_name.clone(),
                wrapping: if *required { wrapping.wrap_required() } else { wrapping },
            },
            ParentNode::List { nullable, parent } => {
                // Ok, so call parent.to_edge_weight and then modifiy the wrapping in it.
                // Wrapping the wrapping in a List(Required()) or just List() as appropriate.
                let mut wrapping = wrapping.wrap_list();
                if !nullable {
                    wrapping = wrapping.wrap_required();
                }
                parent.create_edge_weight(wrapping)
            }
            ParentNode::Union { .. } => Edge::HasUnionMember,
            ParentNode::PathParameter { name, .. } => Edge::HasPathParameter {
                name: name.clone(),
                wrapping,
            },
            ParentNode::QueryParameter {
                name, encoding_style, ..
            } => Edge::HasQueryParameter {
                name: name.clone(),
                wrapping,
                encoding_style: *encoding_style,
            },
        }
    }
}

fn extract_types(ctx: &mut Context, schema_or_ref: &ReferenceOr<openapiv3::Schema>, parent: ParentNode) {
    use openapiv3::SchemaKind;

    match schema_or_ref {
        ReferenceOr::Reference { reference } => {
            let reference = Ref::absolute(reference);
            let Some(schema) = ctx.schema_index.get(&reference) else {
                ctx.errors.push(Error::UnresolvedReference(reference));
                return;
            };

            ctx.add_type_edge(parent, *schema, false);
        }
        ReferenceOr::Item(schema) => match &schema.schema_kind {
            SchemaKind::Type(Type::String(ty)) => {
                if ty.enumeration.is_empty() {
                    ctx.add_type_node(parent, Node::Scalar(ScalarKind::String), schema.schema_data.nullable);
                } else {
                    ctx.add_type_node(
                        parent,
                        Node::Enum {
                            values: ty.enumeration.iter().flatten().cloned().collect(),
                        },
                        schema.schema_data.nullable,
                    );
                }
            }
            SchemaKind::Type(Type::Boolean {}) => {
                ctx.add_type_node(parent, Node::Scalar(ScalarKind::Boolean), schema.schema_data.nullable);
            }
            SchemaKind::Type(Type::Integer(_)) => {
                ctx.add_type_node(parent, Node::Scalar(ScalarKind::Integer), schema.schema_data.nullable);
            }
            SchemaKind::Type(Type::Number(_)) => {
                ctx.add_type_node(parent, Node::Scalar(ScalarKind::Float), schema.schema_data.nullable);
            }
            SchemaKind::Type(Type::Object(obj)) => {
                if obj.properties.is_empty() {
                    // If there's no explicit properties we make this a custom scalar
                    ctx.add_type_node(parent, Node::Scalar(ScalarKind::JsonObject), false);
                    return;
                }
                let object_index = ctx.add_type_node(parent, Node::Object, schema.schema_data.nullable);
                for (field_name, field_schema_or_ref) in &obj.properties {
                    let required = obj.required.contains(field_name);
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
                let union_index = ctx.add_type_node(parent, Node::Union, schema.schema_data.nullable);
                for schema in schemas {
                    extract_types(ctx, schema, ParentNode::Union(union_index));
                }
            }
            SchemaKind::AllOf { .. } => {
                ctx.errors.push(Error::AllOfSchema);
            }
            SchemaKind::Not { .. } => {
                ctx.errors.push(Error::NotSchema);
            }
            SchemaKind::Any(_) => {
                ctx.errors.push(Error::AnySchema);
            }
        },
    }
}
