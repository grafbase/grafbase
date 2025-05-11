use std::marker::PhantomData;

use operation::{Operation, OperationContext};
use petgraph::{Direction, Graph, visit::EdgeRef};
use schema::Schema;
use walker::Walk;

use crate::{
    Derived, QueryField, QueryFieldNode, QuerySolutionSpace,
    query::{Edge, Node},
    solution_space::{SpaceEdge, SpaceNode},
};

use super::{CrudeSolvedQuery, SteinerTreeSolution};

pub(crate) fn generate_crude_solved_query(
    schema: &Schema,
    operation: &mut Operation,
    mut query: QuerySolutionSpace<'_>,
    solution: SteinerTreeSolution,
) -> crate::Result<CrudeSolvedQuery> {
    let n = operation.data_fields.len() + operation.typename_fields.len();
    let mut graph = Graph::with_capacity(n, n);
    let root_node_ix = graph.add_node(Node::Root);

    let mut stack = Vec::new();

    for edge in query.graph.edges(query.root_node_ix) {
        match edge.weight() {
            SpaceEdge::CreateChildResolver => {
                stack.push((root_node_ix, edge.target()));
            }
            // For now assign __typename fields to the root node, they will be later be added
            // to an appropriate query partition.
            SpaceEdge::TypenameField => {
                if let SpaceNode::QueryField(QueryFieldNode {
                    id: query_field_id,
                    flags,
                }) = query.graph[edge.target()]
                {
                    if query[query_field_id].definition_id.is_none() {
                        let typename_field_ix = graph.add_node(Node::Field {
                            id: query_field_id,
                            flags,
                        });
                        graph.add_edge(root_node_ix, typename_field_ix, Edge::Field);
                    }
                }
            }
            _ => (),
        }
    }

    let mut node_requires_space_node_tuples = Vec::new();
    let mut space_edges_to_remove = Vec::new();
    let mut field_to_node = vec![root_node_ix; query.fields.len()];
    while let Some((parent_node_ix, space_node_ix)) = stack.pop() {
        let new_node_ix = match &query.graph[space_node_ix] {
            SpaceNode::Resolver(resolver) if solution.node_bitset[space_node_ix.index()] => {
                let ix = graph.add_node(Node::QueryPartition {
                    entity_definition_id: resolver.entity_definition_id,
                    resolver_definition_id: resolver.definition_id,
                });
                graph.add_edge(parent_node_ix, ix, Edge::QueryPartition);
                ix
            }
            SpaceNode::ProvidableField(providable_field) if solution.node_bitset[space_node_ix.index()] => {
                let (field_space_node_ix, &QueryFieldNode { id, flags }) = query
                    .graph
                    .edges(space_node_ix)
                    .find_map(|edge| {
                        if matches!(edge.weight(), SpaceEdge::Provides) {
                            if let SpaceNode::QueryField(field) = &query.graph[edge.target()] {
                                Some((edge.target(), field))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .unwrap();

                let field_node_ix = graph.add_node(Node::Field { id, flags });
                graph.add_edge(parent_node_ix, field_node_ix, Edge::Field);
                field_to_node[usize::from(id)] = field_node_ix;

                if let Some(Derived::Field { mapping }) = providable_field.derived {
                    let grandparent_node_ix = graph
                        .edges_directed(parent_node_ix, Direction::Incoming)
                        .find(|edge| matches!(edge.weight(), Edge::Field))
                        .map(|edge| edge.source())
                        .unwrap();
                    let derived_from_node_ix = graph
                        .edges_directed(grandparent_node_ix, Direction::Outgoing)
                        .find_map(|edge| {
                            if !matches!(edge.weight(), Edge::Field) {
                                return None;
                            }
                            let Node::Field { id, .. } = graph[edge.target()] else {
                                return None;
                            };
                            let field = &query[id];
                            if field.definition_id == Some(mapping.from_id) {
                                return Some(edge.target());
                            }
                            None
                        })
                        .unwrap_or_else(|| {
                            let parent_id = graph[parent_node_ix]
                                .as_query_field()
                                .expect("Could not be derived otherwise.");
                            let field = QueryField {
                                type_conditions: query[parent_id].type_conditions,
                                query_position: query[id].query_position,
                                response_key: Some(
                                    operation
                                        .response_keys
                                        .get_or_intern(mapping.from_id.walk(schema).name()),
                                ),
                                subgraph_key: None,
                                definition_id: Some(mapping.from_id),
                                matching_field_id: None,
                                argument_ids: Default::default(),
                                location: query[id].location,
                                flat_directive_id: query[id].flat_directive_id,
                            };
                            query.fields.push(field);
                            let ix = graph.add_node(Node::Field {
                                id: (query.fields.len() - 1).into(),
                                flags,
                            });
                            graph.add_edge(grandparent_node_ix, ix, Edge::Field);
                            ix
                        });
                    graph.add_edge(derived_from_node_ix, field_node_ix, Edge::Derived);
                }

                for edge in query.graph.edges(field_space_node_ix) {
                    match edge.weight() {
                        SpaceEdge::Requires => {
                            node_requires_space_node_tuples.push((field_node_ix, field_space_node_ix));
                        }
                        // Assigning __typename fields to the first resolver that provides the
                        // parent field. There might be multiple with shared root fields.
                        SpaceEdge::TypenameField => {
                            space_edges_to_remove.push(edge.id());
                            if let SpaceNode::QueryField(QueryFieldNode {
                                id: query_field_id,
                                flags,
                            }) = query.graph[edge.target()]
                            {
                                if query[query_field_id].definition_id.is_none() {
                                    let typename_field_ix = graph.add_node(Node::Field {
                                        id: query_field_id,
                                        flags,
                                    });
                                    graph.add_edge(field_node_ix, typename_field_ix, Edge::Field);
                                }
                            }
                        }
                        _ => (),
                    }
                }

                for edge in space_edges_to_remove.drain(..) {
                    query.graph.remove_edge(edge);
                }

                field_node_ix
            }
            SpaceNode::QueryField(QueryFieldNode {
                id: query_field_id,
                flags,
            }) if query[*query_field_id].definition_id.is_none() => {
                let typename_field_ix = graph.add_node(Node::Field {
                    id: *query_field_id,
                    flags: *flags,
                });
                graph.add_edge(parent_node_ix, typename_field_ix, Edge::Field);
                typename_field_ix
            }
            _ => continue,
        };

        if query
            .graph
            .edges(space_node_ix)
            .any(|edge| matches!(edge.weight(), SpaceEdge::Requires))
        {
            node_requires_space_node_tuples.push((new_node_ix, space_node_ix));
        }

        stack.extend(
            query
                .graph
                .edges(space_node_ix)
                .filter(|edge| {
                    matches!(
                        edge.weight(),
                        SpaceEdge::CreateChildResolver
                            | SpaceEdge::CanProvide
                            | SpaceEdge::Field
                            | SpaceEdge::TypenameField
                    )
                })
                .map(|edge| (new_node_ix, edge.target())),
        );
    }

    for (node_ix, space_node_ix) in node_requires_space_node_tuples {
        let weight = match &query.graph[space_node_ix] {
            SpaceNode::QueryField(_) => Edge::RequiredBySupergraph,
            _ => Edge::RequiredBySubgraph,
        };
        for edge in query.graph.edges(space_node_ix) {
            if !matches!(edge.weight(), SpaceEdge::Requires) {
                continue;
            }
            let SpaceNode::QueryField(field) = &query.graph[edge.target()] else {
                continue;
            };

            let required_node_ix = field_to_node[usize::from(field.id)];
            debug_assert_ne!(required_node_ix, root_node_ix);

            graph.add_edge(node_ix, required_node_ix, weight);
        }
    }
    let query = CrudeSolvedQuery {
        step: PhantomData,
        root_node_ix,
        graph,
        fields: query.fields,
        shared_type_conditions: query.shared_type_conditions,
        deduplicated_flat_sorted_executable_directives: query.deduplicated_flat_sorted_executable_directives,
    };

    tracing::debug!(
        "Partial solution:\n{}",
        query.to_pretty_dot_graph(OperationContext { schema, operation })
    );

    Ok(query)
}
