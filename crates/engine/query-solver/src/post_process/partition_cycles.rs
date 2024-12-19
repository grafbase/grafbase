use operation::QueryPosition;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeFiltered;
use petgraph::visit::EdgeRef;
use schema::ResolverDefinitionId;

use crate::{
    query::{Edge, Node},
    solve::CrudeSolvedQuery,
};

pub(super) fn split_query_partition_dependency_cycles(query: &mut CrudeSolvedQuery, starting_nodes: Vec<NodeIndex>) {
    struct Field {
        position: Option<QueryPosition>,
        original_partition_node_ix: NodeIndex,
        resolver_definition_id: ResolverDefinitionId,
        query_field_node_ix: NodeIndex,
    }
    let mut partition_root_fields = Vec::new();
    let mut stack = starting_nodes;
    let mut partitions = Vec::new();

    while let Some(root_node_ix) = stack.pop() {
        debug_assert!(partition_root_fields.is_empty());
        for node_ix in query.graph.neighbors(root_node_ix) {
            match query.graph[node_ix] {
                Node::QueryPartition {
                    resolver_definition_id, ..
                } => {
                    partitions.push((node_ix, resolver_definition_id));
                    for second_degree_node_ix in query.graph.neighbors(node_ix) {
                        if let Node::Field { id, .. } = query.graph[second_degree_node_ix] {
                            partition_root_fields.push(Field {
                                position: query[id].query_position,
                                original_partition_node_ix: node_ix,
                                resolver_definition_id,
                                query_field_node_ix: second_degree_node_ix,
                            });
                            stack.push(second_degree_node_ix);
                        }
                    }
                }
                Node::Field { .. } => {
                    stack.push(node_ix);
                }
                _ => (),
            }
        }

        if partition_root_fields.is_empty() {
            continue;
        }

        partition_root_fields.sort_unstable_by(|a, b| a.position.cmp(&b.position));

        // Removing edges to the parent partitions
        for field in &partition_root_fields {
            if let Some(id) = query
                .graph
                .find_edge(field.original_partition_node_ix, field.query_field_node_ix)
            {
                query.graph.remove_edge(id);
            }
        }

        for Field {
            original_partition_node_ix,
            resolver_definition_id,
            query_field_node_ix,
            ..
        } in partition_root_fields.drain(..)
        {
            let partition_node_ix = partitions
                .iter()
                .filter(|(_, id)| *id == resolver_definition_id)
                .filter_map(|(partition_node_ix, _)| {
                    let is_connected = query
                        .graph
                        .edges(*partition_node_ix)
                        .filter_map(|edge| {
                            if matches!(edge.weight(), Edge::Field) {
                                Some(edge.target())
                            } else {
                                None
                            }
                        })
                        .any(|partition_field_node_ix| {
                            petgraph::algo::has_path_connecting(
                                &EdgeFiltered::from_fn(&query.graph, |edge| {
                                    matches!(edge.weight(), Edge::RequiredBySubgraph)
                                }),
                                partition_field_node_ix,
                                query_field_node_ix,
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
                    let weight = query.graph[original_partition_node_ix];
                    let new_partition_ix = query.graph.add_node(weight);
                    query
                        .graph
                        .add_edge(root_node_ix, new_partition_ix, Edge::QueryPartition);

                    let mut neighbors = query.graph.neighbors(original_partition_node_ix).detach();
                    while let Some((edge_ix, node_ix)) = neighbors.next(&query.graph) {
                        let weight = query.graph[edge_ix];
                        if matches!(weight, Edge::RequiredBySubgraph | Edge::MutationExecutedAfter) {
                            query.graph.add_edge(new_partition_ix, node_ix, weight);
                        }
                    }

                    partitions.push((new_partition_ix, resolver_definition_id));
                    new_partition_ix
                });

            query
                .graph
                .add_edge(partition_node_ix, query_field_node_ix, Edge::Field);
        }

        partitions.clear();
    }
}
