use petgraph::{
    graph::NodeIndex,
    visit::{Dfs, EdgeFiltered, EdgeRef, Walker},
};

use crate::output::OutputFieldKind;

use super::{Edge, Enum, Node, OpenApiGraph, Scalar, WrappingType};

#[derive(Clone, Copy, Debug)]
pub enum OutputType {
    Object(NodeIndex),
    Union(NodeIndex),
}

pub struct OutputField {
    pub name: String,
    pub ty: OutputFieldType,
}

pub struct OutputFieldType {
    pub wrapping: WrappingType,
    target_index: NodeIndex,
}

impl OpenApiGraph {
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
    pub fn name(self, graph: &OpenApiGraph) -> Option<String> {
        graph.type_name(self.index())
    }

    pub fn fields(self, graph: &OpenApiGraph) -> Vec<OutputField> {
        graph
            .graph
            .edges(self.index())
            .filter_map(|edge| match edge.weight() {
                super::Edge::HasField { name, wrapping } => Some(OutputField {
                    name: name.clone(),
                    ty: OutputFieldType::from_index(edge.target(), wrapping),
                }),
                _ => None,
            })
            .collect()
    }

    pub fn possible_types(self, graph: &OpenApiGraph) -> Vec<OutputType> {
        graph
            .graph
            .edges(self.index())
            .filter_map(|edge| match edge.weight() {
                super::Edge::HasUnionMember => OutputType::from_index(edge.target(), graph),
                _ => None,
            })
            .flat_map(|output_type| {
                // OpenAPI unions can contain other unions, GraphQL unions cannot.
                // So we flatten any nested unions down here.
                match output_type {
                    OutputType::Object(_) => vec![output_type],
                    OutputType::Union(_) => output_type.possible_types(graph),
                }
            })
            .collect()
    }

    fn index(self) -> NodeIndex {
        match self {
            OutputType::Object(idx) | OutputType::Union(idx) => idx,
        }
    }

    pub(super) fn from_index(index: NodeIndex, graph: &OpenApiGraph) -> Option<Self> {
        match graph.graph[index] {
            Node::Object => Some(OutputType::Object(index)),
            Node::Union => Some(OutputType::Union(index)),
            Node::Schema(_) => OutputType::from_index(graph.schema_target(index)?, graph),
            _ => None,
        }
    }
}

impl OutputFieldType {
    pub(super) fn from_index(index: NodeIndex, wrapping: &WrappingType) -> Self {
        OutputFieldType {
            wrapping: wrapping.clone(),
            target_index: index,
        }
    }

    pub fn type_name(&self, graph: &OpenApiGraph) -> Option<String> {
        if let Some(output_type) = OutputType::from_index(self.target_index, graph) {
            output_type.name(graph)
        } else if let Some(enum_type) = Enum::from_index(self.target_index, graph) {
            enum_type.name(graph)
        } else if let Some(scalar) = Scalar::from_index(self.target_index, graph) {
            scalar.name(graph)
        } else {
            None
        }
    }

    pub fn inner_kind(&self, graph: &OpenApiGraph) -> OutputFieldKind {
        if Enum::from_index(self.target_index, graph).is_some() {
            OutputFieldKind::Enum
        } else {
            OutputFieldKind::Other
        }
    }
}
