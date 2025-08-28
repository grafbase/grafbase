use operation::QueryPosition;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeFiltered;
use petgraph::visit::EdgeRef;
use schema::ResolverDefinitionId;

use crate::{
    query::{Edge, Node},
    solve::QuerySteinerSolution,
};

pub(super) fn split_query_partition_dependency_cycles(
    query: &mut QuerySteinerSolution,
    starting_nodes: Vec<NodeIndex>,
) {
    struct Field {
        position: Option<QueryPosition>,
        original_partition_node_ix: NodeIndex,
        resolver_definition_id: ResolverDefinitionId,
        query_field_node_ix: NodeIndex,
    }
    let mut stack = starting_nodes;
    let mut nested_partitions = Vec::new();
    let mut nested_patitions_fields = Vec::new();

    while let Some(root_node_ix) = stack.pop() {
        debug_assert!(nested_partitions.is_empty() && nested_patitions_fields.is_empty());
        for edge in query.graph.edges(root_node_ix) {
            // Ignoring requirements among other things.
            if !matches!(edge.weight(), Edge::QueryPartition | Edge::Field) {
                continue;
            }
            let node_ix = edge.target();
            match query.graph[node_ix] {
                Node::QueryPartition {
                    resolver_definition_id, ..
                } => {
                    nested_partitions.push((node_ix, resolver_definition_id));
                    for second_degree_edge in query.graph.edges(node_ix) {
                        // Ignoring requirements among other things.
                        if !matches!(second_degree_edge.weight(), Edge::Field) {
                            continue;
                        }
                        let second_degree_node_ix = second_degree_edge.target();
                        if let Node::Field(node) = query.graph[second_degree_node_ix] {
                            nested_patitions_fields.push(Field {
                                position: query[node.id].query_position,
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

        if nested_patitions_fields.is_empty() {
            continue;
        }

        nested_patitions_fields.sort_unstable_by(|a, b| a.position.cmp(&b.position));

        // Removing edges to the parent partitions
        for field in &nested_patitions_fields {
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
        } in nested_patitions_fields.drain(..)
        {
            let partition_node_ix = nested_partitions
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
                        .any(|partition_query_field_node_ix| {
                            let graph = EdgeFiltered::from_fn(&query.graph, |edge| {
                                matches!(edge.weight(), Edge::RequiredBySubgraph)
                            });
                            petgraph::algo::has_path_connecting(
                                &graph,
                                partition_query_field_node_ix,
                                query_field_node_ix,
                                None,
                            ) || petgraph::algo::has_path_connecting(
                                &graph,
                                query_field_node_ix,
                                partition_query_field_node_ix,
                                None,
                            )
                        });
                    if is_connected { None } else { Some(*partition_node_ix) }
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

                    nested_partitions.push((new_partition_ix, resolver_definition_id));
                    new_partition_ix
                });

            query
                .graph
                .add_edge(partition_node_ix, query_field_node_ix, Edge::Field);
        }

        nested_partitions.clear();
    }
}
