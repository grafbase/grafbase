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
            Node::Enum { .. } => Some(Enum(index)),
            _ => None,
        }
    }

    pub fn name(self, graph: &super::OpenApiGraph) -> Option<String> {
        graph.type_name(self.0)
    }

    pub fn values(self, graph: &super::OpenApiGraph) -> Option<&[String]> {
        match &graph.graph[self.0] {
            Node::Enum { values } => Some(values.as_slice()),
            _ => None,
        }
    }
}

impl super::OpenApiGraph {
    pub fn enums(&self) -> Vec<Enum> {
        let mut dfs = Dfs::empty(&self.graph);
        dfs.stack = self.query_operations().into_iter().map(|op| op.0).collect();

        dfs.iter(&self.graph)
            .filter_map(|idx| Enum::from_index(idx, self))
            .collect()
    }
}
