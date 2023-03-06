use std::borrow::Cow;

use dynaql::registry::resolvers::http::QueryParameterEncodingStyle;
use inflector::Inflector;
use petgraph::{
    graph::{EdgeIndex, NodeIndex},
    visit::{Dfs, EdgeFiltered, EdgeRef, Walker},
};

use crate::{
    is_ok,
    output::{Field, FieldType},
    parsing::operations::{OperationDetails, Verb},
};

use super::{input_value::InputValue, Edge, Node};

#[derive(Clone, Copy, Debug)]
pub enum OutputType {
    Object(NodeIndex),
    Union(NodeIndex),
}

#[derive(Clone, Copy)]
pub struct QueryOperation(pub(super) NodeIndex);

#[derive(Clone, Copy)]
pub struct PathParameter(EdgeIndex);

#[derive(Clone, Copy)]
pub struct QueryParameter(EdgeIndex);

impl super::OpenApiGraph {
    /// Gets an iterator of all the OutputTypes that we'll need in the eventual schema
    pub fn output_types(&self) -> Vec<OutputType> {
        let filtered_graph = EdgeFiltered::from_fn(&self.graph, |edge| {
            // Don't follow edges that lead to input types
            !matches!(
                edge.weight(),
                Edge::HasPathParameter { .. } | Edge::HasQueryParameter { .. }
            )
        });

        let mut dfs = Dfs::empty(&filtered_graph);
        dfs.stack = self.query_operations().into_iter().map(|op| op.0).collect();

        dfs.iter(&filtered_graph)
            .filter_map(|idx| OutputType::from_index(idx, self))
            .collect()
    }

    /// Gets all the QueryOperations we'll need in the eventual schema
    pub fn query_operations(&self) -> Vec<QueryOperation> {
        self.operation_indices
            .iter()
            .filter(|&&idx| {
                self.graph[idx]
                    .as_operation()
                    .map(|op| op.verb == Verb::Get)
                    .unwrap_or_default()
            })
            .copied()
            .map(QueryOperation)
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

impl QueryOperation {
    pub fn url(self, graph: &super::OpenApiGraph) -> Option<String> {
        let path = &self.details(graph)?.path;

        // Remove any leading `/` so we can join cleanly
        // (graph.metadata.url should always have a trailing slash)
        let path = path.trim_start_matches('/');

        // Note that we can't use Url::join here as it'll escape any OpenAPI parameter
        // placeholders.
        Some(format!("{}{path}", graph.metadata.url))
    }

    pub fn name(self, graph: &super::OpenApiGraph) -> Option<OperationName> {
        Some(OperationName(self.details(graph)?.operation_id.clone()?))
    }

    pub fn path_parameters(self, graph: &super::OpenApiGraph) -> Vec<PathParameter> {
        graph
            .graph
            .edges(self.0)
            .filter_map(|edge| match edge.weight() {
                Edge::HasPathParameter { .. } => Some(PathParameter(edge.id())),
                _ => None,
            })
            .collect()
    }

    pub fn query_parameters(self, graph: &super::OpenApiGraph) -> Vec<QueryParameter> {
        graph
            .graph
            .edges(self.0)
            .filter_map(|edge| match edge.weight() {
                Edge::HasQueryParameter { .. } => Some(QueryParameter(edge.id())),
                _ => None,
            })
            .collect()
    }

    fn details(self, graph: &super::OpenApiGraph) -> Option<&OperationDetails> {
        match &graph.graph[self.0] {
            super::Node::Operation(op) => Some(op),
            _ => None,
        }
    }

    pub fn ty(self, graph: &super::OpenApiGraph) -> Option<FieldType> {
        // An query operation can have a lot of different types: successes/fails,
        // and different content types for each of those scenarios.
        //
        // For now we're just picking the first success response we come across
        // but we'll probably want to do something smarter in the future.
        graph.graph.edges(self.0).find_map(|edge| match edge.weight() {
            super::Edge::HasResponseType {
                status_code, wrapping, ..
            } if is_ok(status_code) => Some(FieldType::new(wrapping, graph.type_name(edge.target())?)),
            _ => None,
        })
    }
}

impl PathParameter {
    pub fn name(self, graph: &super::OpenApiGraph) -> Option<FieldName<'_>> {
        match graph.graph.edge_weight(self.0)? {
            Edge::HasPathParameter { name, .. } => Some(FieldName(Cow::Borrowed(name))),
            _ => None,
        }
    }

    pub fn input_value(self, graph: &super::OpenApiGraph) -> Option<InputValue> {
        let (_, dest_index) = graph.graph.edge_endpoints(self.0)?;
        match graph.graph.edge_weight(self.0)? {
            Edge::HasPathParameter { wrapping, .. } => InputValue::from_index(dest_index, wrapping.clone(), graph),
            _ => None,
        }
    }
}

impl QueryParameter {
    pub fn name(self, graph: &super::OpenApiGraph) -> Option<&str> {
        match graph.graph.edge_weight(self.0)? {
            Edge::HasQueryParameter { name, .. } => Some(name),
            _ => None,
        }
    }

    pub fn input_value(self, graph: &super::OpenApiGraph) -> Option<InputValue> {
        let (_, dest_index) = graph.graph.edge_endpoints(self.0)?;
        match graph.graph.edge_weight(self.0)? {
            Edge::HasQueryParameter { wrapping, .. } => InputValue::from_index(dest_index, wrapping.clone(), graph),
            _ => None,
        }
    }

    pub fn encoding_style(self, graph: &super::OpenApiGraph) -> Option<QueryParameterEncodingStyle> {
        match graph.graph.edge_weight(self.0)? {
            Edge::HasQueryParameter { encoding_style, .. } => Some(*encoding_style),
            _ => None,
        }
    }
}

pub struct OperationName(String);

impl std::fmt::Display for OperationName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.0.to_camel_case();

        write!(f, "{name}")
    }
}

pub struct FieldName<'a>(Cow<'a, str>);

impl<'a> std::fmt::Display for FieldName<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.0.to_camel_case();

        write!(f, "{name}")
    }
}
