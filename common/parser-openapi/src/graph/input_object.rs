use petgraph::{graph::NodeIndex, visit::EdgeRef};

use super::{Edge, InputValue, Node};

#[derive(Clone, Copy, Debug)]
pub struct InputObject(NodeIndex);

pub struct InputField {
    pub value_type: InputValue,
    pub name: String,
}

impl InputObject {
    pub(super) fn from_index(index: NodeIndex, graph: &super::OpenApiGraph) -> Option<Self> {
        match graph.graph[index] {
            Node::Object => Some(InputObject(index)),
            Node::Schema(_) => {
                let inner_index = graph
                    .graph
                    .edges(index)
                    .find(|edge| matches!(edge.weight(), Edge::HasType { .. }))?
                    .target();

                InputObject::from_index(inner_index, graph)
            }
            Node::Operation(_) | Node::Scalar(_) | Node::Union | Node::Enum { .. } => None,
        }
    }

    pub fn fields(self, graph: &super::OpenApiGraph) -> Vec<InputField> {
        graph
            .graph
            .edges(self.0)
            .filter_map(|edge| match edge.weight() {
                super::Edge::HasField { name, wrapping } => Some(InputField {
                    value_type: InputValue::from_index(edge.target(), wrapping.clone(), graph)?,
                    name: name.clone(),
                }),
                _ => None,
            })
            .collect()
    }
}
