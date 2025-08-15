use fixedbitset::FixedBitSet;
use operation::{Operation, OperationContext};
use petgraph::{Direction, Graph, visit::EdgeRef};
use schema::Schema;
use walker::Walk;

use crate::{
    Derive, QueryField, QueryFieldNode,
    query::{Edge, Node},
    solution_space::{SpaceEdge, SpaceNode},
    solve::{input::SteinerInput, steiner_tree::SteinerTree},
};

use super::CrudeSolvedQuery;

pub(crate) struct Solution<'schema> {
    pub input: SteinerInput<'schema>,
    pub steiner_tree: SteinerTree,
}

impl Solution<'_> {
    pub fn into_query(self, schema: &Schema, operation: &mut Operation) -> crate::Result<CrudeSolvedQuery> {
        let Solution {
            input:
                SteinerInput {
                    mut space,
                    map: steiner_input_map,
                    ..
                },
            steiner_tree,
        } = self;
        let n = operation.data_fields.len() + operation.typename_fields.len();
        let mut graph = Graph::with_capacity(n, n);
        let root_node_id = graph.add_node(Node::Root);

        let excluded_space_edges = {
            let mut excluded = FixedBitSet::with_capacity(space.graph.edge_count());
            let mut stack = Vec::new();
            for steiner_edge_ix in steiner_tree.edges.zeroes() {
                let space_edge_id = steiner_input_map.edge_id_to_space_edge_id[steiner_edge_ix];
                excluded.insert(space_edge_id.index());

                let (src, dst) = space.graph.edge_endpoints(space_edge_id).unwrap();
                stack.push(src);

                // If the parent node has no other included child edges, we can remove it and check
                // the next one.
                while let Some(node_id) = stack.pop() {
                    let has_any_included_outgoing_edges =
                        space.graph.edges_directed(node_id, Direction::Outgoing).any(|edge| {
                            matches!(edge.weight(), SpaceEdge::CreateChildResolver | SpaceEdge::CanProvide)
                                && !excluded[edge.id().index()]
                        });
                    if has_any_included_outgoing_edges {
                        continue;
                    }
                    stack.extend(
                        space
                            .graph
                            .edges_directed(node_id, Direction::Incoming)
                            .filter(|edge| {
                                matches!(edge.weight(), SpaceEdge::CreateChildResolver | SpaceEdge::CanProvide)
                            })
                            .map(|edge| {
                                excluded.insert(edge.id().index());
                                edge.source()
                            }),
                    );
                }
            }
            excluded
        };

        let mut stack = Vec::new();
        for edge in space.graph.edges(space.root_node_id) {
            match edge.weight() {
                SpaceEdge::CreateChildResolver if !excluded_space_edges[edge.id().index()] => {
                    stack.push((root_node_id, edge.source(), edge.target()));
                }
                // For now assign __typename fields to the root node, they will be later be added
                // to an appropriate query partition.
                SpaceEdge::TypenameField => {
                    if let SpaceNode::QueryField(QueryFieldNode {
                        id: query_field_id,
                        flags,
                    }) = space.graph[edge.target()]
                        && space[query_field_id].definition_id.is_none()
                    {
                        let typename_field_ix = graph.add_node(Node::Field {
                            id: query_field_id,
                            flags,
                        });
                        graph.add_edge(root_node_id, typename_field_ix, Edge::Field);
                    }
                }
                _ => (),
            }
        }

        let mut node_requires_space_node_tuples = Vec::new();
        let mut space_edges_to_remove = Vec::new();
        // FIXME: doesn't take into account shared roots.
        let mut field_to_node = vec![root_node_id; space.fields.len()];
        while let Some((parent_node_id, space_parent_node_id, space_node_id)) = stack.pop() {
            let new_node_id = match &space.graph[space_node_id] {
                SpaceNode::Resolver(resolver) => {
                    let id = graph.add_node(Node::QueryPartition {
                        entity_definition_id: resolver.entity_definition_id,
                        resolver_definition_id: resolver.definition_id,
                    });
                    graph.add_edge(parent_node_id, id, Edge::QueryPartition);
                    id
                }
                SpaceNode::ProvidableField(providable_field) => {
                    let Some((field_space_node_id, &QueryFieldNode { id, flags })) = space
                        .graph
                        .edges_directed(space_node_id, Direction::Outgoing)
                        .find_map(|edge| {
                            if matches!(edge.weight(), SpaceEdge::Provides) && !excluded_space_edges[edge.id().index()]
                            {
                                if let SpaceNode::QueryField(field) = &space.graph[edge.target()] {
                                    Some((edge.target(), field))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                    else {
                        continue;
                    };

                    let field_node_id = graph.add_node(Node::Field { id, flags });
                    graph.add_edge(parent_node_id, field_node_id, Edge::Field);
                    field_to_node[usize::from(id)] = field_node_id;

                    // FIXME: Move this logic to the solution space.
                    if let Some(derive) = providable_field.derive {
                        let derive_root = match derive {
                            Derive::Root { id } => id,
                            _ => space.graph[space_parent_node_id]
                                .as_providable_field()
                                .and_then(|field| field.derive)
                                .and_then(Derive::into_root)
                                .unwrap(),
                        }
                        .walk(schema);

                        let (type_conditions, source_node_id) = if let Some(batch_field_id) = derive_root.batch_field_id
                        {
                            let source_node_id = if matches!(derive, Derive::Root { .. }) {
                                let source_node_id = graph
                                    .edges_directed(parent_node_id, Direction::Outgoing)
                                    .find_map(|edge| {
                                        if !matches!(edge.weight(), Edge::Field) {
                                            return None;
                                        }
                                        let Node::Field { id, .. } = graph[edge.target()] else {
                                            return None;
                                        };
                                        let field = &space[id];
                                        if field.definition_id == Some(batch_field_id) {
                                            return Some(edge.target());
                                        }
                                        None
                                    })
                                    .unwrap_or_else(|| {
                                        let field = QueryField {
                                            type_conditions: space[id].type_conditions,
                                            query_position: None,
                                            response_key: Some(
                                                operation
                                                    .response_keys
                                                    .get_or_intern(batch_field_id.walk(schema).name()),
                                            ),
                                            subgraph_key: None,
                                            definition_id: Some(batch_field_id),
                                            matching_field_id: None,
                                            argument_ids: Default::default(),
                                            location: space[id].location,
                                            flat_directive_id: None,
                                        };
                                        space.fields.push(field);
                                        let node_id = graph.add_node(Node::Field {
                                            id: (space.fields.len() - 1).into(),
                                            flags,
                                        });
                                        graph.add_edge(parent_node_id, node_id, Edge::Field);
                                        node_id
                                    });
                                graph.add_edge(source_node_id, field_node_id, Edge::Derive);
                                source_node_id
                            } else {
                                let grandparent_node_id = graph
                                    .edges_directed(parent_node_id, Direction::Incoming)
                                    .find(|edge| matches!(edge.weight(), Edge::Field))
                                    .map(|edge| edge.source())
                                    .unwrap();
                                graph
                                    .edges_directed(grandparent_node_id, Direction::Outgoing)
                                    .find_map(|edge| {
                                        if !matches!(edge.weight(), Edge::Field) {
                                            return None;
                                        }
                                        let Node::Field { id, .. } = graph[edge.target()] else {
                                            return None;
                                        };
                                        let field = &space[id];
                                        if field.definition_id == Some(batch_field_id) {
                                            return Some(edge.target());
                                        }
                                        None
                                    })
                                    .expect("Batch field id should be present")
                            };
                            (Default::default(), source_node_id)
                        } else {
                            let parent_query_field_id = graph[parent_node_id]
                                .as_query_field()
                                .expect("Could not be derived otherwise.");
                            let grandparent_node_id = graph
                                .edges_directed(parent_node_id, Direction::Incoming)
                                .find(|edge| matches!(edge.weight(), Edge::Field))
                                .map(|edge| edge.source())
                                .unwrap();
                            (space[parent_query_field_id].type_conditions, grandparent_node_id)
                        };
                        match derive {
                            Derive::Field { from_id } => {
                                let derived_from_node_id = graph
                                    .edges_directed(source_node_id, Direction::Outgoing)
                                    .find_map(|edge| {
                                        if !matches!(edge.weight(), Edge::Field) {
                                            return None;
                                        }
                                        let Node::Field { id, .. } = graph[edge.target()] else {
                                            return None;
                                        };
                                        let field = &space[id];
                                        if field.definition_id == Some(from_id) {
                                            return Some(edge.target());
                                        }
                                        None
                                    })
                                    .unwrap_or_else(|| {
                                        let field = QueryField {
                                            type_conditions,
                                            query_position: None,
                                            response_key: Some(
                                                operation.response_keys.get_or_intern(from_id.walk(schema).name()),
                                            ),
                                            subgraph_key: None,
                                            definition_id: Some(from_id),
                                            matching_field_id: None,
                                            argument_ids: Default::default(),
                                            location: space[id].location,
                                            flat_directive_id: None,
                                        };
                                        space.fields.push(field);
                                        let ix = graph.add_node(Node::Field {
                                            id: (space.fields.len() - 1).into(),
                                            flags,
                                        });
                                        graph.add_edge(source_node_id, ix, Edge::Field);
                                        ix
                                    });
                                graph.add_edge(derived_from_node_id, field_node_id, Edge::Derive);
                            }
                            Derive::ScalarAsField => {
                                graph.add_edge(source_node_id, field_node_id, Edge::Derive);
                            }
                            Derive::Root { .. } => {}
                        }
                    }

                    for edge in space.graph.edges(field_space_node_id) {
                        match edge.weight() {
                            SpaceEdge::Requires => {
                                node_requires_space_node_tuples.push((field_node_id, field_space_node_id));
                            }
                            // Assigning __typename fields to the first resolver that provides the
                            // parent field. There might be multiple with shared root fields.
                            SpaceEdge::TypenameField => {
                                space_edges_to_remove.push(edge.id());
                                if let SpaceNode::QueryField(QueryFieldNode {
                                    id: query_field_id,
                                    flags,
                                }) = space.graph[edge.target()]
                                    && space[query_field_id].definition_id.is_none()
                                {
                                    let typename_field_ix = graph.add_node(Node::Field {
                                        id: query_field_id,
                                        flags,
                                    });
                                    graph.add_edge(field_node_id, typename_field_ix, Edge::Field);
                                }
                            }
                            _ => (),
                        }
                    }

                    for edge in space_edges_to_remove.drain(..) {
                        space.graph.remove_edge(edge);
                    }

                    field_node_id
                }
                SpaceNode::QueryField(QueryFieldNode {
                    id: query_field_id,
                    flags,
                }) if space[*query_field_id].definition_id.is_none() => {
                    let typename_field_ix = graph.add_node(Node::Field {
                        id: *query_field_id,
                        flags: *flags,
                    });
                    graph.add_edge(parent_node_id, typename_field_ix, Edge::Field);
                    typename_field_ix
                }
                _ => continue,
            };

            if space
                .graph
                .edges(space_node_id)
                .any(|edge| matches!(edge.weight(), SpaceEdge::Requires))
            {
                node_requires_space_node_tuples.push((new_node_id, space_node_id));
            }

            stack.extend(
                space
                    .graph
                    .edges(space_node_id)
                    .filter(|edge| {
                        matches!(
                            edge.weight(),
                            SpaceEdge::CreateChildResolver
                                | SpaceEdge::CanProvide
                                | SpaceEdge::Field
                                | SpaceEdge::TypenameField
                        ) && !excluded_space_edges[edge.id().index()]
                    })
                    .map(|edge| (new_node_id, edge.source(), edge.target())),
            );
        }

        for (node_id, space_node_id) in node_requires_space_node_tuples {
            let weight = match &space.graph[space_node_id] {
                SpaceNode::QueryField(_) => Edge::RequiredBySupergraph,
                _ => Edge::RequiredBySubgraph,
            };
            for edge in space.graph.edges(space_node_id) {
                if !matches!(edge.weight(), SpaceEdge::Requires) {
                    continue;
                }
                let SpaceNode::QueryField(field) = &space.graph[edge.target()] else {
                    continue;
                };

                let required_node_ix = field_to_node[usize::from(field.id)];
                debug_assert_ne!(required_node_ix, root_node_id);

                graph.add_edge(node_id, required_node_ix, weight);
            }
        }

        let query = CrudeSolvedQuery {
            step: crate::query::steps::SteinerTreeSolution,
            root_node_id,
            graph,
            fields: space.fields,
            shared_type_conditions: space.shared_type_conditions,
            deduplicated_flat_sorted_executable_directives: space.deduplicated_flat_sorted_executable_directives,
        };

        tracing::debug!(
            "Partial solution:\n{}",
            query.to_pretty_dot_graph(OperationContext { schema, operation })
        );

        Ok(query)
    }
}
