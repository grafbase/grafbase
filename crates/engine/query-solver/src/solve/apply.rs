use std::marker::PhantomData;

use operation::{Operation, OperationContext};
use petgraph::{visit::EdgeRef, Graph};
use schema::Schema;

use crate::{
    query::{Edge, Node},
    solution_space::{SpaceEdge, SpaceNode},
    QueryFieldNode, QuerySolutionSpace,
};

use super::{CrudeSolvedQuery, SteinerTreeSolution};

pub(crate) fn generate_crude_solved_query(
    schema: &Schema,
    operation: &Operation,
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

    let mut nodes_with_dependencies = Vec::new();
    let mut edges_to_remove = Vec::new();
    let mut field_to_solution_node = vec![root_node_ix; query.fields.len()];
    while let Some((parent_solution_node_ix, node_ix)) = stack.pop() {
        let new_solution_node_ix = match &query.graph[node_ix] {
            SpaceNode::Resolver(resolver) if solution.node_bitset[node_ix.index()] => {
                let ix = graph.add_node(Node::QueryPartition {
                    entity_definition_id: resolver.entity_definition_id,
                    resolver_definition_id: resolver.definition_id,
                });
                graph.add_edge(parent_solution_node_ix, ix, Edge::QueryPartition);
                ix
            }
            SpaceNode::ProvidableField(_) if solution.node_bitset[node_ix.index()] => {
                let (field_node_ix, field) = query
                    .graph
                    .edges(node_ix)
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

                let field_solution_node_ix = graph.add_node(Node::Field {
                    id: field.id,
                    flags: field.flags,
                });
                graph.add_edge(parent_solution_node_ix, field_solution_node_ix, Edge::Field);
                field_to_solution_node[usize::from(field.id)] = field_solution_node_ix;

                for edge in query.graph.edges(field_node_ix) {
                    match edge.weight() {
                        SpaceEdge::Requires => {
                            nodes_with_dependencies.push((field_solution_node_ix, field_node_ix));
                        }
                        // Assigning __typename fields to the first resolver that provides the
                        // parent field. There might be multiple with shared root fields.
                        SpaceEdge::TypenameField => {
                            edges_to_remove.push(edge.id());
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
                                    graph.add_edge(field_solution_node_ix, typename_field_ix, Edge::Field);
                                }
                            }
                        }
                        _ => (),
                    }
                }

                for edge in edges_to_remove.drain(..) {
                    query.graph.remove_edge(edge);
                }

                field_solution_node_ix
            }
            SpaceNode::QueryField(QueryFieldNode {
                id: query_field_id,
                flags,
            }) if query[*query_field_id].definition_id.is_none() => {
                let typename_field_ix = graph.add_node(Node::Field {
                    id: *query_field_id,
                    flags: *flags,
                });
                graph.add_edge(parent_solution_node_ix, typename_field_ix, Edge::Field);
                typename_field_ix
            }
            _ => continue,
        };

        if query
            .graph
            .edges(node_ix)
            .any(|edge| matches!(edge.weight(), SpaceEdge::Requires))
        {
            nodes_with_dependencies.push((new_solution_node_ix, node_ix));
        }

        stack.extend(
            query
                .graph
                .edges(node_ix)
                .filter(|edge| {
                    matches!(
                        edge.weight(),
                        SpaceEdge::CreateChildResolver
                            | SpaceEdge::CanProvide
                            | SpaceEdge::Field
                            | SpaceEdge::TypenameField
                    )
                })
                .map(|edge| (new_solution_node_ix, edge.target())),
        );
    }

    for (new_solution_node_ix, node_ix) in nodes_with_dependencies {
        let weight = match &query.graph[node_ix] {
            SpaceNode::QueryField(_) => Edge::RequiredBySupergraph,
            _ => Edge::RequiredBySubgraph,
        };
        for edge in query.graph.edges(node_ix) {
            if !matches!(edge.weight(), SpaceEdge::Requires) {
                continue;
            }
            let SpaceNode::QueryField(field) = &query.graph[edge.target()] else {
                continue;
            };

            let dependency = field_to_solution_node[usize::from(field.id)];
            debug_assert_ne!(dependency, root_node_ix);

            graph.add_edge(new_solution_node_ix, dependency, weight);
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
