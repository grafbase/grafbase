use engine::registry::resolvers::http::ExpectedStatusCode;
use inflector::Inflector;
use petgraph::{
    graph::NodeIndex,
    visit::{EdgeRef, IntoEdges, Reversed},
};

use super::{
    output_type::OutputFieldType, Arity, Edge, HttpMethod, Node, OperationDetails, PathParameter, QueryParameter,
    RequestBody, RequestBodyContentType,
};
use crate::{is_ok, QueryNamingStrategy};

#[derive(Clone, Copy, Debug)]
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
            .map(Operation::Mutation)
            .collect()
    }
}

impl Operation {
    pub(super) fn from_index(index: NodeIndex, graph: &super::OpenApiGraph) -> Option<Self> {
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

    pub fn http_method(self, graph: &super::OpenApiGraph) -> String {
        self.details(graph).http_method.to_string()
    }

    pub fn url(self, graph: &super::OpenApiGraph) -> String {
        let path = &self.details(graph).path;

        // Remove any leading `/` so we can join cleanly
        // (graph.metadata.url should always have a trailing slash)
        let path = path.trim_start_matches('/');

        // Note that we can't use Url::join here as it'll escape any OpenAPI parameter
        // placeholders.
        format!(
            "{}{path}",
            graph.metadata.url.as_ref().expect("a URL to be in metadata")
        )
    }

    pub fn name(self, graph: &super::OpenApiGraph) -> Option<OperationName> {
        match graph.metadata.query_naming {
            QueryNamingStrategy::OperationId => self.name_by_operation_id(graph),
            QueryNamingStrategy::SchemaName => self.name_by_associated_schema(graph),
        }
    }

    fn name_by_operation_id(self, graph: &super::OpenApiGraph) -> Option<OperationName> {
        let details = self.details(graph);
        let mut name = details.operation_id.clone()?;
        if details.http_method == HttpMethod::Get && name.to_lowercase().starts_with("get") {
            name = name[3..].to_string();
        }

        Some(OperationName(name))
    }

    fn name_by_associated_schema(self, graph: &super::OpenApiGraph) -> Option<OperationName> {
        let Operation::Query(operation_index) = self else {
            // If we have a mutation we always want to use the operationId for naming.
            return self.name_by_operation_id(graph);
        };

        let reversed_graph = Reversed(&graph.graph);

        let arity_and_schema = graph
            .graph
            .edges(operation_index)
            .find_map(|edge| match edge.weight() {
                Edge::ForResource { arity } => Some((arity, edge.target())),
                _ => None,
            })
            .filter(|(operation_arity, schema_index)| {
                // We need to make sure there's only one query operation of this arity
                // associated with this schema, otherwise we'll end up with clashes if we try to
                // use the schema name.
                let count = reversed_graph
                    .edges(*schema_index)
                    .filter_map(|edge| match edge.weight() {
                        Edge::ForResource { arity } if arity == *operation_arity => Some(edge.target()),
                        _ => None,
                    })
                    .filter(|index| matches!(Operation::from_index(*index, graph), Some(Operation::Query(_))))
                    .count();

                count == 1
            });

        match arity_and_schema {
            None => self.name_by_operation_id(graph),
            Some((Arity::One, schema_index)) => Some(OperationName(graph.graph[schema_index].name()?)),
            Some((Arity::Many, schema_index)) => Some(OperationName(graph.graph[schema_index].name()?.to_plural())),
        }
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

    pub fn expected_status(self, graph: &super::OpenApiGraph) -> Option<ExpectedStatusCode> {
        // As in ty below, we're only taking succesful expected statuses for now
        graph
            .graph
            .edges(self.node_index())
            .find_map(|edge| match edge.weight() {
                super::Edge::HasResponseType { status_code, .. } if is_ok(status_code) => Some(status_code.clone()),
                _ => None,
            })
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
        potential_bodies.sort_by_key(|(content_type, _)| match content_type.as_ref() {
            RequestBodyContentType::Json => 2,
            RequestBodyContentType::FormEncoded(_) => 1,
        });

        let (_, edge_index) = potential_bodies.pop()?;
        Some(RequestBody(edge_index))
    }

    fn details(self, graph: &super::OpenApiGraph) -> &OperationDetails {
        match &graph.graph[self.node_index()] {
            super::Node::Operation(op) => op,
            _ => unreachable!(),
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
                    content_type,
                    status_code,
                    wrapping,
                    ..
                } if is_ok(status_code) && is_json(content_type) => {
                    OutputFieldType::from_index(edge.target(), wrapping, graph)
                }
                _ => None,
            })
    }
}

fn is_json(content_type: &str) -> bool {
    content_type == "application/json"
}

#[derive(Debug)]
pub struct OperationName(String);

impl std::fmt::Display for OperationName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.0.to_camel_case();

        write!(f, "{name}")
    }
}
