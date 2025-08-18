use fixedbitset::FixedBitSet;
use id_newtypes::IdToMany;
use operation::{Operation, OperationContext};
use petgraph::{
    Direction, Graph,
    visit::{EdgeIndexable, EdgeRef},
};
use schema::Schema;
use walker::Walk;

use crate::{
    Derive, QueryField, QueryFieldNode, SpaceNodeId,
    query::{Edge, Node},
    solution_space::{SpaceEdge, SpaceNode},
    solve::{
        input::{SteinerInput, SteinerNodeId},
        steiner_tree::SteinerTree,
    },
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
                    space_node_is_terminal,
                    ..
                },
            steiner_tree,
        } = self;
        let n = operation.data_fields.len() + operation.typename_fields.len();
        let mut graph = Graph::with_capacity(n, n);
        let root_node_id = graph.add_node(Node::Root);

        let included_space_edges = {
            let mut stack = space_node_is_terminal
                .into_ones()
                .map(SpaceNodeId::new)
                .collect::<Vec<_>>();
            let mut included = FixedBitSet::with_capacity(space.graph.edge_bound());
            while let Some(space_node_id) = stack.pop() {
                for space_edge in space.graph.edges_directed(space_node_id, Direction::Incoming) {
                    if matches!(
                        space_edge.weight(),
                        SpaceEdge::CreateChildResolver | SpaceEdge::CanProvide | SpaceEdge::Provides
                    ) && steiner_input_map
                        .space_edge_id_to_edge_id
                        .get(&space_edge.id())
                        .map(|id| steiner_tree.edges[id.index()])
                        .unwrap_or(true)
                    {
                        included.insert(space_edge.id().index());
                        stack.push(space_edge.source());
                    }
                }
            }

            included
        };

        let mut stack = Vec::new();
        for edge in space.graph.edges(space.root_node_id) {
            match edge.weight() {
                SpaceEdge::CreateChildResolver if included_space_edges[edge.id().index()] => {
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
                        let typename_field_id = graph.add_node(Node::Field {
                            id: query_field_id,
                            flags,
                        });
                        graph.add_edge(root_node_id, typename_field_id, Edge::Field);
                    }
                }
                _ => (),
            }
        }

        struct RequiredEdge {
            source_node_id: SteinerNodeId,
            weight: Edge,
            target_space_node_id: SpaceNodeId,
        }

        let mut required_edges = Vec::<RequiredEdge>::new();
        let mut space_edges_to_remove = Vec::new();
        let mut query_field_to_node = Vec::with_capacity(space.fields.len());
        while let Some((parent_node_id, space_parent_node_id, space_node_id)) = stack.pop() {
            debug_assert!(
                steiner_tree.nodes[steiner_input_map.space_node_id_to_node_id[space_node_id.index()].index()]
            );
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
                    let (
                        field_space_node_id,
                        &QueryFieldNode {
                            id: query_field_id,
                            flags,
                        },
                    ) = space
                        .graph
                        .edges_directed(space_node_id, Direction::Outgoing)
                        .find_map(|edge| {
                            if matches!(edge.weight(), SpaceEdge::Provides) {
                                if let SpaceNode::QueryField(field) = &space.graph[edge.target()] {
                                    Some((edge.target(), field))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .expect("Providable field should have at least one outgoing provides edge");

                    let field_node_id = graph.add_node(Node::Field {
                        id: query_field_id,
                        flags,
                    });
                    graph.add_edge(parent_node_id, field_node_id, Edge::Field);
                    query_field_to_node.push((query_field_id, field_node_id));

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
                                            type_conditions: space[query_field_id].type_conditions,
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
                                            location: space[query_field_id].location,
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
                                            location: space[query_field_id].location,
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

                    for space_edge in space.graph.edges(field_space_node_id) {
                        match space_edge.weight() {
                            SpaceEdge::Requires => required_edges.push(RequiredEdge {
                                source_node_id: field_node_id,
                                weight: Edge::RequiredBySupergraph,
                                target_space_node_id: space_edge.target(),
                            }),
                            // Assigning __typename fields to the first resolver that provides the
                            // parent field. There might be multiple with shared root fields.
                            SpaceEdge::TypenameField => {
                                space_edges_to_remove.push(space_edge.id());
                                if let SpaceNode::QueryField(QueryFieldNode {
                                    id: query_field_id,
                                    flags,
                                }) = space.graph[space_edge.target()]
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

            for space_edge in space.graph.edges(space_node_id) {
                if !matches!(space_edge.weight(), SpaceEdge::Requires) {
                    continue;
                }
                required_edges.push(RequiredEdge {
                    source_node_id: new_node_id,
                    weight: Edge::RequiredBySubgraph,
                    target_space_node_id: space_edge.target(),
                });
            }

            stack.extend(
                space
                    .graph
                    .edges(space_node_id)
                    .filter(|edge| {
                        (matches!(
                            edge.weight(),
                            SpaceEdge::CreateChildResolver | SpaceEdge::CanProvide | SpaceEdge::Provides
                        ) && included_space_edges[edge.id().index()])
                            || matches!(edge.weight(), SpaceEdge::TypenameField)
                    })
                    .map(|edge| (new_node_id, edge.source(), edge.target())),
            );
        }

        let field_to_node = IdToMany::from(query_field_to_node);

        for RequiredEdge {
            source_node_id,
            weight,
            target_space_node_id,
        } in required_edges
        {
            let SpaceNode::QueryField(field) = &space.graph[target_space_node_id] else {
                unreachable!()
            };
            for node_id in field_to_node.find_all(field.id).copied() {
                graph.add_edge(source_node_id, node_id, weight);
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
