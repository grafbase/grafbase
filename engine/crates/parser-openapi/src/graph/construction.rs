use petgraph::{graph::NodeIndex, Graph};
use registry_v2::resolvers::http::{ExpectedStatusCode, QueryParameterEncodingStyle, RequestBodyContentType};
use serde_json::Value;

use super::{Edge, Node, WrappingType};

/// Stores details about the parent of a graph node we're currently constructing,
/// so we can insert the correct edge to each of its children
#[derive(Debug)]
pub enum ParentNode {
    Schema(NodeIndex),
    OperationRequest {
        content_type: RequestBodyContentType,
        operation_index: NodeIndex,
        required: bool,
    },
    OperationResponse {
        status_code: ExpectedStatusCode,
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
        required: bool,
    },
    AllOf(NodeIndex),
}

impl ParentNode {
    fn node_index(&self) -> NodeIndex {
        match self {
            ParentNode::Union(index) | ParentNode::Schema(index) | ParentNode::AllOf(index) => *index,
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
            ParentNode::OperationRequest {
                content_type, required, ..
            } => Edge::HasRequestType {
                content_type: Box::new(content_type.clone()),
                // If a parameter is marked as not required, we need to make sure that we
                // don't record it as required, regardless of what the schema says.
                wrapping: wrapping.set_required(*required),
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
                required: *required,
                wrapping,
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
                // Path parameters are always required, so lets make sure they are here too.
                wrapping: wrapping.wrap_required(),
            },
            ParentNode::QueryParameter {
                name,
                encoding_style,
                required,
                ..
            } => Edge::HasQueryParameter {
                name: name.clone(),
                // If a parameter is marked as not required, we need to make sure that we
                // don't record it as required, regardless of what the schema says.
                wrapping: wrapping.set_required(*required),
                encoding_style: *encoding_style,
            },
            ParentNode::AllOf(_) => Edge::AllOfMember,
        }
    }
}

/// A wrapper around one of the type nodes in the graph that lets us add additional
/// properties to that type
pub struct TypeNode<'a>(NodeIndex, &'a mut Graph<Node, Edge>);

impl<'a> TypeNode<'a> {
    pub fn node_index(self) -> NodeIndex {
        self.0
    }

    pub fn add_default(self, default: Option<&Value>) -> Self {
        if let Some(default_value) = default {
            let default_index = self.1.add_node(Node::Default(Box::new(default_value.clone())));
            self.1.add_edge(self.0, default_index, Edge::HasDefault);
        }
        self
    }

    pub fn add_possible_values<T>(self, values: &[T]) -> Self
    where
        T: serde::Serialize,
    {
        for value in values {
            let value_index = self.1.add_node(Node::PossibleValue(Box::new(
                serde_json::to_value(value).expect("default valueto be serializable"),
            )));
            self.1.add_edge(self.0, value_index, Edge::HasPossibleValue);
        }
        self
    }
}

impl crate::parsing::Context {
    pub fn add_type_node(&mut self, parent: ParentNode, node: Node, nullable: bool) -> TypeNode<'_> {
        let dest_index = self.graph.add_node(node);
        self.add_type_edge(parent, dest_index, nullable);

        TypeNode(dest_index, &mut self.graph)
    }

    pub fn add_type_edge(&mut self, parent: ParentNode, dest_index: NodeIndex, nullable: bool) {
        let src_index = parent.node_index();
        let mut wrapping = WrappingType::Named;
        if !nullable {
            wrapping = wrapping.wrap_required();
        }
        self.graph
            .add_edge(src_index, dest_index, parent.create_edge_weight(wrapping));
    }
}
