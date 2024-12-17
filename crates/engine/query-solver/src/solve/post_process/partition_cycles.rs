use operation::QueryPosition;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeFiltered;
use petgraph::visit::EdgeRef;
use schema::ResolverDefinitionId;

use crate::query::{FieldFlags, SolutionEdge, SolutionNode};

use super::SolvedQueryWithoutPostProcessing;

impl SolvedQueryWithoutPostProcessing<'_> {
    pub(super) fn split_query_partition_dependency_cycles(&mut self, starting_nodes: Vec<NodeIndex>) {
        struct Field {
            position: Option<QueryPosition>,
            original_partition_node_ix: NodeIndex,
            resolver_definition_id: ResolverDefinitionId,
            field_node_ix: NodeIndex,
        }
        let mut partition_fields = Vec::new();
        let mut stack = starting_nodes;
        let mut partitions = Vec::new();

        while let Some(root_node_ix) = stack.pop() {
            partitions.clear();
            debug_assert!(partition_fields.is_empty());
            for edge in self.graph.edges(root_node_ix) {
                if !matches!(edge.weight(), SolutionEdge::Field | SolutionEdge::QueryPartition) {
                    continue;
                }
                match self.graph[edge.target()] {
                    SolutionNode::QueryPartition {
                        resolver_definition_id, ..
                    } => {
                        partitions.push((edge.target(), resolver_definition_id));
                        for second_degree_edge in self.graph.edges(edge.target()) {
                            if !matches!(
                                second_degree_edge.weight(),
                                SolutionEdge::Field | SolutionEdge::QueryPartition
                            ) {
                                continue;
                            }
                            let node_ix = second_degree_edge.target();
                            if let SolutionNode::Field { id, flags, .. } = self.graph[node_ix] {
                                partition_fields.push(Field {
                                    position: self[id].query_position,
                                    original_partition_node_ix: edge.target(),
                                    resolver_definition_id,
                                    field_node_ix: node_ix,
                                });
                                if flags.contains(FieldFlags::IS_COMPOSITE_TYPE) {
                                    stack.push(node_ix);
                                }
                            }
                        }
                    }
                    SolutionNode::Field { flags, .. } => {
                        if flags.contains(FieldFlags::IS_COMPOSITE_TYPE) {
                            stack.push(edge.target());
                        }
                    }
                    _ => (),
                }
            }

            partition_fields.sort_unstable_by(|a, b| a.position.cmp(&b.position));

            // Removing edges to the parent partitions
            for field in &partition_fields {
                if let Some(id) = self
                    .graph
                    .find_edge(field.original_partition_node_ix, field.field_node_ix)
                {
                    self.graph.remove_edge(id);
                }
            }

            for Field {
                original_partition_node_ix,
                resolver_definition_id,
                field_node_ix,
                ..
            } in partition_fields.drain(..)
            {
                let partition_node_ix = partitions
                    .iter()
                    .filter(|(_, id)| *id == resolver_definition_id)
                    .filter_map(|(partition_node_ix, _)| {
                        let is_connected = self
                            .graph
                            .edges(*partition_node_ix)
                            .filter_map(|edge| {
                                if matches!(edge.weight(), SolutionEdge::Field) {
                                    Some(edge.target())
                                } else {
                                    None
                                }
                            })
                            .any(|partition_field_node_ix| {
                                petgraph::algo::has_path_connecting(
                                    &EdgeFiltered::from_fn(&self.graph, |edge| {
                                        matches!(edge.weight(), SolutionEdge::RequiredBySubgraph)
                                    }),
                                    partition_field_node_ix,
                                    field_node_ix,
                                    None,
                                )
                            });
                        if is_connected {
                            None
                        } else {
                            Some(*partition_node_ix)
                        }
                    })
                    .next()
                    .unwrap_or_else(|| {
                        let weight = self.graph[original_partition_node_ix];
                        let new_partition_ix = self.graph.add_node(weight);
                        self.graph
                            .add_edge(root_node_ix, new_partition_ix, SolutionEdge::QueryPartition);

                        let mut neighbors = self.graph.neighbors(original_partition_node_ix).detach();
                        while let Some((edge_ix, node_ix)) = neighbors.next(&self.graph) {
                            let weight = self.graph[edge_ix];
                            if matches!(
                                weight,
                                SolutionEdge::RequiredBySubgraph | SolutionEdge::MutationExecutedAfter
                            ) {
                                self.graph.add_edge(new_partition_ix, node_ix, weight);
                            }
                        }

                        partitions.push((new_partition_ix, resolver_definition_id));
                        new_partition_ix
                    });

                self.graph
                    .add_edge(partition_node_ix, field_node_ix, SolutionEdge::Field);
            }
        }
    }
}
