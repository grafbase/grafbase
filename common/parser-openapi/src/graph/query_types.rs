use std::borrow::Cow;

use inflector::Inflector;
use petgraph::{
    graph::{EdgeIndex, NodeIndex},
    visit::{Dfs, EdgeFiltered, EdgeRef, Reversed, Walker},
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
pub struct QueryOperation(NodeIndex);

#[derive(Clone, Copy)]
pub struct PathParameter(EdgeIndex);

impl super::OpenApiGraph {
    /// Gets an iterator of all the OutputTypes that we'll need in the eventual schema
    pub fn output_types(&self) -> impl Iterator<Item = OutputType> + '_ {
        let mut dfs = Dfs::empty(&self.graph);
        dfs.stack = self.query_operations().into_iter().map(|op| op.0).collect();

        dfs.iter(&self.graph)
            .filter_map(|idx| OutputType::from_index(idx, self))
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

    fn type_name(&self, node: NodeIndex) -> Option<String> {
        match &self.graph[node] {
            schema @ super::Node::Schema { .. } => Some(schema.name()?),
            super::Node::Operation(_) => None,
            super::Node::Object => {
                // OpenAPI objects are generally anonymous so we walk back up the graph to the
                // nearest named thing, and construct a name based on the fields in-betweeen.
                // Not ideal, but the best we can do.
                let reversed_graph = Reversed(&self.graph);
                let filtered_graph = EdgeFiltered::from_fn(reversed_graph, |edge| {
                    matches!(
                        edge.weight(),
                        Edge::HasField { .. }
                            | Edge::HasResponseType { .. }
                            | Edge::HasType { .. }
                            | Edge::HasUnionMember
                    )
                });

                let (_, mut path) = petgraph::algo::astar(
                    &filtered_graph,
                    node,
                    |current_node| self.graph[current_node].name().is_some(),
                    |_| 0,
                    |_| 0,
                )?;

                let named_node = path.pop()?;

                // Reverse our path so we can look things up in the original graph.
                path.reverse();

                let mut name_components = Vec::new();
                let mut path_iter = path.into_iter().peekable();
                while let Some(src_node) = path_iter.next() {
                    let Some(&dest_node) = path_iter.peek() else { break; };

                    // I am sort of assuming there's only one edge here.
                    // Should be the case at the moment but might need to update this to a loop if that changes
                    let edge = self.graph.edges_connecting(src_node, dest_node).next().unwrap();
                    if let Edge::HasField { name, .. } = edge.weight() {
                        name_components.push(name.as_str());
                    }
                }

                let root_name = self.graph[named_node].name().unwrap();
                name_components.push(root_name.as_str());

                name_components.reverse();
                Some(name_components.join("_").to_pascal_case())
            }
            super::Node::Scalar(kind) => Some(kind.type_name()),
            super::Node::Union => {
                // Unions are named based on the names of their constituent types.
                // Although it's perfectly possible for any of the members to be un-named
                // so this will probably require a bit more work at some point.
                let mut name = self
                    .graph
                    .edges(node)
                    .filter_map(|edge| match edge.weight() {
                        Edge::HasUnionMember => OutputType::from_index(edge.target(), self)?.name(self),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("Or");
                name.push_str("Union");
                Some(name)
            }
        }
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

    fn from_index(idx: NodeIndex, graph: &super::OpenApiGraph) -> Option<Self> {
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

impl super::Node {
    // Used to determine whether this specific node type has a name.
    // To generate the full name of a particular node you should use the OpenApiGraph::type_name
    // function.
    fn name(&self) -> Option<String> {
        match self {
            super::Node::Schema(schema) => Some(
                // There's a title property that we _could_ use for a name, but the spec doesn't
                // enforce that it's unique and (certainly in stripes case) it is not.
                // Might do some stuff to work around htat, but for now it's either "x-resourceId"
                // which stripe use or the name of the schema in components.
                schema
                    .openapi
                    .schema_data
                    .extensions
                    .get("x-resourceId")
                    .and_then(|v| v.as_str())
                    .unwrap_or(schema.openapi_name.as_str())
                    .to_pascal_case(),
            ),
            super::Node::Operation(op) => op.operation_id.clone(),
            _ => None,
        }
    }
}
