use petgraph::{
    graph::NodeIndex,
    visit::{Dfs, EdgeFiltered, EdgeRef, IntoEdges, Walker},
};
use serde_json::Value;

use crate::{graph::Arity, output::OutputFieldKind};

use super::{Edge, Enum, Node, OpenApiGraph, Scalar, WrappingType};

/// A node that represents a composite output type in GraphQL
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputType {
    Object(NodeIndex),
    Union(NodeIndex),
}

/// A field of a GraphQL object
pub struct OutputField {
    pub openapi_name: String,
    pub ty: OutputFieldType,
}

/// The type of a field of a GraphQL object - this contains wrapping information, and
/// points at some underlying OutputType/Enum/Scalar
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

    pub fn field(self, openapi_name: &str, graph: &OpenApiGraph) -> Option<OutputField> {
        // Find other_types equivalent to field if it exists
        self.fields(graph)
            .into_iter()
            .find(|field| field.openapi_name == openapi_name)
    }

    pub fn fields(self, graph: &OpenApiGraph) -> Vec<OutputField> {
        graph
            .graph
            .edges(self.index())
            .filter_map(|edge| match edge.weight() {
                super::Edge::HasField { name, wrapping } => Some(OutputField {
                    openapi_name: name.clone(),
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

impl OutputField {
    /// It's fairly common in OpenAPI specs to have a field named `data`
    /// that would probably be called `nodes` in a GraphQL API.
    ///
    /// This function tries to detect those fields so we can rename them.
    pub fn looks_like_nodes_field(&self, graph: &OpenApiGraph) -> bool {
        // For now we're only considering list fields that have the very generic name "data"
        if self.openapi_name != "data" || self.ty.wrapping.arity() != Some(Arity::Many) {
            return false;
        }

        // If the field doesn't point at a schema we probably don't want to count it as an edge.
        let Node::Schema(_) = graph.graph[self.ty.target_index] else {
            return false;
        };

        let reversed_graph = petgraph::visit::Reversed(&graph.graph);

        // We only do this transform on schemas that we consider a "resource"
        // so that we don't pull in arbitrary lists named `data`
        reversed_graph
            .edges(self.ty.target_index)
            .any(|edge| matches!(edge.weight(), Edge::ForResource { .. }))
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
        } else if let Some(OutputType::Union(_)) = OutputType::from_index(self.target_index, graph) {
            OutputFieldKind::Union
        } else {
            OutputFieldKind::Other
        }
    }

    pub fn is_required(&self) -> bool {
        matches!(self.wrapping, WrappingType::NonNull(_))
    }

    pub fn possible_values<'a>(&self, graph: &'a OpenApiGraph) -> Vec<&'a Value> {
        graph
            .graph
            .neighbors(self.target_index)
            .filter_map(|index| match &graph.graph[index] {
                Node::PossibleValue(value) => Some(value),
                _ => None,
            })
            .collect()
    }
}
