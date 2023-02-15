use case::CaseExt;
use petgraph::{
    graph::NodeIndex,
    visit::{Dfs, EdgeRef, IntoEdges, Reversed, Walker},
};

use crate::{
    is_ok,
    output::{Field, FieldType},
    parsing::operations::Verb,
};

use super::{Edge, ScalarKind};

#[derive(Clone, Copy)]
pub struct OutputType(NodeIndex);

#[derive(Clone, Copy)]
pub struct QueryOperation(NodeIndex);

impl super::OpenApiGraph {
    /// Gets an iterator of all the OutputTypes that we'll need in the eventual schema
    pub fn output_types(&self) -> impl Iterator<Item = OutputType> + '_ {
        let mut dfs = Dfs::empty(&self.graph);
        dfs.stack = self.query_operations().into_iter().map(|op| op.0).collect();

        dfs.iter(&self.graph)
            .filter_map(|idx| self.graph[idx].object().map(|_| OutputType(idx)))
    }

    /// Gets all the QueryOperations we'll need in the eventual schema
    pub fn query_operations(&self) -> Vec<QueryOperation> {
        self.operation_index
            .iter()
            .filter(|&&idx| {
                self.graph[idx]
                    .operation()
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
                let mut name_components = Vec::new();
                let mut current_node = node;
                'outer: loop {
                    for edge in reversed_graph.edges(current_node) {
                        if let Some(name) = self.graph[edge.target()].name() {
                            name_components.push(name);
                            break 'outer;
                        }

                        match edge.weight() {
                            Edge::HasField { name, .. } => {
                                name_components.push(name.clone());
                                current_node = edge.target();
                                continue 'outer;
                            }
                            Edge::HasResponseType { .. } | Edge::HasType { .. } | Edge::HasUnionMember { .. } => {
                                // For now we follow these edges but they don't contribute to the name.
                                current_node = edge.target();
                                continue 'outer;
                            }
                            _ => {}
                        }
                    }
                    return None;
                }

                name_components.reverse();
                Some(name_components.join("_").to_camel())
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
                        Edge::HasUnionMember => self.graph[edge.target()].name(),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("_Or_");
                name.push_str("Union");
                Some(name)
            }
        }
    }
}

impl OutputType {
    pub fn name(self, graph: &super::OpenApiGraph) -> Option<String> {
        graph.type_name(self.0)
    }

    pub fn fields(self, graph: &super::OpenApiGraph) -> Vec<Field> {
        graph
            .graph
            .edges(self.0)
            .filter_map(|edge| match edge.weight() {
                super::Edge::HasField { name, wrapper } => Some(Field::new(
                    name.clone(),
                    FieldType::new(wrapper, graph.type_name(edge.target())?),
                )),
                _ => None,
            })
            .collect()
    }
}

impl QueryOperation {
    pub fn name(self, graph: &super::OpenApiGraph) -> Option<OperationName> {
        match &graph.graph[self.0] {
            super::Node::Operation(op) => Some(OperationName(op.operation_id.clone()?)),
            _ => None,
        }
    }

    pub fn ty(self, graph: &super::OpenApiGraph) -> Option<FieldType> {
        graph.graph.edges(self.0).find_map(|edge| match edge.weight() {
            super::Edge::HasResponseType {
                status_code, wrapper, ..
            } if is_ok(status_code) => Some(FieldType::new(wrapper, graph.type_name(edge.target())?)),
            _ => None,
        })
    }
}

pub struct OperationName(String);

impl std::fmt::Display for OperationName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.0.to_camel_lowercase();

        // Now actually lowercase the first letter since the above doesn't do that :|
        let mut chars = name.chars();
        let name = match chars.next() {
            None => String::new(),
            Some(c) => c.to_lowercase().collect::<String>() + chars.as_str(),
        };

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
                schema
                    .openapi
                    .schema_data
                    .title
                    .as_ref()
                    .unwrap_or(&schema.name)
                    .to_camel(),
            ),
            super::Node::Operation(op) => op.operation_id.clone(),
            _ => None,
        }
    }
}

impl ScalarKind {
    fn type_name(&self) -> String {
        match self {
            ScalarKind::String => "String".to_string(),
            ScalarKind::Integer => "Int".to_string(),
            ScalarKind::Float => "Float".to_string(),
            ScalarKind::Boolean => "Boolean".to_string(),
            ScalarKind::Id => "ID".to_string(),
        }
    }
}
