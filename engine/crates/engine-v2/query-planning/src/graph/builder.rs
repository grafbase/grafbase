use schema::{Definition, RequiredFieldSet, Schema, SubgraphId, TypeSystemDirective};
use walker::Walk;

use super::*;
use petgraph::{graph::NodeIndex, visit::EdgeRef, Direction};

pub(super) struct OperationGraphBuilder<'ctx, Op: Operation> {
    inner: OperationGraph<'ctx, Op>,
    field_ingestion_stack: Vec<Field<Op::FieldId>>,
    requirement_ingestion_stack: Vec<Requirement<'ctx>>,
}

impl<'ctx, Op: Operation> std::ops::Deref for OperationGraphBuilder<'ctx, Op> {
    type Target = OperationGraph<'ctx, Op>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'ctx, Op: Operation> std::ops::DerefMut for OperationGraphBuilder<'ctx, Op> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

struct Requirement<'ctx> {
    parent_field_node: NodeIndex,
    petitioner_node: NodeIndex,
    required_field_set: RequiredFieldSet<'ctx>,
}

#[derive(Clone)]
struct ParentResolver {
    field_resolver_node: NodeIndex,
    subgraph_id: SubgraphId,
}

struct Field<Id> {
    parent_field_node: NodeIndex,
    parent_field_resolver: Option<ParentResolver>,
    field_id: Id,
}

impl<'ctx, Op: Operation> OperationGraph<'ctx, Op> {
    pub(super) fn builder(schema: &'ctx Schema, operation: &'ctx mut Op) -> OperationGraphBuilder<'ctx, Op> {
        let n = operation.field_ids().len();
        let mut graph = petgraph::stable_graph::StableGraph::with_capacity(n * 2, n * 2);
        let root = graph.add_node(Node::Root);

        OperationGraphBuilder {
            inner: OperationGraph {
                schema,
                operation,
                root,
                graph,
                leaf_nodes: Vec::new(),
                field_nodes: Vec::new(),
            },
            field_ingestion_stack: Vec::new(),
            requirement_ingestion_stack: Vec::new(),
        }
    }
}

impl<'ctx, Op: Operation> OperationGraphBuilder<'ctx, Op> {
    pub(super) fn build(mut self) -> OperationGraph<'ctx, Op> {
        self.field_nodes = self
            .inner
            .operation
            .field_ids()
            .map(|field_id| self.inner.graph.add_node(Node::Field(field_id)))
            .collect();

        self.leaf_nodes = self
            .operation
            .field_ids()
            .filter_map(|field_id| {
                if self
                    .operation
                    .field_defintion(field_id)
                    .walk(self.schema)
                    .is_some_and(|definition| matches!(definition.ty().definition(), Definition::Scalar(_)))
                {
                    Some(self[field_id])
                } else {
                    None
                }
            })
            .collect();

        self.field_ingestion_stack = self
            .operation
            .root_selection_set()
            .map(|field_id| Field {
                parent_field_node: self.root,
                parent_field_resolver: None,
                field_id,
            })
            .collect();

        // We first ingest all fields so that requirements can reference them. We use a double
        // stack as requirement may means adding new fields and adding new fields may add new
        // requirements.
        loop {
            if let Some(field) = self.field_ingestion_stack.pop() {
                self.handle_field_resolvers(field)
            } else if let Some(requirement) = self.requirement_ingestion_stack.pop() {
                self.handle_requirements(requirement)
            } else {
                break;
            }
        }

        self.inner.debug_assert_invariants();

        self.inner
    }

    fn handle_field_resolvers(
        &mut self,
        Field {
            parent_field_node,
            parent_field_resolver,
            field_id,
        }: Field<Op::FieldId>,
    ) {
        let field_node = self[field_id];
        let Some(definition_id) = self.operation.field_defintion(field_id) else {
            self.graph.add_edge(parent_field_node, field_node, Edge::TypenameField);
            return;
        };
        let field_definition = definition_id.walk(self.schema);

        // if it's the first time we see this field, there won't be any edges and we add any requirements from type system
        // directives. Otherwise it means we're not the first resolver path to this operation
        // field.
        if self.graph.edges(field_node).next().is_none() {
            for required_field_set in field_definition.directives().filter_map(|directive| match directive {
                TypeSystemDirective::Authenticated
                | TypeSystemDirective::Deprecated(_)
                | TypeSystemDirective::RequiresScopes(_) => None,
                TypeSystemDirective::Authorized(directive) => directive.fields(),
            }) {
                self.requirement_ingestion_stack.push(Requirement {
                    parent_field_node,
                    petitioner_node: field_node,
                    required_field_set,
                })
            }
        }
        self.graph.add_edge(parent_field_node, field_node, Edge::Field);

        // --
        // If resolvable withing the current subgraph. We skip requirements in this case.
        // --
        if let Some((parent_field_resolver_node, field_resolver, subgraph_id)) =
            parent_field_resolver.as_ref().and_then(
                |ParentResolver {
                     field_resolver_node,
                     subgraph_id,
                 }| {
                    self.graph[*field_resolver_node]
                        .as_field_resolver()
                        .unwrap()
                        .child(self.schema, field_definition.id())
                        .map(|child| (*field_resolver_node, child, *subgraph_id))
                },
            )
        {
            let field_resolver_node = self.graph.add_node(Node::FieldResolver(field_resolver));
            self.graph.add_edge(field_resolver_node, field_node, Edge::Resolves);
            self.graph.add_edge(
                parent_field_resolver_node,
                field_resolver_node,
                Edge::CanResolveField(0),
            );
            for nested_field_id in self.inner.operation.subselection(field_id) {
                self.field_ingestion_stack.push(Field {
                    parent_field_node: field_node,
                    parent_field_resolver: Some(ParentResolver {
                        field_resolver_node,
                        subgraph_id,
                    }),
                    field_id: nested_field_id,
                })
            }
        }

        // --
        // Try to plan this field with alternative resolvers if any exist.
        // --
        let parent_field_resolver_node = parent_field_resolver
            .as_ref()
            .map(
                |ParentResolver {
                     field_resolver_node, ..
                 }| *field_resolver_node,
            )
            .unwrap_or(self.root);
        let parent_subgraph_id = parent_field_resolver
            .as_ref()
            .map(|ParentResolver { subgraph_id, .. }| *subgraph_id);
        for resolver_definition in field_definition.resolvers() {
            // If within the same subgraph, we skip it. Resolvers are entrypoints.
            if Some(resolver_definition.subgraph_id()) == parent_subgraph_id {
                continue;
            };
            let resolver = FieldResolver::new(resolver_definition.id(), field_definition);
            let field_resolver_node = self.graph.add_node(Node::FieldResolver(resolver.clone()));

            // if the field has specific requirements for this subgraph we add it to the stack.
            if let Some(required_field_set) = field_definition.requires_for_subgraph(resolver_definition.subgraph_id())
            {
                self.requirement_ingestion_stack.push(Requirement {
                    parent_field_node,
                    petitioner_node: field_resolver_node,
                    required_field_set,
                })
            }

            // Try to find an existing resolver node if a sibling field already added it, otherwise
            // create one.
            let resolver_node = if let Some(edge) = self
                .graph
                .edges_directed(parent_field_resolver_node, Direction::Outgoing)
                .find(|edge| {
                    self.graph[edge.target()]
                        .as_resolver()
                        .is_some_and(|res| res.definition_id == resolver_definition.id())
                }) {
                edge.target()
            } else {
                let node = self.graph.add_node(Node::Resolver(Resolver {
                    definition_id: resolver_definition.id(),
                }));
                // Resolvers have an intrinsic initial cost of 1 as we'll need to make a request
                // for them.
                self.graph.add_edge(parent_field_resolver_node, node, Edge::Resolver(1));
                if let Some(required_field_set) = resolver_definition.required_field_set() {
                    self.requirement_ingestion_stack.push(Requirement {
                        parent_field_node,
                        petitioner_node: node,
                        required_field_set,
                    });
                };

                node
            };

            self.graph
                .add_edge(resolver_node, field_resolver_node, Edge::CanResolveField(0));
            self.graph.add_edge(field_resolver_node, field_node, Edge::Resolves);

            for nested_field_id in self.inner.operation.subselection(field_id) {
                self.field_ingestion_stack.push(Field {
                    parent_field_node: field_node,
                    parent_field_resolver: Some(ParentResolver {
                        field_resolver_node,
                        subgraph_id: resolver_definition.subgraph_id(),
                    }),
                    field_id: nested_field_id,
                })
            }
        }
    }

    fn handle_requirements(
        &mut self,
        Requirement {
            parent_field_node,
            petitioner_node,
            required_field_set,
        }: Requirement<'ctx>,
    ) {
        for item in required_field_set.items() {
            // Find an existing field that satisfies the requirement.
            let existing_field = self
                .graph
                .edges_directed(parent_field_node, Direction::Outgoing)
                .filter_map(|edge| {
                    if matches!(edge.weight(), Edge::Field) {
                        self.graph[edge.target()]
                            .as_field()
                            .map(|field_id| (edge.target(), field_id))
                    } else {
                        None
                    }
                })
                .filter(|(_, field_id)| self.operation.field_satisfies(*field_id, item.field()))
                // not sure if necessary but provides consistency
                .min_by_key(|(_, field_id)| *field_id);

            // Create the required field otherwise.
            let required_node = existing_field.map(|(node, _)| node).unwrap_or_else(|| {
                let field_id = self.operation.create_extra_field(item.field());
                let field_node = self.graph.add_node(Node::Field(field_id));
                self.field_nodes.push(field_node);
                if matches!(item.field().definition().ty().definition(), Definition::Scalar(_)) {
                    self.leaf_nodes.push(field_node);
                }
                self.field_ingestion_stack.extend(
                    self.inner
                        .graph
                        .edges_directed(parent_field_node, Direction::Incoming)
                        .filter_map(|edge| {
                            if matches!(edge.weight(), Edge::Resolves) {
                                let node = edge.source();
                                self.inner.graph[node].as_field_resolver().map(|r| (node, r))
                            } else {
                                None
                            }
                        })
                        .map(|(field_resolver_node, field_resolver)| Field {
                            parent_field_node,
                            parent_field_resolver: Some(ParentResolver {
                                field_resolver_node,
                                subgraph_id: field_resolver
                                    .resolver_definition_id
                                    .walk(self.inner.schema)
                                    .subgraph_id(),
                            }),
                            field_id,
                        }),
                );
                field_node
            });
            self.graph.add_edge(petitioner_node, required_node, Edge::Requires);

            if item.subselection().items().len() != 0 {
                self.requirement_ingestion_stack.push(Requirement {
                    parent_field_node: required_node,
                    petitioner_node,
                    required_field_set: item.subselection(),
                })
            }
        }
    }
}
