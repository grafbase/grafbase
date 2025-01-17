use std::marker::PhantomData;

use operation::{Operation, OperationContext};
use petgraph::{visit::EdgeRef, Graph};
use schema::Schema;

use crate::{
    query::{Edge, Node},
    solution_space::{SpaceEdge, SpaceNode},
    QuerySelectionSet, QuerySolutionSpace, QuerySolutionSpaceSelectionSet,
};

use super::{CrudeSolvedQuery, SteinerTreeSolution};

pub(crate) fn generate_crude_solved_query(
    schema: &Schema,
    operation: &Operation,
    mut query_space: QuerySolutionSpace<'_>,
    solution: SteinerTreeSolution,
) -> crate::Result<CrudeSolvedQuery> {
    let n = operation.data_fields.len() + operation.typename_fields.len();
    let mut graph = Graph::with_capacity(n, n);
    let root_node_ix = graph.add_node(Node::Root);

    let mut stack = Vec::new();

    for edge in query_space.graph.edges(query_space.root_node_ix) {
        if matches!(edge.weight(), SpaceEdge::CreateChildResolver) {
            stack.push((root_node_ix, edge.target()));
        }
    }

    let mut nodes_with_dependencies = Vec::new();
    let mut edges_to_remove = Vec::new();

    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    struct SpaceToSolutionNode {
        // SpaceNode MUST be first for sort order.
        space_node_ix: petgraph::stable_graph::NodeIndex,
        solution_node_ix: petgraph::graph::NodeIndex,
    }
    let mut space_to_solution_node = Vec::with_capacity(query_space.fields.len() + query_space.selection_sets.len());

    while let Some((parent_solution_node_ix, space_node_ix)) = stack.pop() {
        let new_solution_node_ix = match &query_space.graph[space_node_ix] {
            SpaceNode::Resolver(resolver) if solution.node_bitset[space_node_ix.index()] => {
                let ix = graph.add_node(Node::QueryPartition {
                    entity_definition_id: resolver.entity_definition_id,
                    resolver_definition_id: resolver.definition_id,
                });
                graph.add_edge(parent_solution_node_ix, ix, Edge::QueryPartition);
                ix
            }
            SpaceNode::ProvidableField(_) if solution.node_bitset[space_node_ix.index()] => {
                let (query_field_space_node_ix, id) = query_space
                    .graph
                    .edges(space_node_ix)
                    .find_map(|edge| {
                        if matches!(edge.weight(), SpaceEdge::Provides) {
                            if let SpaceNode::QueryField { id, .. } = query_space.graph[edge.target()] {
                                Some((edge.target(), id))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .unwrap();

                let field_solution_node_ix = graph.add_node(Node::Field { id });
                graph.add_edge(parent_solution_node_ix, field_solution_node_ix, Edge::Field);
                space_to_solution_node.push(SpaceToSolutionNode {
                    space_node_ix: query_field_space_node_ix,
                    solution_node_ix: field_solution_node_ix,
                });

                if query_space.graph.edges(query_field_space_node_ix).any(|edge| {
                    matches!(
                        edge.weight(),
                        SpaceEdge::RequiredBySubgraph | SpaceEdge::RequiredBySupergraph
                    )
                }) {
                    nodes_with_dependencies.push((field_solution_node_ix, space_node_ix));
                }

                for edge in edges_to_remove.drain(..) {
                    query_space.graph.remove_edge(edge);
                }

                field_solution_node_ix
            }
            SpaceNode::Typename { .. } if solution.node_bitset[space_node_ix.index()] => {
                let typename_field_solution_node_ix = graph.add_node(Node::Typename);
                graph.add_edge(parent_solution_node_ix, typename_field_solution_node_ix, Edge::Field);
                space_to_solution_node.push(SpaceToSolutionNode {
                    space_node_ix,
                    solution_node_ix: typename_field_solution_node_ix,
                });
                typename_field_solution_node_ix
            }
            _ => continue,
        };

        if query_space.graph.edges(space_node_ix).any(|edge| {
            matches!(
                edge.weight(),
                SpaceEdge::RequiredBySubgraph | SpaceEdge::RequiredBySupergraph
            )
        }) {
            nodes_with_dependencies.push((new_solution_node_ix, space_node_ix));
        }

        stack.extend(
            query_space
                .graph
                .edges(space_node_ix)
                .filter(|edge| {
                    matches!(
                        edge.weight(),
                        SpaceEdge::CreateChildResolver | SpaceEdge::CanProvide | SpaceEdge::ProvidesTypename
                    )
                })
                .map(|edge| (new_solution_node_ix, edge.target())),
        );
    }

    space_to_solution_node.sort_unstable();
    for (new_solution_node_ix, space_node_ix) in nodes_with_dependencies {
        for edge in query_space.graph.edges(space_node_ix) {
            let weight = match edge.weight() {
                SpaceEdge::RequiredBySupergraph => Edge::RequiredBySupergraph,
                SpaceEdge::RequiredBySubgraph => Edge::RequiredBySubgraph,
                _ => continue,
            };
            if let Ok(i) = space_to_solution_node.binary_search_by(|probe| probe.space_node_ix.cmp(&edge.target())) {
                graph.add_edge(new_solution_node_ix, space_to_solution_node[i].solution_node_ix, weight);
            } else {
                tracing::warn!("Missing requirement in Solution?");
            }
        }
    }
    let query = CrudeSolvedQuery {
        step: PhantomData,
        root_node_ix,
        graph,
        root_selection_set_id: query_space.root_selection_set_id,
        selection_sets: query_space
            .selection_sets
            .into_iter()
            .map(
                |QuerySolutionSpaceSelectionSet {
                     output_type_id,
                     typename_fields,
                     ..
                 }| QuerySelectionSet {
                    output_type_id,
                    typename_fields,
                },
            )
            .collect(),
        fields: query_space.fields,
        shared_type_conditions: query_space.shared_type_conditions,
        deduplicated_flat_sorted_executable_directives: query_space.deduplicated_flat_sorted_executable_directives,
    };

    tracing::debug!(
        "Partial solution:\n{}",
        query.to_pretty_dot_graph(OperationContext { schema, operation })
    );

    Ok(query)
}
