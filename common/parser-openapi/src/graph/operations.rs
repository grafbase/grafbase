use inflector::Inflector;
use petgraph::{graph::NodeIndex, visit::EdgeRef};

use crate::{
    is_ok,
    parsing::operations::{HttpMethod, OperationDetails},
};

use super::{
    output_type::OutputFieldType, Edge, Node, PathParameter, QueryParameter, RequestBody, RequestBodyContentType,
};

#[derive(Clone, Copy)]
pub enum Operation {
    Query(NodeIndex),
    Mutation(NodeIndex),
}

impl super::OpenApiGraph {
    /// Gets all the operations we'll need in the eventual schema
    pub fn operations(&self) -> Vec<Operation> {
        self.operation_indices
            .iter()
            .filter_map(|&index| Operation::from_index(index, self))
            .collect()
    }

    /// Gets all the query operations we'll need in the eventual schema
    pub fn query_operations(&self) -> Vec<Operation> {
        self.operation_indices
            .iter()
            .filter(|&&idx| {
                self.graph[idx]
                    .as_operation()
                    .map(|op| op.http_method == HttpMethod::Get)
                    .unwrap_or_default()
            })
            .copied()
            .map(Operation::Query)
            .collect()
    }

    pub fn mutation_operations(&self) -> Vec<Operation> {
        self.operation_indices
            .iter()
            .filter(|&&idx| {
                self.graph[idx]
                    .as_operation()
                    .map(|op| op.http_method != HttpMethod::Get)
                    .unwrap_or_default()
            })
            .copied()
            .map(Operation::Query)
            .collect()
    }
}

impl Operation {
    fn from_index(index: NodeIndex, graph: &super::OpenApiGraph) -> Option<Self> {
        match &graph.graph[index] {
            Node::Operation(details) if details.http_method == HttpMethod::Get => Some(Operation::Query(index)),
            Node::Operation(_) => Some(Operation::Mutation(index)),
            _ => None,
        }
    }

    pub(super) fn node_index(self) -> NodeIndex {
        match self {
            Operation::Query(index) | Operation::Mutation(index) => index,
        }
    }

    pub fn http_method(self, graph: &super::OpenApiGraph) -> Option<String> {
        return Some(self.details(graph)?.http_method.to_string());
    }

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
            .edges(self.node_index())
            .filter_map(|edge| match edge.weight() {
                Edge::HasPathParameter { .. } => Some(PathParameter(edge.id())),
                _ => None,
            })
            .collect()
    }

    pub fn query_parameters(self, graph: &super::OpenApiGraph) -> Vec<QueryParameter> {
        graph
            .graph
            .edges(self.node_index())
            .filter_map(|edge| match edge.weight() {
                Edge::HasQueryParameter { .. } => Some(QueryParameter(edge.id())),
                _ => None,
            })
            .collect()
    }

    pub fn request_body(self, graph: &super::OpenApiGraph) -> Option<RequestBody> {
        let mut potential_bodies = graph
            .graph
            .edges(self.node_index())
            .filter_map(|edge| match edge.weight() {
                Edge::HasRequestType { content_type, .. } => Some((content_type, edge.id())),
                _ => None,
            })
            .collect::<Vec<_>>();

        // Sort the bodies such that we prefer JSON over form encoded
        potential_bodies.sort_by_key(|(content_type, _)| match content_type {
            RequestBodyContentType::Json => 2,
            RequestBodyContentType::FormEncoded(_) => 1,
        });

        let (_, edge_index) = potential_bodies.pop()?;
        Some(RequestBody(edge_index))
    }

    fn details(self, graph: &super::OpenApiGraph) -> Option<&OperationDetails> {
        match &graph.graph[self.node_index()] {
            super::Node::Operation(op) => Some(op),
            _ => None,
        }
    }

    pub fn ty(self, graph: &super::OpenApiGraph) -> Option<OutputFieldType> {
        // A query operation can have a lot of different types: successes/fails,
        // and different content types for each of those scenarios.
        //
        // For now we're just picking the first success response we come across
        // but we'll probably want to do something smarter in the future.
        graph
            .graph
            .edges(self.node_index())
            .find_map(|edge| match edge.weight() {
                super::Edge::HasResponseType {
                    status_code, wrapping, ..
                } if is_ok(status_code) => Some(OutputFieldType::from_index(edge.target(), wrapping)),
                _ => None,
            })
    }
}

pub struct OperationName(String);

impl std::fmt::Display for OperationName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.0.to_camel_case();

        write!(f, "{name}")
    }
}
