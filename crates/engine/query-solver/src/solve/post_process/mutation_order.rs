use std::collections::VecDeque;

use itertools::Itertools;
use operation::QueryPosition;
use petgraph::graph::NodeIndex;
use schema::ResolverDefinitionId;

use crate::query::{SolutionEdge, SolutionNode};

use super::SolvedQueryWithoutPostProcessing;

impl SolvedQueryWithoutPostProcessing<'_> {
    pub(super) fn ensure_mutation_execution_order(&mut self) -> Vec<NodeIndex> {
        struct Field {
            position: Option<QueryPosition>,
            original_partition_node_ix: NodeIndex,
            resolver_definition_id: ResolverDefinitionId,
            field_node_ix: NodeIndex,
        }

        let mut selection_set = Vec::new();
        for partition_node_ix in self.graph.neighbors(self.root_node_ix) {
            if let SolutionNode::QueryPartition {
                resolver_definition_id, ..
            } = self.graph[partition_node_ix]
            {
                for field_node_ix in self.graph.neighbors(partition_node_ix) {
                    if let SolutionNode::Field { id, .. } = self.graph[field_node_ix] {
                        selection_set.push(Field {
                            position: self[id].query_position,
                            original_partition_node_ix: partition_node_ix,
                            resolver_definition_id,
                            field_node_ix,
                        });
                    }
                }
            }
        }

        selection_set.sort_unstable_by(|a, b| a.position.cmp(&b.position));
        let selection_set = VecDeque::from(selection_set);

        let mut partitions = Vec::new();
        let mut root_fields = Vec::with_capacity(selection_set.len());

        for Field {
            original_partition_node_ix,
            resolver_definition_id,
            field_node_ix,
            ..
        } in selection_set
        {
            if let Some((last_partition_node_ix, _)) = partitions
                .last()
                .filter(|(_, last_resolver_definition_id)| *last_resolver_definition_id == resolver_definition_id)
            {
                if original_partition_node_ix == *last_partition_node_ix {
                    continue;
                } else {
                    if let Some(id) = self.graph.find_edge(original_partition_node_ix, field_node_ix) {
                        self.graph.remove_edge(id);
                    }
                    self.graph
                        .add_edge(*last_partition_node_ix, field_node_ix, SolutionEdge::Field);
                }
            }

            // If original partition is already used, create a new one.
            if partitions.iter().any(|(id, _)| *id == original_partition_node_ix) {
                let weight = self.graph[original_partition_node_ix];
                let new_partition_ix = self.graph.add_node(weight);
                self.0
                    .graph
                    .add_edge(self.0.root_node_ix, new_partition_ix, SolutionEdge::QueryPartition);
                partitions.push((new_partition_ix, resolver_definition_id));

                if let Some(id) = self.graph.find_edge(original_partition_node_ix, field_node_ix) {
                    self.graph.remove_edge(id);
                }
                self.graph
                    .add_edge(new_partition_ix, field_node_ix, SolutionEdge::Field);
            } else {
                partitions.push((original_partition_node_ix, resolver_definition_id));
            }

            root_fields.push(field_node_ix);
        }

        for ((partition1_ix, _), (partition2_ix, _)) in partitions.into_iter().tuple_windows() {
            self.graph
                .add_edge(partition2_ix, partition1_ix, SolutionEdge::MutationExecutedAfter);
        }

        root_fields
    }
}
