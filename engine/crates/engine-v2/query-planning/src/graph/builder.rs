use schema::{RequiredFieldSet, Schema, SubgraphId, TypeSystemDirective};
use walker::Walk;

use crate::EdgeCostId;

use super::*;
use petgraph::{graph::NodeIndex, visit::EdgeRef, Direction};

pub(super) struct OperationGraphBuilder<'ctx, Op: Operation> {
    pub(super) schema: &'ctx Schema,
    pub(super) operation: &'ctx mut Op,
    pub(super) graph: StableGraph<Node<Op::FieldId>, Edge>,
    pub(super) root: NodeIndex,
    pub(super) field_nodes: Vec<NodeIndex>,
    field_ingestion_stack: Vec<CreateProvidableFields<Op::FieldId>>,
    requirement_ingestion_stack: Vec<Requirement<'ctx>>,
    next_edge_cost_id: usize,
}

impl<'ctx, Op: Operation> std::ops::Index<Op::FieldId> for OperationGraphBuilder<'ctx, Op> {
    type Output = NodeIndex;
    fn index(&self, field_id: Op::FieldId) -> &Self::Output {
        let ix: usize = field_id.into();
        &self.field_nodes[ix]
    }
}

impl<'ctx, Op: Operation> OperationGraph<'ctx, Op> {
    pub(super) fn builder(schema: &'ctx Schema, operation: &'ctx mut Op) -> OperationGraphBuilder<'ctx, Op> {
        let n = operation.field_ids().len();
        let mut graph = petgraph::stable_graph::StableGraph::with_capacity(n * 2, n * 2);
        let root = graph.add_node(Node::Root);

        OperationGraphBuilder {
            schema,
            operation,
            root,
            graph,
            field_nodes: Vec::new(),
            field_ingestion_stack: Vec::new(),
            requirement_ingestion_stack: Vec::new(),
            next_edge_cost_id: 0,
        }
    }
}

impl<'ctx, Op: Operation> OperationGraphBuilder<'ctx, Op> {
    pub(super) fn build(mut self) -> crate::Result<OperationGraph<'ctx, Op>> {
        self.field_nodes = self
            .operation
            .field_ids()
            .map(|field_id| {
                self.graph.add_node(Node::QueryField(QueryField {
                    id: field_id,
                    // part of the query, so required
                    flags: FieldFlags::INDISPENSABLE,
                    // not known yet, added later
                    query_depth: u8::MAX,
                    // Added by the CostEstimator
                    requirement_id: None,
                }))
            })
            .collect();

        self.field_ingestion_stack = self
            .operation
            .root_selection_set()
            .map(|field_id| CreateProvidableFields {
                parent_query_depth: 0,
                parent_query_field_ix: self.root,
                parent_providable_field: None,
                field_id,
            })
            .collect();

        // We first ingest all fields so that requirements can reference them. We use a double
        // stack as requirement may means adding new fields and adding new fields may add new
        // requirements.
        loop {
            if let Some(create_providable_fields) = self.field_ingestion_stack.pop() {
                self.ingest_providable_fields(create_providable_fields)
            } else if let Some(requirement) = self.requirement_ingestion_stack.pop() {
                self.ingest_requirement(requirement)
            } else {
                break;
            }
        }

        self.prune_resolvers_not_leading_to_any_scalar_node();

        let cost_estimator = self.build_cost_estimator()?;

        Ok(OperationGraph {
            schema: self.schema,
            operation: self.operation,
            graph: self.graph,
            cost_estimator,
        })
    }

    pub(super) fn edge_cost_count(&self) -> usize {
        self.next_edge_cost_id
    }
}

struct Requirement<'ctx> {
    dependent_ix: NodeIndex,
    origin_query_field_ix: NodeIndex,
    parent_query_field_ix: NodeIndex,
    required_field_set: RequiredFieldSet<'ctx>,
}

#[derive(Clone)]
struct ParentProvidableField {
    ix: NodeIndex,
    subgraph_id: SubgraphId,
}

struct CreateProvidableFields<Id> {
    parent_query_depth: u8,
    parent_query_field_ix: NodeIndex,
    parent_providable_field: Option<ParentProvidableField>,
    field_id: Id,
}

impl<'ctx, Op: Operation> OperationGraphBuilder<'ctx, Op> {
    fn ingest_providable_fields(
        &mut self,
        CreateProvidableFields {
            parent_query_depth,
            parent_query_field_ix,
            parent_providable_field,
            field_id,
        }: CreateProvidableFields<Op::FieldId>,
    ) {
        let query_depth = parent_query_depth + 1;
        let query_field_ix = self[field_id];
        let Some(definition_id) = self.operation.field_defintion(field_id) else {
            if self
                .graph
                .edges_directed(query_field_ix, Direction::Incoming)
                .next()
                .is_none()
            {
                self.graph
                    .node_weight_mut(query_field_ix)
                    .unwrap()
                    .as_query_field_mut()
                    .unwrap()
                    .query_depth = query_depth;
                self.graph
                    .add_edge(parent_query_field_ix, query_field_ix, Edge::TypenameField);
            }
            return;
        };
        let field_definition = definition_id.walk(self.schema);

        // if it's the first time we see this field, there won't be any edges and we add any requirements from type system
        // directives. Otherwise it means we're not the first resolver path to this operation
        // field.
        if !self
            .graph
            .edges_directed(query_field_ix, Direction::Incoming)
            .any(|edge| matches!(edge.weight(), Edge::Field))
        {
            for required_field_set in field_definition.directives().filter_map(|directive| match directive {
                TypeSystemDirective::Authenticated
                | TypeSystemDirective::Deprecated(_)
                | TypeSystemDirective::RequiresScopes(_) => None,
                TypeSystemDirective::Authorized(directive) => directive.fields(),
            }) {
                self.requirement_ingestion_stack.push(Requirement {
                    dependent_ix: query_field_ix,
                    origin_query_field_ix: parent_query_field_ix,
                    parent_query_field_ix,
                    required_field_set,
                })
            }
            self.graph
                .node_weight_mut(query_field_ix)
                .unwrap()
                .as_query_field_mut()
                .unwrap()
                .query_depth = query_depth;
            self.graph.add_edge(parent_query_field_ix, query_field_ix, Edge::Field);
        }

        // --
        // If resolvable withing the current subgraph. We skip requirements in this case.
        // --
        if let Some((parent_providable_field_ix, providable_field, subgraph_id)) = parent_providable_field
            .as_ref()
            .and_then(|ParentProvidableField { ix, subgraph_id }| {
                self.graph[*ix]
                    .as_providable_field()
                    .unwrap()
                    .child(self.schema, field_definition.id())
                    .map(|providable_field| (*ix, providable_field, *subgraph_id))
            })
        {
            let providable_field_ix = self.graph.add_node(Node::ProvidableField(providable_field));
            let id = self.next_edge_cost_id();
            self.graph
                .add_edge(parent_providable_field_ix, providable_field_ix, Edge::CanProvide { id });
            self.graph.add_edge(providable_field_ix, query_field_ix, Edge::Provides);
            for nested_field_id in self.operation.subselection(field_id) {
                self.field_ingestion_stack.push(CreateProvidableFields {
                    parent_query_field_ix: query_field_ix,
                    parent_providable_field: Some(ParentProvidableField {
                        ix: providable_field_ix,
                        subgraph_id,
                    }),
                    field_id: nested_field_id,
                    parent_query_depth: query_depth,
                })
            }
        }

        // --
        // Try to plan this field with alternative resolvers if any exist.
        // --
        let parent_providable_field_ix = parent_providable_field
            .as_ref()
            .map(|ParentProvidableField { ix, .. }| *ix)
            .unwrap_or(self.root);
        let parent_subgraph_id = parent_providable_field
            .as_ref()
            .map(|ParentProvidableField { subgraph_id, .. }| *subgraph_id);

        for resolver_definition in field_definition.resolvers() {
            // If within the same subgraph, we skip it. Resolvers are entrypoints.
            if Some(resolver_definition.subgraph_id()) == parent_subgraph_id {
                continue;
            };

            // Try to find an existing resolver node if a sibling field already added it, otherwise
            // create one.
            let resolver_ix = if let Some(edge) = self
                .graph
                .edges_directed(parent_query_field_ix, Direction::Outgoing)
                .find(|edge| match edge.weight() {
                    Edge::HasChildResolver => self.graph[edge.target()]
                        .as_resolver()
                        .is_some_and(|res| res.definition_id == resolver_definition.id()),
                    _ => false,
                }) {
                let resolver_ix = edge.target();

                // A resolver node already exists within this selection set, so we don't need to
                // create one. The field itself might already have been processed through a
                // different path, so we check if there is any ProvidableField already providing the
                // current field.
                if self
                    .graph
                    .edges_directed(resolver_ix, Direction::Outgoing)
                    .any(|edge| match edge.weight() {
                        Edge::CanProvide { .. } => self
                            .graph
                            .edges_directed(edge.target(), Direction::Outgoing)
                            .any(|edge| matches!(edge.weight(), Edge::Provides) && edge.target() == query_field_ix),
                        _ => false,
                    })
                {
                    // If there is already a ProvidableField for the current query field, we don't need to process it any further,
                    // But we do need to add a CreateChildResolver edge to our parent ProvidableField as it may
                    // not exist yet.
                    if !self
                        .graph
                        .edges_directed(resolver_ix, Direction::Incoming)
                        .any(|edge| edge.source() == parent_providable_field_ix)
                    {
                        let id = self.next_edge_cost_id();
                        self.graph.add_edge(
                            parent_providable_field_ix,
                            resolver_ix,
                            Edge::CreateChildResolver { id },
                        );
                    }
                    continue;
                }
                resolver_ix
            } else {
                let resolver_ix = self.graph.add_node(Node::Resolver(Resolver {
                    definition_id: resolver_definition.id(),
                    query_depth,
                }));
                // Resolvers have an intrinsic initial cost of 1 as we'll need to make a request
                // for them.
                let id = self.next_edge_cost_id();
                self.graph.add_edge(
                    parent_providable_field_ix,
                    resolver_ix,
                    Edge::CreateChildResolver { id },
                );
                self.graph
                    .add_edge(parent_query_field_ix, resolver_ix, Edge::HasChildResolver);
                if let Some(required_field_set) = resolver_definition.required_field_set() {
                    self.requirement_ingestion_stack.push(Requirement {
                        dependent_ix: resolver_ix,
                        origin_query_field_ix: parent_query_field_ix,
                        parent_query_field_ix,
                        required_field_set,
                    });
                };

                resolver_ix
            };

            let providable_field = ProvidableField {
                query_depth,
                resolver_definition_id: resolver_definition.id(),
                field_definition_id: field_definition.id(),
            };
            let providable_field_ix = self.graph.add_node(Node::ProvidableField(providable_field));

            // if the field has specific requirements for this subgraph we add it to the stack.
            if let Some(required_field_set) = field_definition.requires_for_subgraph(resolver_definition.subgraph_id())
            {
                self.requirement_ingestion_stack.push(Requirement {
                    dependent_ix: providable_field_ix,
                    origin_query_field_ix: parent_query_field_ix,
                    parent_query_field_ix,
                    required_field_set,
                })
            }

            let id = self.next_edge_cost_id();
            self.graph
                .add_edge(resolver_ix, providable_field_ix, Edge::CanProvide { id });
            self.graph.add_edge(providable_field_ix, query_field_ix, Edge::Provides);

            for nested_field_id in self.operation.subselection(field_id) {
                self.field_ingestion_stack.push(CreateProvidableFields {
                    parent_query_field_ix: query_field_ix,
                    parent_providable_field: Some(ParentProvidableField {
                        ix: providable_field_ix,
                        subgraph_id: resolver_definition.subgraph_id(),
                    }),
                    field_id: nested_field_id,
                    parent_query_depth: query_depth,
                })
            }
        }
    }

    fn ingest_requirement(
        &mut self,
        Requirement {
            dependent_ix,
            origin_query_field_ix,
            parent_query_field_ix,
            required_field_set,
        }: Requirement<'ctx>,
    ) {
        for item in required_field_set.items() {
            // Find an existing field that satisfies the requirement.
            let existing_field = self
                .graph
                .edges_directed(parent_query_field_ix, Direction::Outgoing)
                .filter_map(|edge| {
                    if matches!(edge.weight(), Edge::Field) {
                        self.graph[edge.target()]
                            .as_query_field()
                            .map(|field| (edge.target(), field.id))
                    } else {
                        None
                    }
                })
                .filter(|(_, field_id)| self.operation.field_satisfies(*field_id, item.field()))
                // not sure if necessary but provides consistency
                .min_by_key(|(_, field_id)| *field_id);

            // Create the required field otherwise.
            let required_query_field_ix = if let Some((required_query_field_ix, _)) = existing_field {
                required_query_field_ix
            } else {
                // Create the QueryField Node
                let parent_query_depth = self.graph[parent_query_field_ix].query_depth();
                let field_id = self.operation.create_extra_field(item.field());
                let required_query_field = Node::QueryField(QueryField {
                    id: field_id,
                    query_depth: parent_query_depth + 1,
                    flags: FieldFlags::EXTRA,
                    requirement_id: None,
                });
                let required_query_field_ix = self.graph.add_node(required_query_field);
                self.field_nodes.push(required_query_field_ix);

                // For all the ProvidableField which may provide the parent QueryField, we have
                // to try whether they can provide this newly added nested QueryField
                self.field_ingestion_stack.extend(
                    self.graph
                        .edges_directed(parent_query_field_ix, Direction::Incoming)
                        .filter_map(|edge| {
                            if matches!(edge.weight(), Edge::Provides) {
                                let node = edge.source();
                                self.graph[node].as_providable_field().map(|r| (node, r))
                            } else {
                                None
                            }
                        })
                        .map(|(field_resolver_node, field_resolver)| CreateProvidableFields {
                            parent_query_field_ix,
                            parent_providable_field: Some(ParentProvidableField {
                                ix: field_resolver_node,
                                subgraph_id: field_resolver.resolver_definition_id.walk(self.schema).subgraph_id(),
                            }),
                            field_id,
                            parent_query_depth,
                        }),
                );

                required_query_field_ix
            };

            self.graph.add_edge(
                dependent_ix,
                required_query_field_ix,
                Edge::Requires { origin_query_field_ix },
            );

            if item.subselection().items().len() != 0 {
                self.requirement_ingestion_stack.push(Requirement {
                    dependent_ix,
                    origin_query_field_ix,
                    parent_query_field_ix: required_query_field_ix,
                    required_field_set: item.subselection(),
                })
            }
        }
    }

    fn next_edge_cost_id(&mut self) -> EdgeCostId {
        let id = EdgeCostId::from(self.next_edge_cost_id);
        self.next_edge_cost_id += 1;
        id
    }
}
