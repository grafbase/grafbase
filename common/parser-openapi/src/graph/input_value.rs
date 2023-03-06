use petgraph::{graph::NodeIndex, visit::EdgeRef};

use super::{Edge, Enum, InputObject, Node, WrappingType};

#[derive(Clone, Debug)]
pub struct InputValue {
    index: NodeIndex,
    wrapping: WrappingType,
}

pub enum InputValueKind {
    Scalar,
    InputObject,
    Enum,
    Union,
}

impl InputValue {
    pub(super) fn from_index(index: NodeIndex, wrapping: WrappingType, graph: &super::OpenApiGraph) -> Option<Self> {
        match &graph.graph[index] {
            Node::Object | Node::Scalar(_) | Node::Enum { .. } | Node::Union => Some(InputValue { index, wrapping }),
            Node::Schema(_) => {
                let type_edge = graph
                    .graph
                    .edges(index)
                    .find(|edge| matches!(edge.weight(), Edge::HasType { .. }))?;

                let Edge::HasType { wrapping: edge_wrapping } = type_edge.weight() else {
                    // This should never happen
                    return None;
                };

                // The HasType edge can introduce more wrapping so we need to make sure to account
                // for that.
                let wrapping = wrapping.wrap_with(edge_wrapping.clone());

                let inner_index = type_edge.target();

                InputValue::from_index(inner_index, wrapping, graph)
            }
            Node::Operation(_) => None,
        }
    }

    pub fn kind(&self, graph: &super::OpenApiGraph) -> Option<InputValueKind> {
        match &graph.graph[self.index] {
            Node::Scalar(_) => Some(InputValueKind::Scalar),
            Node::Object => Some(InputValueKind::InputObject),
            Node::Enum { .. } => Some(InputValueKind::Enum),
            Node::Union => Some(InputValueKind::Union),
            Node::Schema(_) | Node::Operation(_) => None,
        }
    }

    pub fn type_name(&self, graph: &super::OpenApiGraph) -> Option<String> {
        match &graph.graph[self.index] {
            Node::Scalar(s) => Some(s.type_name()),
            Node::Enum { .. } => Enum::from_index(self.index, graph)?.name(graph),
            Node::Object | Node::Union => InputObject::from_index(self.index, graph)?.name(graph),
            Node::Schema(_) | Node::Operation(_) => {
                // These shouldn't really happen
                None
            }
        }
    }

    pub fn as_input_object(&self, graph: &super::OpenApiGraph) -> Option<InputObject> {
        InputObject::from_index(self.index, graph)
    }

    pub fn wrapping_type(&self) -> &WrappingType {
        &self.wrapping
    }
}
