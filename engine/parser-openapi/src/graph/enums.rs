use petgraph::{
    graph::NodeIndex,
    visit::{Dfs, Walker},
};

use super::Node;

#[derive(Clone, Copy, Debug)]
pub struct Enum(NodeIndex);

impl Enum {
    pub(super) fn from_index(index: NodeIndex, graph: &super::OpenApiGraph) -> Option<Self> {
        match graph.graph[index] {
            Node::Schema(_) => Enum::from_index(graph.schema_target(index)?, graph),
            Node::Enum { .. } => Some(Enum(index)),
            _ => None,
        }
    }

    pub fn name(self, graph: &super::OpenApiGraph) -> Option<String> {
        graph.type_name(self.0)
    }

    pub fn values(self, graph: &super::OpenApiGraph) -> Vec<&str> {
        graph
            .graph
            .neighbors(self.0)
            .filter_map(|index| match &graph.graph[index] {
                Node::PossibleValue(value) => value.as_str(),
                _ => None,
            })
            .collect()
    }
}

impl super::OpenApiGraph {
    pub fn enums(&self) -> Vec<Enum> {
        let mut dfs = Dfs::empty(&self.graph);
        dfs.stack = self.operations().into_iter().map(|op| op.node_index()).collect();

        dfs.iter(&self.graph)
            .filter_map(|idx| Enum::from_index(idx, self))
            .collect()
    }
}
