use petgraph::graph::NodeIndex;

use super::Node;

#[derive(Clone, Copy, Debug)]
pub struct Scalar(NodeIndex);

impl Scalar {
    pub(super) fn from_index(index: NodeIndex, graph: &super::OpenApiGraph) -> Option<Self> {
        match graph.graph[index] {
            Node::Schema(_) => Scalar::from_index(graph.schema_target(index)?, graph),
            Node::Scalar(_) => Some(Scalar(index)),
            _ => None,
        }
    }

    pub fn name(self, graph: &super::OpenApiGraph) -> Option<String> {
        graph.type_name(self.0)
    }
}
