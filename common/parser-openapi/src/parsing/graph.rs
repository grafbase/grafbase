use openapiv3::{ReferenceOr, StatusCode, Type};
use petgraph::graph::NodeIndex;

use crate::{
    graph::{ScalarKind, SchemaDetails, WrapperType},
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

        for (verb, operation) in item.iter() {
            let Ok(verb) = verb.parse() else {
                ctx.errors.push(Error::UnknownHttpVerb(verb.to_string()));
                continue;
            };

            let operation = match OperationDetails::new(path.clone(), verb, operation, &components) {
                Ok(operation) => operation,
                Err(e) => {
                    ctx.errors.push(e);
                    continue;
                }
            };
            let index = ctx.graph.add_node(Node::Operation(operation.clone()));

            for response in operation.responses {
                let Some(schema) = &response.schema else {
                    ctx.errors.push(Error::OperationMissingResponseSchema(operation.operation_id.clone().unwrap_or_else(|| format!("HTTP {verb:?} {path}"))));
                    continue;
                };

                extract_types(
                    ctx,
                    schema,
                    ParentNode::OperationResponse {
                        status_code: response.status_code,
                        content_type: response.content_type,
                        operation_index: index,
                    },
                );
            }

            ctx.operation_index.push(index);
        }
    }
}

enum ParentNode {
    Schema(NodeIndex),
    #[allow(dead_code)]
    OperationRequest {
        content_type: String,
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
}

impl ParentNode {
    fn link_type(&self, dest_index: NodeIndex, nullable: bool, ctx: &mut Context) {
        let src_index = self.parent_index();
        let mut wrapper = WrapperType::Named;
        if !nullable {
            wrapper = wrapper.wrap_required();
        }
        let weight = self.to_edge_weight(wrapper);
        ctx.graph.add_edge(src_index, dest_index, weight);
    }

    fn parent_index(&self) -> NodeIndex {
        match self {
            ParentNode::Union(idx) | ParentNode::Schema(idx) => *idx,
            ParentNode::OperationResponse { operation_index, .. }
            | ParentNode::OperationRequest { operation_index, .. } => *operation_index,
            ParentNode::Field { object_index, .. } => *object_index,
            ParentNode::List { parent, .. } => parent.parent_index(),
        }
    }

    fn to_edge_weight(&self, wrapper: WrapperType) -> Edge {
        match self {
            ParentNode::Schema(_) => Edge::HasType { wrapper },
            ParentNode::OperationRequest { content_type, .. } => Edge::HasRequestType {
                content_type: content_type.clone(),
                wrapper,
            },
            ParentNode::OperationResponse {
                status_code,
                content_type,
                ..
            } => Edge::HasResponseType {
                content_type: content_type.clone(),
                status_code: status_code.clone(),
                wrapper,
            },
            ParentNode::Field {
                field_name, required, ..
            } => Edge::HasField {
                name: field_name.clone(),
                wrapper: if *required { wrapper.wrap_required() } else { wrapper },
            },
            ParentNode::List { nullable, parent } => {
                // Ok, so call parent.to_edge_weight and then modifiy the wrapper in it.
                // Wrapping the wrapper in a List(Required()) or just List() as appropriate.
                let mut wrapper = wrapper.wrap_list();
                if !nullable {
                    wrapper = wrapper.wrap_required();
                }
                parent.to_edge_weight(wrapper)
            }
            ParentNode::Union { .. } => Edge::HasUnionMember,
        }
    }
}

impl WrapperType {
    fn wrap_list(self) -> WrapperType {
        WrapperType::List(Box::new(self))
    }

    fn wrap_required(self) -> WrapperType {
        if matches!(self, WrapperType::Required(_)) {
            // Don't double wrap things in required
            self
        } else {
            WrapperType::Required(Box::new(self))
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

            parent.link_type(*schema, false, ctx);
        }
        ReferenceOr::Item(schema) => match &schema.schema_kind {
            SchemaKind::Type(Type::String(_)) => {
                parent.link_type(
                    ctx.graph.add_node(Node::Scalar(ScalarKind::String)),
                    schema.schema_data.nullable,
                    ctx,
                );
            }
            SchemaKind::Type(Type::Boolean {}) => {
                parent.link_type(
                    ctx.graph.add_node(Node::Scalar(ScalarKind::Boolean)),
                    schema.schema_data.nullable,
                    ctx,
                );
            }
            SchemaKind::Type(Type::Integer(_)) => {
                parent.link_type(
                    ctx.graph.add_node(Node::Scalar(ScalarKind::Integer)),
                    schema.schema_data.nullable,
                    ctx,
                );
            }
            SchemaKind::Type(Type::Number(_)) => {
                parent.link_type(
                    ctx.graph.add_node(Node::Scalar(ScalarKind::Float)),
                    schema.schema_data.nullable,
                    ctx,
                );
            }
            SchemaKind::Type(Type::Object(obj)) => {
                let object_index = ctx.graph.add_node(Node::Object);
                parent.link_type(object_index, schema.schema_data.nullable, ctx);
                for (field_name, schema_or_ref) in &obj.properties {
                    let required = obj.required.contains(field_name);
                    extract_types(
                        ctx,
                        &schema_or_ref.clone().unbox(),
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
                let union_index = ctx.graph.add_node(Node::Union);
                parent.link_type(union_index, schema.schema_data.nullable, ctx);
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
