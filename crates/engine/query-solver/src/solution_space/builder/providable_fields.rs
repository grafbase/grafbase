use std::borrow::Cow;

use operation::{OperationContext, QueryInputValueRecord};
use petgraph::{stable_graph::NodeIndex, visit::EdgeRef, Direction};
use schema::{
    CompositeType, CompositeTypeId, EntityDefinition, FieldDefinition, FieldSet, FieldSetItem, FieldSetRecord,
    SchemaInputValueRecord, SubgraphId,
};
use walker::Walk;

use crate::{NodeFlags, QueryField, QueryOrSchemaFieldArgumentIds, QuerySelectionSet, QuerySelectionSetId};

use super::{ProvidableField, QueryFieldId, QuerySolutionSpaceBuilder, Resolver, SpaceEdge, SpaceNode};

pub(super) struct CreateRequirementTask<'schema> {
    pub petitioner_field_id: QueryFieldId,
    pub dependent_ix: NodeIndex,
    pub indispensable: bool,
    pub parent_selection_set_id: QuerySelectionSetId,
    pub required_field_set: FieldSet<'schema>,
}

#[derive(Clone)]
pub(super) struct Parent {
    pub selection_set_id: QuerySelectionSetId,
    pub providable_field_or_root_ix: NodeIndex,
}

pub(super) struct CreateProvidableFieldsTask {
    pub parent: Parent,
    pub query_field_node_ix: NodeIndex,
    pub query_field_id: QueryFieldId,
}

pub(super) struct UnplannableField {
    pub parent_selection_set_id: QuerySelectionSetId,
    pub query_field_node_ix: NodeIndex,
}

impl<'schema, 'op> QuerySolutionSpaceBuilder<'schema, 'op>
where
    'schema: 'op,
{
    pub(super) fn create_providable_fields_tasks_for_subselection(&mut self, parent: Parent) {
        let mut neighbors = self
            .query
            .graph
            .neighbors_directed(self.query[parent.selection_set_id].parent_node_ix, Direction::Outgoing)
            .detach();
        while let Some(node_ix) = neighbors.next_node(&self.query.graph) {
            match &self.query.graph[node_ix] {
                SpaceNode::QueryField { id, .. } => {
                    self.create_provideable_fields_task_stack
                        .push(CreateProvidableFieldsTask {
                            parent: parent.clone(),
                            query_field_node_ix: node_ix,
                            query_field_id: *id,
                        });
                }
                SpaceNode::Typename { .. } => {
                    if let SpaceNode::ProvidableField(providable_field) =
                        &self.query.graph[parent.providable_field_or_root_ix]
                    {
                        let QuerySelectionSet { output_type_id, .. } = self.query[parent.selection_set_id];
                        if output_type_id
                            .as_interface()
                            .map(|id| {
                                id.walk(self.schema)
                                    .is_interface_object_in_ids
                                    .contains(&providable_field.subgraph_id())
                            })
                            .unwrap_or_default()
                        {
                        } else {
                        }
                    } else {
                        debug_assert_eq!(parent.providable_field_or_root_ix, self.query.root_node_ix);
                        let resolver_definition_id = self.schema.subgraphs.introspection.resolver_definition_id;
                        let resolver_ix = if let Some(edge) = self
                            .query
                            .graph
                            .edges_directed(self.query.root_node_ix, Direction::Outgoing)
                            .find(|edge| match edge.weight() {
                                SpaceEdge::HasChildResolver { .. } => self.query.graph[edge.target()]
                                    .as_resolver()
                                    .is_some_and(|res| res.definition_id == resolver_definition_id),
                                _ => false,
                            }) {
                            edge.target()
                        } else {
                            let resolver_ix = self.query.graph.add_node(SpaceNode::Resolver(Resolver {
                                entity_definition_id: self.operation.root_object_id.into(),
                                definition_id: resolver_definition_id,
                            }));
                            self.query.graph.add_edge(
                                parent.providable_field_or_root_ix,
                                resolver_ix,
                                SpaceEdge::CreateChildResolver,
                            );
                            self.query.graph.add_edge(
                                self.query.root_node_ix,
                                resolver_ix,
                                SpaceEdge::HasChildResolver,
                            );
                            resolver_ix
                        };

                        self.query
                            .graph
                            .add_edge(resolver_ix, node_ix, SpaceEdge::ProvidesTypename);

                        self.query.graph[node_ix]
                            .flags_mut()
                            .unwrap()
                            .insert(NodeFlags::PROVIDABLE);
                    }
                }
                _ => (),
            }
        }
    }

    pub(super) fn create_providable_fields(
        &mut self,
        CreateProvidableFieldsTask {
            parent,
            query_field_node_ix,
            query_field_id,
        }: CreateProvidableFieldsTask,
    ) {
        let &QuerySelectionSet {
            parent_node_ix,
            output_type_id: parent_output,
            ..
        } = &self.query[parent.selection_set_id];
        let field_definition = self.query[query_field_id].definition_id.walk(self.schema);

        // --
        // If providable by parent, we don't need to find for a resolver.
        // --
        let provide_result = self.query.graph[parent.providable_field_or_root_ix]
            .as_providable_field()
            .map(|parent_providable_field| {
                self.provide_field_from_parent(parent_providable_field, parent_output, query_field_id, field_definition)
            })
            .unwrap_or_default();
        let could_be_provided_from_parent = match provide_result {
            ParentProvideResult::Providable(child) => {
                let providable_field_ix = self.query.graph.add_node(SpaceNode::ProvidableField(child));
                self.query.graph.add_edge(
                    parent.providable_field_or_root_ix,
                    providable_field_ix,
                    SpaceEdge::CanProvide,
                );
                self.query
                    .graph
                    .add_edge(providable_field_ix, query_field_node_ix, SpaceEdge::Provides);
                self.query.graph[query_field_node_ix]
                    .flags_mut()
                    .unwrap()
                    .insert(NodeFlags::PROVIDABLE);
                if let Some(selection_set_id) = self.query[query_field_id].selection_set_id {
                    self.create_providable_fields_tasks_for_subselection(Parent {
                        selection_set_id,
                        providable_field_or_root_ix: providable_field_ix,
                    });
                }
                true
            }
            ParentProvideResult::NotProvidable => false,
            ParentProvideResult::UnreachableObject => {
                self.query.graph[query_field_node_ix]
                    .flags_mut()
                    .unwrap()
                    .insert(NodeFlags::UNREACHABLE);
                self.maybe_unplannable_query_fields_stack.push(UnplannableField {
                    parent_selection_set_id: parent.selection_set_id,
                    query_field_node_ix,
                });
                return;
            }
        };

        let parent_subgraph_id = self.query.graph[parent.providable_field_or_root_ix]
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
                .query
                .graph
                .edges_directed(parent_node_ix, Direction::Outgoing)
                .find(|edge| match edge.weight() {
                    SpaceEdge::HasChildResolver { .. } => self.query.graph[edge.target()]
                        .as_resolver()
                        .is_some_and(|res| res.definition_id == resolver_definition.id),
                    _ => false,
                }) {
                let resolver_ix = edge.target();

                // If it does not exist already we a relation between the parent providable field
                // and the existing resolver. It may exist already as we needed this resolver for
                // another field.
                if !self
                    .query
                    .graph
                    .edges_directed(resolver_ix, Direction::Incoming)
                    .any(|edge| edge.source() == parent.providable_field_or_root_ix)
                {
                    self.query.graph.add_edge(
                        parent.providable_field_or_root_ix,
                        resolver_ix,
                        SpaceEdge::CreateChildResolver,
                    );
                }

                // A resolver node already exists within this selection set, so we don't need to
                // create one. The field itself might already have been processed through a
                // different path, so we check if there is any ProvidableField already providing the
                // current field.
                if self
                    .query
                    .graph
                    .edges_directed(resolver_ix, Direction::Outgoing)
                    .any(|edge| match edge.weight() {
                        SpaceEdge::CanProvide { .. } => self
                            .query
                            .graph
                            .edges_directed(edge.target(), Direction::Outgoing)
                            .any(|edge| {
                                matches!(edge.weight(), SpaceEdge::Provides) && edge.target() == query_field_node_ix
                            }),
                        _ => false,
                    })
                {
                    continue;
                }

                resolver_ix
            } else {
                let resolver_ix = self.query.graph.add_node(SpaceNode::Resolver(Resolver {
                    entity_definition_id: field_definition.parent_entity_id,
                    definition_id: resolver_definition.id,
                }));
                self.query.graph.add_edge(
                    parent.providable_field_or_root_ix,
                    resolver_ix,
                    SpaceEdge::CreateChildResolver,
                );
                self.query
                    .graph
                    .add_edge(parent_node_ix, resolver_ix, SpaceEdge::HasChildResolver);
                if let Some(required_field_set) = resolver_definition.required_field_set() {
                    self.create_requirement_task_stack.push(CreateRequirementTask {
                        parent_selection_set_id: parent.selection_set_id,
                        petitioner_field_id: query_field_id,
                        dependent_ix: resolver_ix,
                        indispensable: false,
                        required_field_set,
                    });
                };

                resolver_ix
            };

            let providable_field = ProvidableField::InSubgraph {
                subgraph_id: resolver_definition.subgraph_id(),
                field_id: query_field_id,
                provides: field_definition
                    .provides_for_subgraph(resolver_definition.subgraph_id())
                    .map(|field_set| Cow::Borrowed(field_set.as_ref()))
                    .unwrap_or(Cow::Borrowed(FieldSetRecord::empty())),
            };
            let providable_field_ix = self.query.graph.add_node(SpaceNode::ProvidableField(providable_field));

            // if the field has specific requirements for this subgraph we add it to the stack.
            if let Some(required_field_set) = field_definition.requires_for_subgraph(resolver_definition.subgraph_id())
            {
                self.create_requirement_task_stack.push(CreateRequirementTask {
                    parent_selection_set_id: parent.selection_set_id,
                    petitioner_field_id: query_field_id,
                    dependent_ix: providable_field_ix,
                    indispensable: false,
                    required_field_set,
                })
            }

            self.query
                .graph
                .add_edge(resolver_ix, providable_field_ix, SpaceEdge::CanProvide);
            self.query
                .graph
                .add_edge(providable_field_ix, query_field_node_ix, SpaceEdge::Provides);
            self.query.graph[query_field_node_ix]
                .flags_mut()
                .unwrap()
                .insert(NodeFlags::PROVIDABLE);

            if let Some(selection_set_id) = self.query[query_field_id].selection_set_id {
                self.create_providable_fields_tasks_for_subselection(Parent {
                    selection_set_id,
                    providable_field_or_root_ix: providable_field_ix,
                });
            }
        }

        let SpaceNode::QueryField { flags, .. } = &mut self.query.graph[query_field_node_ix] else {
            unreachable!()
        };
        if !flags.contains(NodeFlags::PROVIDABLE) {
            self.maybe_unplannable_query_fields_stack.push(UnplannableField {
                parent_selection_set_id: parent.selection_set_id,
                query_field_node_ix,
            });
        }
    }

    fn provide_field_from_parent(
        &self,
        parent: &ProvidableField<'schema>,
        parent_output: CompositeTypeId,
        id: QueryFieldId,
        field_definition: FieldDefinition<'schema>,
    ) -> ParentProvideResult<'schema> {
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
                        field_id: id,
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
                                field_id: id,
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
                        field_id: id,
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
        provides: &Cow<'schema, FieldSetRecord>,
        id: QueryFieldId,
        definition: FieldDefinition<'schema>,
    ) -> Option<Cow<'schema, FieldSetRecord>> {
        match provides {
            Cow::Borrowed(provides) => provides
                .iter()
                .find(|item| self.is_field_equivalent(id, item.walk(self.schema)))
                .map(|item| match definition.provides_for_subgraph(subgraph_id) {
                    Some(field_provides) => Cow::Owned(FieldSetRecord::union(
                        field_provides.as_ref(),
                        &item.subselection_record,
                    )),
                    None => Cow::Borrowed(&item.subselection_record),
                }),
            Cow::Owned(provides) => provides
                .iter()
                .find(|item| self.is_field_equivalent(id, item.walk(self.schema)))
                .map(|item| match definition.provides_for_subgraph(subgraph_id) {
                    Some(field_provides) => Cow::Owned(FieldSetRecord::union(
                        field_provides.as_ref(),
                        &item.subselection_record,
                    )),
                    None => Cow::Owned(item.subselection_record.clone()),
                }),
        }
    }

    pub(super) fn create_requirement(
        &mut self,
        CreateRequirementTask {
            petitioner_field_id,
            dependent_ix,
            indispensable,
            parent_selection_set_id,
            required_field_set,
        }: CreateRequirementTask<'schema>,
    ) {
        let &QuerySelectionSet {
            parent_node_ix,
            output_type_id,
            ..
        } = &self.query[parent_selection_set_id];

        for required_item in required_field_set.items() {
            // Find an existing field that satisfies the requirement.
            let existing_field = self
                .query
                .graph
                .edges_directed(parent_node_ix, Direction::Outgoing)
                .filter_map(|edge| {
                    if let SpaceNode::QueryField { id, .. } = self.query.graph[edge.target()] {
                        Some((edge.target(), id))
                    } else {
                        None
                    }
                })
                .filter(|(_, id)| self.is_field_equivalent(*id, required_item))
                // not sure if necessary but provides consistency
                .min_by_key(|(_, id)| *id);

            // Create the required field otherwise.
            let query_field_node_ix = if let Some((query_field_node_ix, id)) = existing_field {
                if let Some(id) = self.query[id].selection_set_id {
                    self.create_requirement_task_stack.push(CreateRequirementTask {
                        petitioner_field_id,
                        dependent_ix,
                        indispensable,
                        parent_selection_set_id: id,
                        required_field_set: required_item.subselection(),
                    });
                }
                query_field_node_ix
            } else {
                // Create the QueryField Node
                let query_field_id = self.query.fields.len().into();
                let query_field_node_ix = self.push_query_field_node(
                    query_field_id,
                    if indispensable {
                        NodeFlags::INDISPENSABLE
                    } else {
                        NodeFlags::empty()
                    },
                );
                let nested_selection_set_id = required_item
                    .field()
                    .definition()
                    .ty()
                    .definition_id
                    .as_composite_type()
                    .map(|output_type_id| {
                        let selection_set = QuerySelectionSet {
                            parent_node_ix: query_field_node_ix,
                            output_type_id,
                            typename_node_ix: None,
                            typename_fields: Vec::new(),
                        };

                        self.query.selection_sets.push(selection_set);
                        let id = (self.query.selection_sets.len() - 1).into();
                        self.create_requirement_task_stack.push(CreateRequirementTask {
                            petitioner_field_id,
                            dependent_ix,
                            indispensable,
                            parent_selection_set_id: id,
                            required_field_set: required_item.subselection(),
                        });

                        id
                    });

                self.query.fields.push(QueryField {
                    type_conditions: {
                        let start = self.query.shared_type_conditions.len();
                        let tyc = required_item.field().definition().parent_entity_id.as_composite_type();
                        if tyc != output_type_id {
                            self.query.shared_type_conditions.push(tyc);
                        }
                        (start..self.query.shared_type_conditions.len()).into()
                    },
                    query_position: None,
                    response_key: None,
                    subgraph_key: None,
                    definition_id: required_item.field().definition_id,
                    argument_ids: QueryOrSchemaFieldArgumentIds::Schema(required_item.field().sorted_argument_ids),
                    location: self.query[petitioner_field_id].location,
                    flat_directive_id: Default::default(),
                    selection_set_id: nested_selection_set_id,
                });
                self.providable_fields_bitset.push(false);
                self.deleted_fields_bitset.push(false);

                self.query
                    .graph
                    .add_edge(parent_node_ix, query_field_node_ix, SpaceEdge::Field);
                self.create_providable_fields_task_for_new_field(
                    parent_selection_set_id,
                    query_field_node_ix,
                    query_field_id,
                );

                query_field_node_ix
            };

            self.query
                .graph
                .add_edge(dependent_ix, query_field_node_ix, SpaceEdge::Requires);
        }
    }

    pub(super) fn create_providable_fields_task_for_new_field(
        &mut self,
        parent_selection_set_id: QuerySelectionSetId,
        query_field_node_ix: NodeIndex,
        query_field_id: QueryFieldId,
    ) {
        let selection_set = &self.query[parent_selection_set_id];
        if selection_set.parent_node_ix == self.query.root_node_ix {
            self.create_provideable_fields_task_stack
                .push(CreateProvidableFieldsTask {
                    parent: Parent {
                        selection_set_id: parent_selection_set_id,
                        providable_field_or_root_ix: self.query.root_node_ix,
                    },
                    query_field_node_ix,
                    query_field_id,
                });
        } else {
            // For all the ProvidableField which may provide the parent QueryField, we have
            // to try whether they can provide this newly added nested QueryField
            self.create_provideable_fields_task_stack.extend(
                self.query
                    .graph
                    .edges_directed(selection_set.parent_node_ix, Direction::Incoming)
                    .filter(|edge| {
                        matches!(edge.weight(), SpaceEdge::Provides)
                            && self.query.graph[edge.source()].is_providable_field()
                    })
                    .map(|edge| CreateProvidableFieldsTask {
                        parent: Parent {
                            selection_set_id: parent_selection_set_id,
                            providable_field_or_root_ix: edge.source(),
                        },
                        query_field_node_ix,
                        query_field_id,
                    }),
            );
        }
    }

    fn is_field_equivalent(&self, id: QueryFieldId, required: FieldSetItem<'_>) -> bool {
        let actual = &self.query[id];
        let required = required.field().as_ref();

        if actual.definition_id != required.definition_id {
            return false;
        }

        match actual.argument_ids {
            QueryOrSchemaFieldArgumentIds::Query(argument_ids) => {
                if argument_ids.len() != required.sorted_argument_ids.len() {
                    return false;
                }

                for argument in &self.operation[argument_ids] {
                    let definition_id = argument.definition_id;
                    let actual_input_value = &self.operation.query_input_values[argument.value_id];
                    if !self.schema[required.sorted_argument_ids]
                        .iter()
                        .find(|arg| arg.definition_id.eq(&definition_id))
                        .map(|required_arg| {
                            self.is_value_equivalent(actual_input_value, &self.schema[required_arg.value_id])
                        })
                        .unwrap_or_default()
                    {
                        return false;
                    }
                }
            }
            QueryOrSchemaFieldArgumentIds::Schema(argument_ids) => {
                if argument_ids.len() != required.sorted_argument_ids.len() {
                    return false;
                }
                for argument in &self.schema[argument_ids] {
                    let definition_id = argument.definition_id;
                    let actual_input_value = &self.schema[argument.value_id];
                    if !self.schema[required.sorted_argument_ids]
                        .iter()
                        .find(|arg| arg.definition_id.eq(&definition_id))
                        .map(|required_arg| {
                            actual_input_value
                                .walk(self.schema)
                                .eq(&required_arg.value_id.walk(self.schema))
                        })
                        .unwrap_or_default()
                    {
                        return false;
                    }
                }
            }
        }

        true
    }

    fn is_value_equivalent(&self, actual: &QueryInputValueRecord, required: &SchemaInputValueRecord) -> bool {
        let ctx = OperationContext {
            schema: self.schema,
            operation: self.operation,
        };
        operation::is_query_value_equivalent_schema_value(ctx, actual, required)
    }
}

#[derive(Default)]
enum ParentProvideResult<'schema> {
    Providable(ProvidableField<'schema>),
    UnreachableObject,
    #[default]
    NotProvidable,
}
