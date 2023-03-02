use petgraph::{graph::NodeIndex, visit::EdgeRef};

use super::{Edge, Node, WrappingType};

#[derive(Clone, Debug)]
pub struct InputValue {
    index: NodeIndex,
    wrapping: WrappingType,
}

pub enum InputValueKind {
    Scalar,
    InputObject,
}

impl InputValue {
    pub(super) fn from_index(index: NodeIndex, wrapping: WrappingType, graph: &super::OpenApiGraph) -> Option<Self> {
        match graph.graph[index] {
            Node::Object | Node::Scalar(_) => Some(InputValue { index, wrapping }),
            Node::Union => {
                // This should probably end up as a oneOf object
                todo!("We don't support union inputs yet")
            }
            Node::Schema(_) => {
                let inner_index = graph
                    .graph
                    .edges(index)
                    .find(|edge| matches!(edge.weight(), Edge::HasType { .. }))?
                    .target();

                InputValue::from_index(inner_index, wrapping, graph)
            }
            Node::Operation(_) => None,
        }
    }

    pub fn kind(&self, graph: &super::OpenApiGraph) -> Option<InputValueKind> {
        match &graph.graph[self.index] {
            Node::Scalar(_) => Some(InputValueKind::Scalar),
            Node::Object => Some(InputValueKind::InputObject),
            _ => None,
        }
    }

    pub fn name(&self, graph: &super::OpenApiGraph) -> Option<String> {
        match &graph.graph[self.index] {
            Node::Scalar(s) => Some(s.type_name()),
            _ => todo!("Finish this"),
        }
    }

    pub fn wrapping_type(&self) -> &WrappingType {
        &self.wrapping
    }
}
