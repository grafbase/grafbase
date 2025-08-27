use std::collections::VecDeque;

use itertools::Itertools;
use operation::QueryPosition;
use petgraph::graph::NodeIndex;
use schema::ResolverDefinitionId;

use crate::{
    query::{Edge, Node},
    solve::QuerySteinerSolution,
};

pub(super) fn ensure_mutation_execution_order(query: &mut QuerySteinerSolution) -> Vec<NodeIndex> {
    struct Field {
        position: Option<QueryPosition>,
        original_partition_node_ix: NodeIndex,
        resolver_definition_id: ResolverDefinitionId,
        field_node_ix: NodeIndex,
    }

    let mut selection_set = Vec::new();
    for partition_node_ix in query.graph.neighbors(query.root_node_id) {
        if let Node::QueryPartition {
            resolver_definition_id, ..
        } = query.graph[partition_node_ix]
        {
            for field_node_ix in query.graph.neighbors(partition_node_ix) {
                if let Node::Field(node) = query.graph[field_node_ix] {
                    selection_set.push(Field {
                        position: query[node.id].query_position,
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
                if let Some(id) = query.graph.find_edge(original_partition_node_ix, field_node_ix) {
                    query.graph.remove_edge(id);
                }
                query
                    .graph
                    .add_edge(*last_partition_node_ix, field_node_ix, Edge::Field);
            }
        }

        // If original partition is already used, create a new one.
        if partitions.iter().any(|(id, _)| *id == original_partition_node_ix) {
            let weight = query.graph[original_partition_node_ix];
            let new_partition_ix = query.graph.add_node(weight);
            query
                .graph
                .add_edge(query.root_node_id, new_partition_ix, Edge::QueryPartition);
            partitions.push((new_partition_ix, resolver_definition_id));

            if let Some(id) = query.graph.find_edge(original_partition_node_ix, field_node_ix) {
                query.graph.remove_edge(id);
            }
            query.graph.add_edge(new_partition_ix, field_node_ix, Edge::Field);
        } else {
            partitions.push((original_partition_node_ix, resolver_definition_id));
        }

        root_fields.push(field_node_ix);
    }

    for ((partition1_ix, _), (partition2_ix, _)) in partitions.into_iter().tuple_windows() {
        query
            .graph
            .add_edge(partition2_ix, partition1_ix, Edge::MutationExecutedAfter);
    }

    root_fields
}
