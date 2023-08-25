use petgraph::graph::NodeIndex;

use super::{Node, OpenApiGraph};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AllOfMember {
    Object(NodeIndex),
    AllOf(NodeIndex),
}

impl AllOfMember {
    pub(super) fn index(self) -> NodeIndex {
        match self {
            AllOfMember::Object(index) | AllOfMember::AllOf(index) => index,
        }
    }

    pub(super) fn from_index(index: NodeIndex, graph: &OpenApiGraph) -> Option<Self> {
        match graph.graph[index] {
            Node::Object => Some(AllOfMember::Object(index)),
            Node::AllOf => Some(AllOfMember::AllOf(index)),
            Node::Schema(_) => AllOfMember::from_index(graph.schema_target(index)?, graph),
            _ => None,
        }
    }
}
