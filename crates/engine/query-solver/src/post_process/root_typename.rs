use operation::{Operation, QueryPosition};
use petgraph::visit::EdgeRef;
use schema::Schema;

use crate::{
    query::{Edge, Node},
    solve::QuerySteinerSolution,
};

pub(super) fn assign_root_typename_fields(schema: &Schema, operation: &Operation, query: &mut QuerySteinerSolution) {
    // There is always at least one field in the query, otherwise validation would fail. So
    // either there is an existing partition or there is only __typename fields and we have to
    // create one.
    let first_partition_ix = query
        .graph
        .neighbors(query.root_node_id)
        .filter(|neighor| matches!(query.graph[*neighor], Node::QueryPartition { .. }))
        .min_by_key(|partition_node_ix| {
            query
                .graph
                .neighbors(*partition_node_ix)
                .filter_map(|neighbor| match query.graph[neighbor] {
                    Node::Field(node) => query[node.id].query_position,
                    _ => None,
                })
                .min()
                .unwrap_or(QueryPosition::MAX)
        })
        .unwrap_or_else(|| {
            let ix = query.graph.add_node(Node::QueryPartition {
                entity_definition_id: operation.root_object_id.into(),
                resolver_definition_id: schema.subgraphs.introspection.resolver_definition_id,
            });
            query.graph.add_edge(query.root_node_id, ix, Edge::QueryPartition);
            ix
        });
    let typename_fields = query
        .graph
        .edges(query.root_node_id)
        .filter_map(|edge| match edge.weight() {
            Edge::Field => match query.graph[edge.target()] {
                Node::Field(node) if query[node.id].definition_id.is_none() => Some(edge.target()),
                _ => None,
            },
            _ => None,
        })
        .collect::<Vec<_>>();
    for ix in typename_fields {
        if let Some(id) = query.graph.find_edge(query.root_node_id, ix) {
            query.graph.remove_edge(id);
        }
        query.graph.add_edge(first_partition_ix, ix, Edge::Field);
    }
}
