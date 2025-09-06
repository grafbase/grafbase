use fixedbitset::FixedBitSet;
use id_newtypes::{IdRange, IdToMany};
use itertools::Itertools;
use operation::{Operation, OperationContext};
use petgraph::{
    Direction, Graph,
    graph::NodeIndex,
    visit::{EdgeIndexable, EdgeRef},
};
use schema::{FieldDefinitionId, Schema};
use walker::Walk;

use crate::{
    Derive, DeriveId, FieldNode, ProvidableField, QueryField, QueryFieldId, QueryOrSchemaSortedFieldArgumentIds,
    QuerySolutionSpace, Resolver, SolutionGraph, SpaceNodeId, TypeConditionSharedVecId,
    query::{Edge, Node},
    solution_space::{SpaceEdge, SpaceNode},
    solve::{
        DeduplicationId, SerializedSolutionGraph,
        deduplication::DeduplicationMap,
        input::{SteinerInput, SteinerInputMap, SteinerNodeId},
        steiner_tree::SteinerTree,
    },
};

use super::QuerySteinerSolution;

pub(crate) struct Solution<'schema> {
    pub input: SteinerInput<'schema>,
    pub steiner_tree: SteinerTree,
}

impl<'schema> Solution<'schema> {
    pub fn into_query(self, schema: &'schema Schema, operation: &mut Operation) -> crate::Result<QuerySteinerSolution> {
        Builder::new(schema, operation, self).build()
    }
}

struct Builder<'schema, 'op> {
    schema: &'schema Schema,
    operation: &'op mut Operation,
    space: QuerySolutionSpace<'schema>,
    deduplication_map: DeduplicationMap,
    graph: SolutionGraph,
    root_node_id: NodeIndex,
    #[allow(unused)]
    field_to_dedup_id: Vec<DeduplicationId>,
    included_space_edges: FixedBitSet,
    steiner_tree: SteinerTree,
    steiner_input_map: SteinerInputMap,
    serialized_graph: SerializedSolutionGraph,
}

impl<'schema, 'op> Builder<'schema, 'op> {
    fn new(schema: &'schema Schema, operation: &'op mut Operation, solution: Solution<'schema>) -> Self {
        let Solution {
            input:
                SteinerInput {
                    space,
                    map: steiner_input_map,
                    space_node_is_terminal,
                    ..
                },
            steiner_tree,
        } = solution;

        let mut deduplication_map = DeduplicationMap::with_capacity(space.fields.len());
        let ctx = OperationContext { schema, operation };
        let field_to_dedup_id = (0..space.fields.len())
            .map(|ix| deduplication_map.get_or_insert_field(ctx, &space, QueryFieldId::from(ix)))
            .collect::<Vec<_>>();

        let mut n_edges = 0;
        let mut n_nodes = 0;
        let mut stack = space_node_is_terminal
            .into_ones()
            .map(SpaceNodeId::new)
            .collect::<Vec<_>>();
        let mut included_space_edges = FixedBitSet::with_capacity(space.graph.edge_bound());
        while let Some(space_node_id) = stack.pop() {
            n_nodes += 1;
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
                    n_edges += 1;
                    included_space_edges.insert(space_edge.id().index());
                    stack.push(space_edge.source());
                }
            }
        }

        let mut graph = Graph::with_capacity(n_nodes, n_edges);
        let root_node_id = graph.add_node(Node::Root);

        let serialized_graph = SerializedSolutionGraph::with_capacity(n_nodes * 2);

        Self {
            schema,
            operation,
            space,
            deduplication_map,
            graph,
            root_node_id,
            field_to_dedup_id,
            included_space_edges,
            steiner_tree,
            steiner_input_map,
            serialized_graph,
        }
    }

    fn build(mut self) -> crate::Result<QuerySteinerSolution> {
        self.ingest_nodes()?;

        let query = QuerySteinerSolution {
            step: crate::query::steps::SteinerSolution {},
            root_node_id: self.root_node_id,
            graph: self.graph,
            fields: self.space.fields,
            shared_type_conditions: self.space.shared_type_conditions,
            deduplicated_flat_sorted_executable_directives: self.space.deduplicated_flat_sorted_executable_directives,
        };

        tracing::debug!(
            "Partial solution:\n{}",
            query.to_pretty_dot_graph(OperationContext {
                schema: self.schema,
                operation: self.operation
            })
        );

        Ok(query)
    }
}

struct NodeToIngest {
    dedup_id: Option<DeduplicationId>,
    kind: NodeKind,
}

enum NodeKind {
    Resolver {
        parent_node_id: NodeIndex,
        space_node_id: SpaceNodeId,
        node: Resolver,
    },
    Field {
        parent_node_id: NodeIndex,
        parent_space_node_id: SpaceNodeId,
        space_node_id: SpaceNodeId,
        providable_field: ProvidableField,
        field_space_node_id: SpaceNodeId,
        node: FieldNode,
    },
    TypenameField {
        parent_node_id: NodeIndex,
        node: FieldNode,
    },
    LastChildMarker,
}

struct DerivedField {
    parent_space_node_id: SpaceNodeId,
    parent_node_id: NodeIndex,
    node_id: NodeIndex,
    derive_id: DeriveId,
}

impl<'schema, 'op> Builder<'schema, 'op> {
    fn ingest_nodes(&mut self) -> crate::Result<()> {
        let mut stack = Vec::new();
        self.insert_children(self.space.root_node_id, self.root_node_id, &mut stack);

        struct RequiredEdge {
            source_node_id: SteinerNodeId,
            weight: Edge,
            target_space_node_id: SpaceNodeId,
        }

        // Processing derived fields after de-duplication.
        let mut derived_fields = Vec::<DerivedField>::new();
        // Node must be created before we can crate required edges.
        let mut required_edges = Vec::<RequiredEdge>::new();
        let mut space_edges_to_remove = Vec::new();
        let mut query_field_to_node = Vec::with_capacity(self.space.fields.len());
        while let Some(NodeToIngest { dedup_id, kind }) = stack.pop() {
            self.serialized_graph.push(dedup_id);
            let (space_node_id, node_id) = match kind {
                NodeKind::LastChildMarker => {
                    continue;
                }
                NodeKind::TypenameField { parent_node_id, node } => {
                    let typename_field_id = self.graph.add_node(Node::Field(node));
                    self.graph.add_edge(parent_node_id, typename_field_id, Edge::Field);
                    continue;
                }
                NodeKind::Resolver {
                    parent_node_id,
                    space_node_id,
                    node,
                } => {
                    debug_assert!(
                        self.steiner_tree.nodes
                            [self.steiner_input_map.space_node_id_to_node_id[space_node_id.index()].index()]
                    );
                    let node_id = self.graph.add_node(Node::QueryPartition {
                        entity_definition_id: node.entity_definition_id,
                        resolver_definition_id: node.definition_id,
                    });
                    self.graph.add_edge(parent_node_id, node_id, Edge::QueryPartition);
                    (space_node_id, node_id)
                }
                NodeKind::Field {
                    parent_node_id,
                    parent_space_node_id,
                    space_node_id,
                    providable_field,
                    field_space_node_id,
                    node,
                } => {
                    debug_assert!(
                        self.steiner_tree.nodes
                            [self.steiner_input_map.space_node_id_to_node_id[space_node_id.index()].index()]
                    );
                    let node_id = self.graph.add_node(Node::Field(node));
                    self.graph.add_edge(parent_node_id, node_id, Edge::Field);
                    query_field_to_node.push(((node.id, node.split_id), node_id));

                    if let Some(derive_id) = providable_field.derive_id {
                        derived_fields.push(DerivedField {
                            parent_space_node_id,
                            parent_node_id,
                            node_id,
                            derive_id,
                        });
                    }

                    for space_edge in self.space.graph.edges(field_space_node_id) {
                        match space_edge.weight() {
                            SpaceEdge::Requires => required_edges.push(RequiredEdge {
                                source_node_id: node_id,
                                weight: Edge::RequiredBySupergraph,
                                target_space_node_id: space_edge.target(),
                            }),
                            // Assigning __typename fields to the first resolver that provides the
                            // parent field. There might be multiple with shared root fields.
                            SpaceEdge::TypenameField => {
                                space_edges_to_remove.push(space_edge.id());
                                if let SpaceNode::Field(node) = self.space.graph[space_edge.target()]
                                    && self.space[node.id].definition_id.is_none()
                                {
                                    let typename_field_ix = self.graph.add_node(Node::Field(node));
                                    self.graph.add_edge(node_id, typename_field_ix, Edge::Field);
                                }
                            }
                            _ => (),
                        }
                    }

                    for edge in space_edges_to_remove.drain(..) {
                        self.space.graph.remove_edge(edge);
                    }

                    (space_node_id, node_id)
                }
            };

            for space_edge in self.space.graph.edges(space_node_id) {
                if !matches!(space_edge.weight(), SpaceEdge::Requires) {
                    continue;
                }
                required_edges.push(RequiredEdge {
                    source_node_id: node_id,
                    weight: Edge::RequiredBySubgraph,
                    target_space_node_id: space_edge.target(),
                });
            }

            self.insert_children(space_node_id, node_id, &mut stack);
        }

        for field in derived_fields {
            self.insert_derived_field(field);
        }

        let field_to_node = IdToMany::from(query_field_to_node);

        for RequiredEdge {
            source_node_id,
            weight,
            target_space_node_id,
        } in required_edges
        {
            let SpaceNode::Field(field) = &self.space.graph[target_space_node_id] else {
                unreachable!()
            };
            for node_id in field_to_node.find_all((field.id, field.split_id)).copied() {
                self.graph.add_edge(source_node_id, node_id, weight);
            }
        }

        Ok(())
    }

    fn insert_children(
        &mut self,
        parent_space_node_id: SpaceNodeId,
        parent_node_id: NodeIndex,
        stack: &mut Vec<NodeToIngest>,
    ) {
        let n = stack.len();
        stack.extend(
            self.space
                .graph
                .edges(parent_space_node_id)
                .filter_map(|edge| match edge.weight() {
                    SpaceEdge::CreateChildResolver => {
                        if self.included_space_edges[edge.id().index()] {
                            let SpaceNode::Resolver(node) = self.space.graph[edge.target()] else {
                                unreachable!(
                                    "CreateChildResolver edges should only point to Resolver nodes {:?}",
                                    self.space.graph[edge.target()]
                                )
                            };
                            let dedup_id = self.deduplication_map.get_or_insert_resolver(node.definition_id);
                            Some(NodeToIngest {
                                dedup_id: Some(dedup_id),
                                kind: NodeKind::Resolver {
                                    parent_node_id,
                                    space_node_id: edge.target(),
                                    node,
                                },
                            })
                        } else {
                            None
                        }
                    }
                    SpaceEdge::TypenameField => {
                        let SpaceNode::Field(node) = self.space.graph[edge.target()] else {
                            unreachable!(
                                "TypenameField edges should only point to Field nodes {:?}",
                                self.space.graph[edge.target()]
                            )
                        };

                        let dedup_id = self.field_to_dedup_id[usize::from(node.id)];

                        Some(NodeToIngest {
                            dedup_id: Some(dedup_id),
                            kind: NodeKind::TypenameField { parent_node_id, node },
                        })
                    }
                    SpaceEdge::CanProvide => {
                        if self.included_space_edges[edge.id().index()] {
                            let SpaceNode::ProvidableField(providable_field) = self.space.graph[edge.target()] else {
                                unreachable!(
                                    "CanProvide and Provides edges should only point to ProvidableField nodes: {:?}",
                                    self.space.graph[edge.target()]
                                )
                            };
                            let (field_space_node_id, &node) = self
                                .space
                                .graph
                                .edges_directed(edge.target(), Direction::Outgoing)
                                .find_map(|edge| {
                                    if matches!(edge.weight(), SpaceEdge::Provides) {
                                        if let SpaceNode::Field(field) = &self.space.graph[edge.target()] {
                                            Some((edge.target(), field))
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                })
                                .expect("Providable field should have at least one outgoing provides edge");
                            let dedup_id = self.field_to_dedup_id[usize::from(node.id)];

                            Some(NodeToIngest {
                                dedup_id: Some(dedup_id),
                                kind: NodeKind::Field {
                                    parent_node_id,
                                    space_node_id: edge.target(),
                                    parent_space_node_id,
                                    providable_field,
                                    field_space_node_id,
                                    node,
                                },
                            })
                        } else {
                            None
                        }
                    }
                    _ => None,
                }),
        );

        // Ensures consistent ordering of the children across the whole graph.
        stack[n..].sort_by_key(|item| item.dedup_id);
        // We shouldn't have nay duplicates within a selection set.
        debug_assert_eq!(
            stack[n..].iter().map(|item| item.dedup_id).dedup().count(),
            stack[n..].len(),
            "Invalid deduplication"
        );
        stack.push(NodeToIngest {
            dedup_id: None,
            kind: NodeKind::LastChildMarker,
        });
    }

    // TODO: Maybe move this logic to the post processing?
    fn insert_derived_field(
        &mut self,
        DerivedField {
            parent_space_node_id,
            parent_node_id,
            node_id,
            derive_id,
        }: DerivedField,
    ) {
        let node = match &self.graph[node_id] {
            Node::Field(node) => *node,
            _ => unreachable!(),
        };
        let derive = self.space.step[derive_id];
        let derive_root = match derive {
            Derive::Root { id } => id,
            _ => self.space.graph[parent_space_node_id]
                .as_providable_field()
                .and_then(|field| field.derive_id)
                .and_then(|id| self.space.step[id].into_root())
                .unwrap(),
        }
        .walk(self.schema);

        let (type_conditions, source_node_id) = if let Some(batch_field_id) = derive_root.batch_field_id {
            let source_node_id = if matches!(derive, Derive::Root { .. }) {
                let source_node_id = self.find_or_create_field(
                    parent_node_id,
                    batch_field_id,
                    self.space[node.id].type_conditions,
                    node,
                );
                self.graph.add_edge(source_node_id, node_id, Edge::Derive);
                source_node_id
            } else {
                let grandparent_node_id = self
                    .graph
                    .edges_directed(parent_node_id, Direction::Incoming)
                    .find(|edge| matches!(edge.weight(), Edge::Field))
                    .map(|edge| edge.source())
                    .unwrap();

                self.find_field(grandparent_node_id, batch_field_id)
                    .expect("Batch field id should be present")
            };
            (Default::default(), source_node_id)
        } else {
            let parent_query_field_id = self.graph[parent_node_id]
                .as_query_field()
                .expect("Could not be derived otherwise.");
            let grandparent_node_id = self
                .graph
                .edges_directed(parent_node_id, Direction::Incoming)
                .find(|edge| matches!(edge.weight(), Edge::Field))
                .map(|edge| edge.source())
                .unwrap();
            (self.space[parent_query_field_id].type_conditions, grandparent_node_id)
        };
        match derive {
            Derive::Field { from_id } => {
                let derived_from_node_id = self.find_or_create_field(source_node_id, from_id, type_conditions, node);
                self.graph.add_edge(derived_from_node_id, node_id, Edge::Derive);
            }
            Derive::ScalarAsField => {
                self.graph.add_edge(source_node_id, node_id, Edge::Derive);
            }
            Derive::Root { .. } => {}
        }
    }

    fn find_or_create_field(
        &mut self,
        parent_node_id: NodeIndex,
        field_definition_id: FieldDefinitionId,
        type_conditions: IdRange<TypeConditionSharedVecId>,
        node: FieldNode,
    ) -> NodeIndex {
        self.find_field(parent_node_id, field_definition_id).unwrap_or_else(|| {
            let field = QueryField {
                type_conditions,
                query_position: None,
                response_key: Some(
                    self.operation
                        .response_keys
                        .get_or_intern(field_definition_id.walk(self.schema).name()),
                ),
                definition_id: Some(field_definition_id),
                matching_field_id: None,
                sorted_argument_ids: QueryOrSchemaSortedFieldArgumentIds::Query(IdRange::empty()),
                location: self.space[node.id].location,
                flat_directive_id: None,
            };
            self.space.fields.push(field);
            let id = QueryFieldId::from(self.space.fields.len() - 1);
            let field_node_id = self.graph.add_node(Node::Field(FieldNode { id, ..node }));
            self.graph.add_edge(parent_node_id, field_node_id, Edge::Field);
            field_node_id
        })
    }

    fn find_field(&self, parent_node_id: NodeIndex, field_definition_id: FieldDefinitionId) -> Option<NodeIndex> {
        self.graph
            .edges_directed(parent_node_id, Direction::Outgoing)
            .find_map(|edge| {
                if !matches!(edge.weight(), Edge::Field) {
                    return None;
                }
                let Node::Field(target) = self.graph[edge.target()] else {
                    return None;
                };
                let field = &self.space[target.id];
                if field.definition_id == Some(field_definition_id) {
                    return Some(edge.target());
                }
                None
            })
    }
}
