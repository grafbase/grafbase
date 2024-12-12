mod alternative;
mod prune;

use std::borrow::Cow;

use fixedbitset::FixedBitSet;
use schema::{
    CompositeType, CompositeTypeId, DefinitionId, EntityDefinition, FieldDefinition, FieldSet, FieldSetRecord, Schema,
    SubgraphId, TypeSystemDirective,
};
use walker::Walk;

use crate::FieldFlags;

use super::*;
use petgraph::{stable_graph::NodeIndex, visit::EdgeRef, Direction};

pub(super) struct OperationGraphBuilder<'ctx, Op: Operation> {
    schema: &'ctx Schema,
    operation: Op,
    graph: StableGraph<Node<'ctx, Op::FieldId>, Edge>,
    root_ix: NodeIndex,
    field_nodes: Vec<NodeIndex>,
    providable_fields_bitset: FixedBitSet,
    deleted_fields_bitset: FixedBitSet,
    field_ingestion_stack: Vec<CreateProvidableFields<Op::FieldId>>,
    requirement_ingestion_stack: Vec<Requirement<'ctx, Op::FieldId>>,
}

impl<Op: Operation> std::ops::Index<Op::FieldId> for OperationGraphBuilder<'_, Op> {
    type Output = NodeIndex;
    fn index(&self, field_id: Op::FieldId) -> &Self::Output {
        let ix: usize = field_id.into();
        &self.field_nodes[ix]
    }
}

impl<'ctx, Op: Operation> OperationGraph<'ctx, Op> {
    pub(super) fn builder(schema: &'ctx Schema, operation: Op) -> OperationGraphBuilder<'ctx, Op> {
        let n = operation.field_ids().len();
        let mut graph = petgraph::stable_graph::StableGraph::with_capacity(n * 2, n * 2);
        let root_ix = graph.add_node(Node::Root);

        OperationGraphBuilder {
            schema,
            operation,
            root_ix,
            graph,
            field_nodes: Vec::with_capacity(n),
            providable_fields_bitset: FixedBitSet::with_capacity(n),
            deleted_fields_bitset: FixedBitSet::with_capacity(n),
            field_ingestion_stack: Vec::new(),
            requirement_ingestion_stack: Vec::new(),
        }
    }
}

impl<'ctx, Op: Operation> OperationGraphBuilder<'ctx, Op> {
    pub(super) fn build(mut self) -> crate::Result<OperationGraph<'ctx, Op>> {
        self.field_nodes = self
            .operation
            .field_ids()
            .map(|field_id| self.push_query_field(field_id, FieldFlags::INDISPENSABLE))
            .collect();

        self.field_ingestion_stack = self
            .operation
            .root_selection_set()
            .map(|field_id| CreateProvidableFields {
                parent_query_field_ix: self.root_ix,
                parent_providable_field_or_root_ix: self.root_ix,
                parent_output_type: CompositeTypeId::Object(self.operation.root_object_id()),
                field_id,
            })
            .collect();

        self.loop_over_ingestion_stacks();
        self.try_providing_missing_fields_through_alternatives();

        // FIXME: Should only check for indispensable fields. And then if we add new indispensable
        // field ensure we can provide them instead of everything at once...
        self.providable_fields_bitset |= &self.deleted_fields_bitset;
        for field_id in self.providable_fields_bitset.zeroes().map(Op::FieldId::from) {
            let node_ix = self[field_id];
            let Some(Node::QueryField(field)) = &mut self.graph.node_weight(node_ix) else {
                continue;
            };

            // If not reachable, we can delete the node and its sub-selection.
            if field.flags.contains(FieldFlags::UNREACHABLE) {
                if let Some(parent_query_field_ix) = self
                    .graph
                    .edges_directed(node_ix, Direction::Incoming)
                    .find(|edge| matches!(edge.weight(), Edge::Field))
                    .map(|edge| edge.source())
                {
                    if !self
                        .graph
                        .edges_directed(parent_query_field_ix, Direction::Outgoing)
                        .any(|edge| {
                            matches!(edge.weight(), Edge::Field | Edge::TypenameField) && edge.target() != node_ix
                        })
                    {
                        if let Node::QueryField(field) = &mut self.graph[parent_query_field_ix] {
                            field.flags |= FieldFlags::LEAF_NODE;
                        }
                    }
                }
                let mut stack = vec![field_id];
                while let Some(field_id) = stack.pop() {
                    stack.extend(self.operation.subselection(field_id));
                    self.graph.remove_node(self[field_id]);
                    self.deleted_fields_bitset.put(field_id.into());
                }

                continue;
            }

            // If the field is not associated with any providable field node and isn't a typename we can't plan it.
            if !self
                .graph
                .edges_directed(node_ix, Direction::Incoming)
                .any(|edge| matches!(edge.weight(), Edge::Provides | Edge::TypenameField))
            {
                let definition = self.operation.field_definition(field_id).walk(self.schema);

                tracing::debug!(
                    "Unplannable OperationGraph:\n{}",
                    OperationGraph {
                        schema: self.schema,
                        operation: self.operation,
                        root_ix: self.root_ix,
                        graph: self.graph,
                    }
                    .to_pretty_dot_graph()
                );

                return Err(crate::Error::CouldNotPlanField {
                    name: definition
                        .map(|def| {
                            tracing::debug!("Could not plan field:\n{def:#?}");
                            format!("{}.{}", def.parent_entity().name(), def.name())
                        })
                        .unwrap_or("__typename".into()),
                });
            }
        }

        self.prune_resolvers_not_leading_any_leafs();

        Ok(OperationGraph {
            schema: self.schema,
            operation: self.operation,
            root_ix: self.root_ix,
            graph: self.graph,
        })
    }

    fn loop_over_ingestion_stacks(&mut self) {
        // We first ingest all fields so that requirements can reference them. We use a double
        // stack as requirement may means adding new fields and adding new fields may add new
        // requirements.
        loop {
            if let Some(create_providable_fields) = self.field_ingestion_stack.pop() {
                self.ingest_providable_fields(create_providable_fields);
            } else if let Some(requirement) = self.requirement_ingestion_stack.pop() {
                self.ingest_requirement(requirement)
            } else {
                break;
            }
        }
    }
}

struct Requirement<'ctx, FieldId> {
    petitioner_field_id: FieldId,
    dependent_ix: NodeIndex,
    indispensable: bool,
    parent_query_field_ix: NodeIndex,
    required_field_set: FieldSet<'ctx>,
}

struct CreateProvidableFields<Id> {
    parent_query_field_ix: NodeIndex,
    parent_providable_field_or_root_ix: NodeIndex,
    #[allow(unused)]
    parent_output_type: CompositeTypeId,
    field_id: Id,
}

impl<'ctx, Op: Operation> OperationGraphBuilder<'ctx, Op> {
    fn ingest_providable_fields(
        &mut self,
        CreateProvidableFields {
            parent_query_field_ix,
            parent_providable_field_or_root_ix,
            parent_output_type,
            field_id,
        }: CreateProvidableFields<Op::FieldId>,
    ) {
        let query_field_ix = self[field_id];
        let Some(definition_id) = self.operation.field_definition(field_id) else {
            self.providable_fields_bitset.put(field_id.into());
            if self
                .graph
                .edges_directed(query_field_ix, Direction::Incoming)
                .next()
                .is_none()
            {
                self.graph
                    .add_edge(parent_query_field_ix, query_field_ix, Edge::TypenameField);
            }
            return;
        };
        let mut first_time_seeing_this_field = false;
        if !self
            .graph
            .edges_directed(query_field_ix, Direction::Incoming)
            .any(|edge| matches!(edge.weight(), Edge::Field))
        {
            first_time_seeing_this_field = true;
            self.graph.add_edge(parent_query_field_ix, query_field_ix, Edge::Field);
        }

        let field_definition = definition_id.walk(self.schema);

        // --
        // If providable by parent, we don't need to find for a resolver.
        // --
        let provide_result = self.graph[parent_providable_field_or_root_ix]
            .as_providable_field()
            .map(|parent| self.provide_field_from_parent(parent, parent_output_type, field_id, field_definition))
            .unwrap_or_default();
        let could_be_provided_from_parent = match provide_result {
            ParentProvideResult::Providable(child) => {
                let providable_field_ix = self.graph.add_node(Node::ProvidableField(child));
                self.graph.add_edge(
                    parent_providable_field_or_root_ix,
                    providable_field_ix,
                    Edge::CanProvide,
                );
                self.graph.add_edge(providable_field_ix, query_field_ix, Edge::Provides);
                self.providable_fields_bitset.put(field_id.into());
                if let Some(output_type) = field_definition.ty().definition_id.as_composite_type() {
                    for nested_field_id in self.operation.subselection(field_id) {
                        self.field_ingestion_stack.push(CreateProvidableFields {
                            parent_query_field_ix: query_field_ix,
                            parent_providable_field_or_root_ix: providable_field_ix,
                            parent_output_type: output_type,
                            field_id: nested_field_id,
                        })
                    }
                }
                true
            }
            ParentProvideResult::NotProvidable => false,
            ParentProvideResult::UnreachableObject => {
                // If can never be present giving the parent type, we simply discard the field and its
                // sub-selection.
                let mut stack = vec![field_id];
                while let Some(field_id) = stack.pop() {
                    stack.extend(self.operation.subselection(field_id));
                    let node_ix = self[field_id];
                    let Node::QueryField(field) = &mut self.graph[node_ix] else {
                        unreachable!()
                    };
                    field.flags |= FieldFlags::UNREACHABLE;
                }
                return;
            }
        };

        // if it's the first time we see this field, there won't be any edges and we add any requirements from type system
        // directives. Otherwise it means we're not the first resolver path to this operation
        // field.
        if first_time_seeing_this_field {
            for directive in field_definition.directives() {
                let TypeSystemDirective::Authorized(auth) = directive else {
                    continue;
                };
                if let Some(fields) = auth.fields() {
                    self.requirement_ingestion_stack.push(Requirement {
                        petitioner_field_id: field_id,
                        dependent_ix: query_field_ix,
                        indispensable: self.graph[query_field_ix].as_query_field().unwrap().is_indispensable(),
                        parent_query_field_ix,
                        required_field_set: fields,
                    })
                }
                if let Some(node) = auth.node() {
                    self.requirement_ingestion_stack.push(Requirement {
                        petitioner_field_id: field_id,
                        dependent_ix: query_field_ix,
                        indispensable: self.graph[query_field_ix].as_query_field().unwrap().is_indispensable(),
                        parent_query_field_ix: query_field_ix,
                        required_field_set: node,
                    })
                }
            }
        }

        let parent_subgraph_id = self.graph[parent_providable_field_or_root_ix]
            .as_providable_field()
            .map(|field| field.subgraph_id());

        // --
        // Try to plan this field with alternative resolvers if any exist.
        // --
        for resolver_definition in field_definition.resolvers() {
            // If within the same subgraph, we skip it. Resolvers are entrypoints.
            if could_be_provided_from_parent && Some(resolver_definition.subgraph_id()) == parent_subgraph_id {
                continue;
            };

            // Try to find an existing resolver node if a sibling field already added it, otherwise
            // create one.
            let resolver_ix = if let Some(edge) = self
                .graph
                .edges_directed(parent_query_field_ix, Direction::Outgoing)
                .find(|edge| match edge.weight() {
                    Edge::HasChildResolver { .. } => self.graph[edge.target()]
                        .as_resolver()
                        .is_some_and(|res| res.definition_id == resolver_definition.id),
                    _ => false,
                }) {
                let resolver_ix = edge.target();

                // If it does not exist already we a relation between the parent providable field
                // and the existing resolver. It may exist already as we needed this resolver for
                // another field.
                if !self
                    .graph
                    .edges_directed(resolver_ix, Direction::Incoming)
                    .any(|edge| edge.source() == parent_providable_field_or_root_ix)
                {
                    self.graph.add_edge(
                        parent_providable_field_or_root_ix,
                        resolver_ix,
                        Edge::CreateChildResolver,
                    );
                }

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
                    continue;
                }

                resolver_ix
            } else {
                let resolver_ix = self.graph.add_node(Node::Resolver(Resolver {
                    entity_definition_id: field_definition.parent_entity_id,
                    definition_id: resolver_definition.id,
                }));
                self.graph.add_edge(
                    parent_providable_field_or_root_ix,
                    resolver_ix,
                    Edge::CreateChildResolver,
                );
                self.graph
                    .add_edge(parent_query_field_ix, resolver_ix, Edge::HasChildResolver);
                if let Some(required_field_set) = resolver_definition.required_field_set() {
                    self.requirement_ingestion_stack.push(Requirement {
                        petitioner_field_id: field_id,
                        dependent_ix: resolver_ix,
                        indispensable: false,
                        parent_query_field_ix,
                        required_field_set,
                    });
                };

                resolver_ix
            };

            let providable_field = ProvidableField::InSubgraph {
                subgraph_id: resolver_definition.subgraph_id(),
                id: field_id,
                provides: field_definition
                    .provides_for_subgraph(resolver_definition.subgraph_id())
                    .map(|field_set| Cow::Borrowed(field_set.as_ref()))
                    .unwrap_or(Cow::Borrowed(FieldSetRecord::empty())),
            };
            let providable_field_ix = self.graph.add_node(Node::ProvidableField(providable_field));

            // if the field has specific requirements for this subgraph we add it to the stack.
            if let Some(required_field_set) = field_definition.requires_for_subgraph(resolver_definition.subgraph_id())
            {
                self.requirement_ingestion_stack.push(Requirement {
                    petitioner_field_id: field_id,
                    dependent_ix: providable_field_ix,
                    indispensable: false,
                    parent_query_field_ix,
                    required_field_set,
                })
            }

            self.graph.add_edge(resolver_ix, providable_field_ix, Edge::CanProvide);
            self.graph.add_edge(providable_field_ix, query_field_ix, Edge::Provides);
            self.providable_fields_bitset.put(field_id.into());

            if let Some(output_type) = field_definition.ty().definition_id.as_composite_type() {
                for nested_field_id in self.operation.subselection(field_id) {
                    self.field_ingestion_stack.push(CreateProvidableFields {
                        parent_query_field_ix: query_field_ix,
                        parent_providable_field_or_root_ix: providable_field_ix,
                        parent_output_type: output_type,
                        field_id: nested_field_id,
                    })
                }
            }
        }
    }

    fn provide_field_from_parent(
        &self,
        parent: &ProvidableField<'ctx, Op::FieldId>,
        parent_output: CompositeTypeId,
        id: Op::FieldId,
        field_definition: FieldDefinition<'ctx>,
    ) -> ParentProvideResult<'ctx, Op::FieldId> {
        match parent {
            ProvidableField::InSubgraph {
                subgraph_id, provides, ..
            } => {
                let subgraph_id = *subgraph_id;
                let is_reachable = self.is_field_parent_object_reachable_in_subgraph_from_parent_output(
                    subgraph_id,
                    parent_output,
                    field_definition,
                );
                if is_reachable
                    && self.is_field_providable_in_subgraph(subgraph_id, field_definition)
                    && field_definition.requires_for_subgraph(subgraph_id).is_none()
                {
                    ParentProvideResult::Providable(ProvidableField::InSubgraph {
                        subgraph_id,
                        id,
                        provides: self
                            .find_in_provides(subgraph_id, provides, id, field_definition)
                            .unwrap_or_else(|| {
                                field_definition
                                    .provides_for_subgraph(subgraph_id)
                                    .map(|field_set| Cow::Borrowed(field_set.as_ref()))
                                    .unwrap_or(Cow::Borrowed(FieldSetRecord::empty()))
                            }),
                    })
                } else {
                    self.find_in_provides(subgraph_id, provides, id, field_definition)
                        .map(|provides| {
                            ParentProvideResult::Providable(ProvidableField::OnlyProvidable {
                                subgraph_id,
                                id,
                                provides,
                            })
                        })
                        .unwrap_or_else(|| {
                            if is_reachable {
                                ParentProvideResult::NotProvidable
                            } else {
                                ParentProvideResult::UnreachableObject
                            }
                        })
                }
            }
            ProvidableField::OnlyProvidable {
                subgraph_id, provides, ..
            } => self
                .find_in_provides(*subgraph_id, provides, id, field_definition)
                .map(|provides| {
                    ParentProvideResult::Providable(ProvidableField::OnlyProvidable {
                        subgraph_id: *subgraph_id,
                        id,
                        provides,
                    })
                })
                .unwrap_or_default(),
        }
    }

    fn is_field_providable_in_subgraph(&self, subgraph_id: SubgraphId, field_definition: FieldDefinition<'_>) -> bool {
        match field_definition.parent_entity() {
            EntityDefinition::Interface(_) => field_definition.exists_in_subgraph_ids.contains(&subgraph_id),
            EntityDefinition::Object(obj) => {
                obj.exists_in_subgraph_ids.contains(&subgraph_id)
                    && (field_definition.exists_in_subgraph_ids.contains(&subgraph_id))
            }
        }
    }

    fn is_field_parent_object_reachable_in_subgraph_from_parent_output(
        &self,
        subgraph_id: SubgraphId,
        parent_output_type: CompositeTypeId,
        field_definition: FieldDefinition<'_>,
    ) -> bool {
        match parent_output_type.walk(self.schema) {
            // If the parent output_type is an interface, we can't say what the actual object type
            // will be underneath. So we can't know whether an object is really unreachable or not.
            CompositeType::Interface(_) => true,
            // If the field is not part of any member of this union, we assume it's unreachable.
            CompositeType::Union(union) => {
                if union.is_fully_implemented_in(subgraph_id) {
                    true
                } else {
                    // Not super efficient...
                    for object in field_definition.parent_entity().possible_type_ids().walk(self.schema) {
                        if union.has_member_in_subgraph(subgraph_id, object.id) {
                            return true;
                        }
                    }
                    false
                }
            }
            // Whatever the field, we know the object type and it is providable by the parent.
            CompositeType::Object(_) => true,
        }
    }

    fn find_in_provides(
        &self,
        subgraph_id: SubgraphId,
        provides: &Cow<'ctx, FieldSetRecord>,
        id: Op::FieldId,
        definition: FieldDefinition<'ctx>,
    ) -> Option<Cow<'ctx, FieldSetRecord>> {
        match provides {
            Cow::Borrowed(provides) => provides
                .iter()
                .find(|item| {
                    self.operation
                        .field_is_equivalent_to(id, item.field_id.walk(self.schema))
                })
                .map(|item| match definition.provides_for_subgraph(subgraph_id) {
                    Some(field_provides) => Cow::Owned(FieldSetRecord::union(
                        field_provides.as_ref(),
                        &item.subselection_record,
                    )),
                    None => Cow::Borrowed(&item.subselection_record),
                }),
            Cow::Owned(provides) => provides
                .iter()
                .find(|item| {
                    self.operation
                        .field_is_equivalent_to(id, item.field_id.walk(self.schema))
                })
                .map(|item| match definition.provides_for_subgraph(subgraph_id) {
                    Some(field_provides) => Cow::Owned(FieldSetRecord::union(
                        field_provides.as_ref(),
                        &item.subselection_record,
                    )),
                    None => Cow::Owned(item.subselection_record.clone()),
                }),
        }
    }

    fn ingest_requirement(
        &mut self,
        Requirement {
            petitioner_field_id,
            dependent_ix,
            indispensable,
            parent_query_field_ix,
            required_field_set,
        }: Requirement<'ctx, Op::FieldId>,
    ) {
        for required_item in required_field_set.items() {
            // Find an existing field that satisfies the requirement.
            let existing_field = self
                .graph
                .edges_directed(parent_query_field_ix, Direction::Outgoing)
                .filter_map(|edge| {
                    if matches!(edge.weight(), Edge::Field) {
                        self.graph[edge.target()]
                            .as_query_field()
                            .map(|field| (edge.target(), field))
                    } else {
                        None
                    }
                })
                .filter(|(_, field)| self.operation.field_is_equivalent_to(field.id, required_item.field()))
                // not sure if necessary but provides consistency
                .min_by_key(|(_, field)| field.id);

            // Create the required field otherwise.
            let required_query_field_ix = if let Some((required_query_field_ix, _)) = existing_field {
                required_query_field_ix
            } else {
                // Create the QueryField Node
                let field_id = self
                    .operation
                    .create_potential_extra_field_from_requirement(petitioner_field_id, required_item.field());
                let required_query_field_ix = self.push_query_field(
                    field_id,
                    if indispensable {
                        FieldFlags::EXTRA | FieldFlags::INDISPENSABLE
                    } else {
                        FieldFlags::EXTRA
                    },
                );
                self.push_field_to_provide(parent_query_field_ix, field_id);

                required_query_field_ix
            };

            self.graph
                .add_edge(dependent_ix, required_query_field_ix, Edge::Requires);

            if required_item.subselection().items().len() != 0 {
                self.requirement_ingestion_stack.push(Requirement {
                    petitioner_field_id,
                    dependent_ix,
                    indispensable,
                    parent_query_field_ix: required_query_field_ix,
                    required_field_set: required_item.subselection(),
                })
            }
        }
    }

    fn push_field_to_provide(&mut self, parent_query_field_ix: NodeIndex, field_id: Op::FieldId) {
        if parent_query_field_ix == self.root_ix {
            self.field_ingestion_stack.push(CreateProvidableFields {
                parent_query_field_ix,
                parent_providable_field_or_root_ix: self.root_ix,
                parent_output_type: CompositeTypeId::Object(self.operation.root_object_id()),
                field_id,
            });
        } else {
            let parent_output_type = self
                .operation
                .field_definition(self.graph[parent_query_field_ix].as_query_field().unwrap().id)
                .unwrap()
                .walk(self.schema)
                .ty()
                .definition_id
                .as_composite_type()
                .unwrap();
            // For all the ProvidableField which may provide the parent QueryField, we have
            // to try whether they can provide this newly added nested QueryField
            self.field_ingestion_stack.extend(
                self.graph
                    .edges_directed(parent_query_field_ix, Direction::Incoming)
                    .filter(|edge| {
                        matches!(edge.weight(), Edge::Provides) && self.graph[edge.source()].is_providable_field()
                    })
                    .map(|edge| CreateProvidableFields {
                        parent_query_field_ix,
                        parent_providable_field_or_root_ix: edge.source(),
                        parent_output_type,
                        field_id,
                    }),
            );
        }
    }

    fn push_query_field(&mut self, id: Op::FieldId, mut flags: FieldFlags) -> NodeIndex {
        if let Some(field_definition) = self.operation.field_definition(id) {
            match field_definition.walk(self.schema).ty().definition_id {
                DefinitionId::Scalar(_) | DefinitionId::Enum(_) => {
                    flags |= FieldFlags::LEAF_NODE;
                }
                DefinitionId::Union(_) | DefinitionId::Interface(_) | DefinitionId::Object(_) => {
                    flags |= FieldFlags::IS_COMPOSITE_TYPE;
                }
                _ => (),
            }
        } else {
            flags |= FieldFlags::TYPENAME;
        }

        let query_field = Node::QueryField(QueryField { id, flags });
        let query_field_ix = self.graph.add_node(query_field);
        self.field_nodes.push(query_field_ix);
        self.providable_fields_bitset.grow(self.field_nodes.len());
        self.deleted_fields_bitset.grow(self.field_nodes.len());
        query_field_ix
    }
}

#[derive(Default)]
enum ParentProvideResult<'ctx, FieldId> {
    Providable(ProvidableField<'ctx, FieldId>),
    UnreachableObject,
    #[default]
    NotProvidable,
}
