use fixedbitset::FixedBitSet;
use id_newtypes::{IdRange, IdToMany};
use operation::{Operation, OperationContext};
use petgraph::{
    Direction, Graph,
    graph::NodeIndex,
    visit::{EdgeIndexable, EdgeRef},
};
use schema::{FieldDefinitionId, Schema};
use walker::Walk;

use crate::{
    Derive, FieldNode, QueryField, QueryFieldId, QueryOrSchemaSortedFieldArgumentIds, QuerySolutionSpace,
    SolutionGraph, SpaceNodeId, TypeConditionSharedVecId,
    query::{Edge, Node},
    solution_space::{SpaceEdge, SpaceNode},
    solve::{
        DeduplicationId,
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
            .map(|ix| deduplication_map.get_or_insert_field(ctx, &space.fields, QueryFieldId::from(ix)))
            .collect::<Vec<_>>();

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

        let n = operation.data_fields.len() + operation.typename_fields.len();
        let mut graph = Graph::with_capacity(n, n);
        let root_node_id = graph.add_node(Node::Root);

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

    fn ingest_nodes(&mut self) -> crate::Result<()> {
        let mut stack = Vec::new();
        for edge in self.space.graph.edges(self.space.root_node_id) {
            match edge.weight() {
                SpaceEdge::CreateChildResolver if self.included_space_edges[edge.id().index()] => {
                    stack.push((self.root_node_id, edge.source(), edge.target()));
                }
                // For now assign __typename fields to the root node, they will be later be added
                // to an appropriate query partition.
                SpaceEdge::TypenameField => {
                    if let SpaceNode::Field(node) = self.space.graph[edge.target()]
                        && self.space[node.id].definition_id.is_none()
                    {
                        let typename_field_id = self.graph.add_node(Node::Field(node));
                        self.graph.add_edge(self.root_node_id, typename_field_id, Edge::Field);
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
        let mut query_field_to_node = Vec::with_capacity(self.space.fields.len());
        while let Some((parent_node_id, space_parent_node_id, space_node_id)) = stack.pop() {
            debug_assert!(
                self.steiner_tree.nodes[self.steiner_input_map.space_node_id_to_node_id[space_node_id.index()].index()]
            );
            let new_node_id = match &self.space.graph[space_node_id] {
                SpaceNode::Resolver(resolver) => {
                    let _dedup_id = self.deduplication_map.get_or_insert_resolver(resolver.definition_id);
                    let id = self.graph.add_node(Node::QueryPartition {
                        entity_definition_id: resolver.entity_definition_id,
                        resolver_definition_id: resolver.definition_id,
                    });
                    self.graph.add_edge(parent_node_id, id, Edge::QueryPartition);
                    id
                }
                SpaceNode::ProvidableField(providable_field) => {
                    let (field_space_node_id, &node) = self
                        .space
                        .graph
                        .edges_directed(space_node_id, Direction::Outgoing)
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

                    let field_node_id = self.graph.add_node(Node::Field(node));
                    self.graph.add_edge(parent_node_id, field_node_id, Edge::Field);
                    query_field_to_node.push(((node.id, node.split_id), field_node_id));

                    if let Some(derive) = providable_field.derive {
                        self.insert_derived_field(space_parent_node_id, parent_node_id, field_node_id, node, derive);
                    }

                    for space_edge in self.space.graph.edges(field_space_node_id) {
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
                                if let SpaceNode::Field(node) = self.space.graph[space_edge.target()]
                                    && self.space[node.id].definition_id.is_none()
                                {
                                    let typename_field_ix = self.graph.add_node(Node::Field(node));
                                    self.graph.add_edge(field_node_id, typename_field_ix, Edge::Field);
                                }
                            }
                            _ => (),
                        }
                    }

                    for edge in space_edges_to_remove.drain(..) {
                        self.space.graph.remove_edge(edge);
                    }

                    field_node_id
                }
                SpaceNode::Field(node) if self.space[node.id].definition_id.is_none() => {
                    let typename_field_ix = self.graph.add_node(Node::Field(*node));
                    self.graph.add_edge(parent_node_id, typename_field_ix, Edge::Field);
                    typename_field_ix
                }
                _ => continue,
            };

            for space_edge in self.space.graph.edges(space_node_id) {
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
                self.space
                    .graph
                    .edges(space_node_id)
                    .filter(|edge| {
                        (matches!(
                            edge.weight(),
                            SpaceEdge::CreateChildResolver | SpaceEdge::CanProvide | SpaceEdge::Provides
                        ) && self.included_space_edges[edge.id().index()])
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
            let SpaceNode::Field(field) = &self.space.graph[target_space_node_id] else {
                unreachable!()
            };
            for node_id in field_to_node.find_all((field.id, field.split_id)).copied() {
                self.graph.add_edge(source_node_id, node_id, weight);
            }
        }

        Ok(())
    }

    // TODO: Maybe move this logic to the post processing?
    fn insert_derived_field(
        &mut self,
        space_parent_node_id: SpaceNodeId,
        parent_node_id: NodeIndex,
        field_node_id: NodeIndex,
        node: FieldNode,
        derive: Derive,
    ) {
        let derive_root = match derive {
            Derive::Root { id } => id,
            _ => self.space.graph[space_parent_node_id]
                .as_providable_field()
                .and_then(|field| field.derive)
                .and_then(Derive::into_root)
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
                self.graph.add_edge(source_node_id, field_node_id, Edge::Derive);
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
                self.graph.add_edge(derived_from_node_id, field_node_id, Edge::Derive);
            }
            Derive::ScalarAsField => {
                self.graph.add_edge(source_node_id, field_node_id, Edge::Derive);
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
            let _dedup_id = self.deduplication_map.get_or_insert_field(
                OperationContext {
                    schema: self.schema,
                    operation: self.operation,
                },
                &self.space.fields,
                id,
            );
            let field_node_id = self.graph.add_node(Node::Field(FieldNode {
                id: (self.space.fields.len() - 1).into(),
                ..node
            }));
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
