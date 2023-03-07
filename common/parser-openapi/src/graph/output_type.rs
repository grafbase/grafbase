use petgraph::{
    graph::NodeIndex,
    visit::{Dfs, EdgeFiltered, EdgeRef, Walker},
};

use crate::output::{Field, FieldType};

use super::{Edge, Node};

#[derive(Clone, Copy, Debug)]
pub enum OutputType {
    Object(NodeIndex),
    Union(NodeIndex),
}

impl super::OpenApiGraph {
    /// Gets an iterator of all the OutputTypes that we'll need in the eventual schema
    pub fn output_types(&self) -> Vec<OutputType> {
        let filtered_graph = EdgeFiltered::from_fn(&self.graph, |edge| {
            // Don't follow edges that lead to input types
            !matches!(
                edge.weight(),
                Edge::HasPathParameter { .. } | Edge::HasQueryParameter { .. } | Edge::HasRequestType { .. }
            )
        });

        let mut dfs = Dfs::empty(&filtered_graph);
        dfs.stack = self.operations().into_iter().map(|op| op.node_index()).collect();

        dfs.iter(&filtered_graph)
            .filter_map(|idx| OutputType::from_index(idx, self))
            .collect()
    }
}

impl OutputType {
    pub fn name(self, graph: &super::OpenApiGraph) -> Option<String> {
        graph.type_name(self.index())
    }

    pub fn fields(self, graph: &super::OpenApiGraph) -> Vec<Field> {
        graph
            .graph
            .edges(self.index())
            .filter_map(|edge| match edge.weight() {
                super::Edge::HasField { name, wrapping } => Some(Field::new(
                    name.clone(),
                    FieldType::new(wrapping, graph.type_name(edge.target())?),
                )),
                _ => None,
            })
            .collect()
    }

    pub fn possible_types(self, graph: &super::OpenApiGraph) -> Vec<OutputType> {
        graph
            .graph
            .edges(self.index())
            .filter_map(|edge| match edge.weight() {
                super::Edge::HasUnionMember => OutputType::from_index(edge.target(), graph),
                _ => None,
            })
            .collect()
    }

    fn index(self) -> NodeIndex {
        match self {
            OutputType::Object(idx) | OutputType::Union(idx) => idx,
        }
    }

    pub(super) fn from_index(idx: NodeIndex, graph: &super::OpenApiGraph) -> Option<Self> {
        match graph.graph[idx] {
            Node::Object => Some(OutputType::Object(idx)),
            Node::Union => Some(OutputType::Union(idx)),
            Node::Schema(_) => {
                let inner_index = graph
                    .graph
                    .edges(idx)
                    .find(|edge| matches!(edge.weight(), Edge::HasType { .. }))?
                    .target();

                OutputType::from_index(inner_index, graph)
            }
            _ => None,
        }
    }
}
