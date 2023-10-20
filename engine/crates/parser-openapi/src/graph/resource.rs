use petgraph::{graph::NodeIndex, visit::EdgeRef, Direction};

use super::{Arity, DebugNode, Edge, Node, OpenApiGraph, Operation, OutputType};

/// A Resource is any named schema in the OpenAPI document that some endpoint
/// directly contains in it's response.
///
/// It's represented in the graph as a Schema with 1 or more ForResource edges
/// pointing at it (with the origin of each edge being the associated operation)
///
/// e.g.
///
/// User <---ForResource {arity: One}---- GetUser Operation
///   ^------ForResource {arity: Many}--- GetUsers Operation
///
/// See `determine_resource_relationships` for how this is calculated
#[derive(Clone, Copy)]
pub struct Resource(NodeIndex);

/// An operation associated with a resource
#[derive(Clone, Copy)]
pub struct ResourceOperation {
    pub operation: Operation,
    pub arity: Arity,
}

impl Resource {
    /// All the operations associated with this resource
    pub fn operations(self, graph: &OpenApiGraph) -> impl Iterator<Item = ResourceOperation> + '_ {
        graph
            .graph
            .edges_directed(self.0, Direction::Incoming)
            .filter_map(|edge| match edge.weight() {
                Edge::ForResource { arity } => Some(ResourceOperation {
                    arity: *arity,
                    operation: Operation::from_index(edge.source(), graph)?,
                }),
                _ => None,
            })
    }

    /// Query operations associated with this resource
    pub fn query_operations(self, graph: &OpenApiGraph) -> impl Iterator<Item = ResourceOperation> + '_ {
        self.operations(graph)
            .filter(|resource_op| matches!(resource_op.operation, Operation::Query(_)))
    }

    /// The name of this resource
    pub fn name(self, graph: &OpenApiGraph) -> Option<String> {
        graph.type_name(self.0)
    }

    /// The OutputType for this resource
    pub fn underlying_type(self, graph: &OpenApiGraph) -> Option<OutputType> {
        OutputType::from_index(self.0, graph)
    }

    fn from_index(index: NodeIndex, graph: &OpenApiGraph) -> Option<Self> {
        let Node::Schema(_) = graph.graph[index] else {
            return None;
        };

        if !graph
            .graph
            .edges_directed(index, Direction::Incoming)
            .any(|edge| matches!(edge.weight(), Edge::ForResource { .. }))
        {
            return None;
        }

        Some(Resource(index))
    }
}

impl DebugNode for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, graph: &OpenApiGraph) -> std::fmt::Result {
        let operations = self.operations(graph).collect::<Vec<_>>();
        f.debug_struct("Resource")
            .field("name", &self.name(graph))
            .field("operations", &operations.debug(graph))
            .finish_non_exhaustive()
    }
}

impl DebugNode for ResourceOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, graph: &OpenApiGraph) -> std::fmt::Result {
        f.debug_struct("ResourceOperation")
            .field("operation", &self.operation.debug(graph))
            .field("arity", &self.arity)
            .finish()
    }
}

impl OpenApiGraph {
    /// All the resources in this Graph
    pub fn resources(&self) -> Vec<Resource> {
        self.graph
            .node_indices()
            .filter_map(|index| Resource::from_index(index, self))
            .collect()
    }
}
