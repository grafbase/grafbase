use operation::QueryPosition;
use petgraph::visit::EdgeRef;

use crate::query::{FieldFlags, SolutionEdge, SolutionNode};

use super::SolvedQueryWithoutPostProcessing;

impl SolvedQueryWithoutPostProcessing<'_> {
    pub(super) fn assign_root_typename_fields(&mut self) {
        // There is always at least one field in the query, otherwise validation would fail. So
        // either there is an existing partition or there is only __typename fields and we have to
        // create one.
        let first_partition_ix = self
            .graph
            .neighbors(self.root_node_ix)
            .filter(|neighor| matches!(self.graph[*neighor], SolutionNode::QueryPartition { .. }))
            .min_by_key(|partition_node_ix| {
                self.graph
                    .neighbors(*partition_node_ix)
                    .filter_map(|neighbor| match self.graph[neighbor] {
                        SolutionNode::Field { id, .. } => self[id].query_position,
                        _ => None,
                    })
                    .min()
                    .unwrap_or(QueryPosition::MAX)
            })
            .unwrap_or_else(|| {
                let ix = self.0.graph.add_node(SolutionNode::QueryPartition {
                    entity_definition_id: self.operation.root_object_id.into(),
                    resolver_definition_id: self.schema.subgraphs.introspection.resolver_definition_id,
                });
                self.0
                    .graph
                    .add_edge(self.0.root_node_ix, ix, SolutionEdge::QueryPartition);
                ix
            });
        let typename_fields = self
            .graph
            .edges(self.root_node_ix)
            .filter_map(|edge| match edge.weight() {
                SolutionEdge::Field => match self.graph[edge.target()] {
                    SolutionNode::Field { flags, .. } if flags.contains(FieldFlags::TYPENAME) => Some(edge.target()),
                    _ => None,
                },
                _ => None,
            })
            .collect::<Vec<_>>();
        for ix in typename_fields {
            if let Some(id) = self.graph.find_edge(self.root_node_ix, ix) {
                self.graph.remove_edge(id);
            }
            self.graph.add_edge(first_partition_ix, ix, SolutionEdge::Field);
        }
    }
}
