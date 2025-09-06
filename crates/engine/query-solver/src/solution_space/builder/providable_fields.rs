use std::borrow::Cow;

use operation::{OperationContext, QueryInputValueRecord};
use petgraph::{Direction, stable_graph::NodeIndex, visit::EdgeRef};
use schema::{
    CompositeType, CompositeTypeId, DeriveMapping, EntityDefinition, FieldDefinition, FieldSet, FieldSetItem,
    FieldSetRecord, ResolverDefinitionRecord, SchemaInputValueRecord, SubgraphId,
};
use walker::Walk;

use crate::{Derive, FieldFlags, QueryField, QueryOrSchemaSortedFieldArgumentIds, SpaceFieldSetId};

use super::{ProvidableField, QueryFieldId, QuerySolutionSpaceBuilder, Resolver, SpaceEdge, SpaceNode};

pub(super) struct CreateRequirementTask<'schema> {
    pub petitioner_field_id: QueryFieldId,
    pub dependent_id: NodeIndex,
    pub indispensable: bool,
    pub parent_query_field_node_id: NodeIndex,
    pub parent_output_type: CompositeTypeId,
    pub required_field_set: FieldSet<'schema>,
}

#[derive(Clone)]
pub(super) struct Parent {
    pub query_field_node_id: NodeIndex,
    pub output_type: CompositeTypeId,
    pub providable_field_or_root_id: NodeIndex,
}

pub(super) struct CreateProvidableFieldsTask {
    pub parent: Parent,
    pub query_field_node_id: NodeIndex,
    pub query_field_id: QueryFieldId,
}

pub(super) struct UnplannableField {
    pub parent_query_field_node_id: NodeIndex,
    pub query_field_node_id: NodeIndex,
}

impl<'schema, 'op> QuerySolutionSpaceBuilder<'schema, 'op>
where
    'schema: 'op,
{
    pub(super) fn create_providable_fields_tasks_for_subselection(&mut self, parent: Parent) {
        for subfield_id in self
            .query
            .graph
            .neighbors_directed(parent.query_field_node_id, Direction::Outgoing)
        {
            if let SpaceNode::Field(subfield) = &self.query.graph[subfield_id] {
                self.create_provideable_fields_task_stack
                    .push(CreateProvidableFieldsTask {
                        parent: parent.clone(),
                        query_field_node_id: subfield_id,
                        query_field_id: subfield.id,
                    });
            }
        }
    }

    pub(super) fn create_providable_fields(
        &mut self,
        CreateProvidableFieldsTask {
            parent,
            query_field_node_id,
            query_field_id,
        }: CreateProvidableFieldsTask,
    ) {
        let query_field = &self.query[query_field_id];

        let Some(definition_id) = query_field.definition_id else {
            self.query.graph[query_field_node_id]
                .as_query_field_mut()
                .unwrap()
                .flags |= FieldFlags::PROVIDABLE;
            return;
        };

        let field_definition = definition_id.walk(self.schema);

        // --
        // If providable by parent, we don't need to find for a resolver.
        // --
        let provide_result = self.query.graph[parent.providable_field_or_root_id]
            .as_providable_field()
            .copied()
            .map(|parent_providable_field| {
                self.provide_field_from_parent(
                    parent_providable_field,
                    parent.output_type,
                    query_field_id,
                    field_definition,
                )
            })
            .unwrap_or_default();
        let could_be_provided_from_parent = match provide_result {
            ParentProvideResult::Providable(child) => {
                let providable_field_id = self.query.graph.add_node(SpaceNode::ProvidableField(child));
                self.query.graph.add_edge(
                    parent.providable_field_or_root_id,
                    providable_field_id,
                    SpaceEdge::CanProvide,
                );
                self.query
                    .graph
                    .add_edge(providable_field_id, query_field_node_id, SpaceEdge::Provides);
                self.query.graph[query_field_node_id]
                    .as_query_field_mut()
                    .unwrap()
                    .flags |= FieldFlags::PROVIDABLE;

                if let Some(output_type) = field_definition.ty().definition_id.as_composite_type() {
                    self.create_providable_fields_tasks_for_subselection(Parent {
                        query_field_node_id,
                        providable_field_or_root_id: providable_field_id,
                        output_type,
                    });
                }
                true
            }
            ParentProvideResult::NotProvidable => false,
            ParentProvideResult::UnreachableObject => {
                self.query.graph[query_field_node_id]
                    .as_query_field_mut()
                    .unwrap()
                    .flags |= FieldFlags::UNREACHABLE;
                self.maybe_unplannable_query_fields_stack.push(UnplannableField {
                    parent_query_field_node_id: parent.query_field_node_id,
                    query_field_node_id,
                });
                return;
            }
        };

        let parent_subgraph_id = self.query.graph[parent.providable_field_or_root_id]
            .as_providable_field()
            .map(|field| field.resolver_definition_id.walk(self.schema).subgraph_id());

        // --
        // Try to plan this field with alternative resolvers if any exist.
        // --
        for resolver_definition in field_definition.resolvers() {
            // If we could provide from the current resolver within the same subgraph, we skip it.
            if could_be_provided_from_parent && Some(resolver_definition.subgraph_id()) == parent_subgraph_id {
                continue;
            };

            if resolver_definition.is_lookup()
                && !self
                    .is_field_connected_to_parent_resolver(
                        resolver_definition.subgraph_id(),
                        parent.output_type,
                        field_definition,
                    )
                    .is_yes()
            {
                continue;
            }

            // Try to find an existing resolver node if a sibling field already added it, otherwise
            // create one.
            let resolver_node_id = if let Some(edge) = self
                .query
                .graph
                .edges_directed(parent.query_field_node_id, Direction::Outgoing)
                .find(|edge| match edge.weight() {
                    SpaceEdge::HasChildResolver => self.query.graph[edge.target()]
                        .as_resolver()
                        .is_some_and(|res| res.definition_id == resolver_definition.id),
                    _ => false,
                }) {
                let resolver_node_id = edge.target();

                // If it does not exist already we a relation between the parent providable field
                // and the existing resolver. It may exist already as we needed this resolver for
                // another field.
                if !self
                    .query
                    .graph
                    .edges_directed(resolver_node_id, Direction::Incoming)
                    .any(|edge| edge.source() == parent.providable_field_or_root_id)
                {
                    self.query.graph.add_edge(
                        parent.providable_field_or_root_id,
                        resolver_node_id,
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
                    .edges_directed(resolver_node_id, Direction::Outgoing)
                    .any(|edge| match edge.weight() {
                        SpaceEdge::CanProvide => {
                            if let SpaceNode::ProvidableField(field) = &self.query.graph[edge.target()] {
                                field.query_field_id == query_field_id
                            } else {
                                false
                            }
                        }
                        _ => false,
                    })
                {
                    continue;
                }

                resolver_node_id
            } else {
                let resolver_node_id = self.query.graph.add_node(SpaceNode::Resolver(Resolver {
                    entity_definition_id: field_definition.parent_entity_id,
                    definition_id: resolver_definition.id,
                }));
                self.query.graph.add_edge(
                    parent.providable_field_or_root_id,
                    resolver_node_id,
                    SpaceEdge::CreateChildResolver,
                );
                self.query.graph.add_edge(
                    parent.query_field_node_id,
                    resolver_node_id,
                    SpaceEdge::HasChildResolver,
                );
                if let Some(required_field_set) = resolver_definition.required_field_set() {
                    self.create_requirement_task_stack.push(CreateRequirementTask {
                        petitioner_field_id: query_field_id,
                        dependent_id: resolver_node_id,
                        indispensable: false,
                        parent_query_field_node_id: parent.query_field_node_id,
                        parent_output_type: parent.output_type,
                        required_field_set,
                    });
                };

                resolver_node_id
            };

            let providable_field = ProvidableField {
                resolver_definition_id: resolver_definition.id,
                query_field_id,
                provides: field_definition
                    .provides_for_subgraph(resolver_definition.subgraph_id())
                    .map(|field_set| self.query.push_field_set_ref(field_set.as_ref())),
                only_providable: false,
                derive_id: None,
            };
            let providable_field_id = self.query.graph.add_node(SpaceNode::ProvidableField(providable_field));

            // if the field has specific requirements for this subgraph we add it to the stack.
            if let Some(requires) = field_definition.requires_for_subgraph(resolver_definition.subgraph_id()) {
                self.create_requirement_task_stack.push(CreateRequirementTask {
                    petitioner_field_id: query_field_id,
                    dependent_id: providable_field_id,
                    indispensable: false,
                    parent_query_field_node_id: parent.query_field_node_id,
                    parent_output_type: parent.output_type,
                    required_field_set: requires.field_set(),
                })
            }

            self.query
                .graph
                .add_edge(resolver_node_id, providable_field_id, SpaceEdge::CanProvide);
            self.query
                .graph
                .add_edge(providable_field_id, query_field_node_id, SpaceEdge::Provides);
            self.query.graph[query_field_node_id]
                .as_query_field_mut()
                .unwrap()
                .flags |= FieldFlags::PROVIDABLE;

            if let Some(output_type) = field_definition.ty().definition_id.as_composite_type() {
                self.create_providable_fields_tasks_for_subselection(Parent {
                    query_field_node_id,
                    providable_field_or_root_id: providable_field_id,
                    output_type,
                });
            }
        }

        let SpaceNode::Field(field) = &mut self.query.graph[query_field_node_id] else {
            unreachable!()
        };
        if !field.flags.contains(FieldFlags::PROVIDABLE) {
            self.maybe_unplannable_query_fields_stack.push(UnplannableField {
                parent_query_field_node_id: parent.query_field_node_id,
                query_field_node_id,
            });
        }
    }

    fn provide_field_from_parent(
        &mut self,
        parent: ProvidableField,
        parent_output: CompositeTypeId,
        id: QueryFieldId,
        field_definition: FieldDefinition<'schema>,
    ) -> ParentProvideResult {
        let parent_subgraph_id = parent.resolver_definition_id.walk(self.schema).subgraph_id();
        if let Some(derive) = parent.derive_id.map(|id| self.query.step[id]) {
            let Derive::Root { id: derive_id } = derive else {
                unreachable!("Nested @derive fields aren't support yet.")
            };
            let derive = derive_id.walk(self.schema);
            match derive.mapping() {
                DeriveMapping::Object(derive_object) => {
                    if let Some(mapping) = derive_object.fields().find(|map| map.to_id == field_definition.id) {
                        ParentProvideResult::Providable(ProvidableField {
                            resolver_definition_id: parent.resolver_definition_id,
                            query_field_id: id,
                            provides: parent.provides,
                            only_providable: false,
                            derive_id: Some(self.query.push_derive(Derive::Field {
                                from_id: mapping.from_id,
                            })),
                        })
                    } else {
                        ParentProvideResult::NotProvidable
                    }
                }
                DeriveMapping::ScalarAsField(mapping) => {
                    if mapping.field_id == field_definition.id {
                        ParentProvideResult::Providable(ProvidableField {
                            resolver_definition_id: parent.resolver_definition_id,
                            query_field_id: id,
                            provides: Default::default(),
                            only_providable: false,
                            derive_id: Some(self.query.push_derive(Derive::ScalarAsField)),
                        })
                    } else {
                        ParentProvideResult::NotProvidable
                    }
                }
            }
        } else if let Some(derive) = field_definition
            .derives()
            .find(|derived| derived.subgraph_id == parent_subgraph_id)
        {
            ParentProvideResult::Providable(ProvidableField {
                resolver_definition_id: parent.resolver_definition_id,
                query_field_id: id,
                provides: parent.provides,
                only_providable: false,
                derive_id: Some(self.query.push_derive(Derive::Root { id: derive.id })),
            })
        } else if parent.only_providable {
            self.find_in_provides(parent_subgraph_id, parent.provides, id, field_definition)
                .map(|provides| {
                    ParentProvideResult::Providable(ProvidableField {
                        resolver_definition_id: parent.resolver_definition_id,
                        query_field_id: id,
                        provides,
                        only_providable: true,
                        derive_id: None,
                    })
                })
                .unwrap_or_default()
        } else {
            match self.is_field_connected_to_parent_resolver(parent_subgraph_id, parent_output, field_definition) {
                IsFieldConnectedToParentResolver::Yes => ParentProvideResult::Providable(ProvidableField {
                    resolver_definition_id: parent.resolver_definition_id,
                    query_field_id: id,
                    provides: match self.find_in_provides(parent_subgraph_id, parent.provides, id, field_definition) {
                        Some(provides) => provides,
                        None => field_definition
                            .provides_for_subgraph(parent_subgraph_id)
                            .map(|field_set| self.query.push_field_set_ref(field_set.as_ref())),
                    },
                    only_providable: false,
                    derive_id: None,
                }),
                IsFieldConnectedToParentResolver::No { is_reachable } => self
                    .find_in_provides(parent_subgraph_id, parent.provides, id, field_definition)
                    .map(|provides| {
                        ParentProvideResult::Providable(ProvidableField {
                            resolver_definition_id: parent.resolver_definition_id,
                            query_field_id: id,
                            provides,
                            only_providable: true,
                            derive_id: None,
                        })
                    })
                    .unwrap_or_else(|| {
                        if is_reachable {
                            ParentProvideResult::NotProvidable
                        } else {
                            ParentProvideResult::UnreachableObject
                        }
                    }),
            }
        }
    }

    fn is_field_connected_to_parent_resolver(
        &self,
        parent_subgraph_id: SubgraphId,
        parent_output: CompositeTypeId,
        field_definition: FieldDefinition<'_>,
    ) -> IsFieldConnectedToParentResolver {
        let is_reachable = self.is_field_parent_object_reachable_in_subgraph_from_parent_output(
            parent_subgraph_id,
            parent_output,
            field_definition,
        );

        // Either it's a GraphQL endpoint and anything we can reach (within the subgraph) is necessarily provideable or it's a virtual
        // one and we need to ensure there isn't any extension resolver defined for this field.
        let doesnt_require_dedicated_resolver = parent_subgraph_id.is_graphql()
            || field_definition.resolvers().all(|r| {
                r.subgraph_id() != parent_subgraph_id
                    || !matches!(
                        r.as_ref(),
                        ResolverDefinitionRecord::Extension(_)
                            | ResolverDefinitionRecord::FieldResolverExtension(_)
                            | ResolverDefinitionRecord::SelectionSetResolverExtension(_)
                    )
            });
        if is_reachable
            && doesnt_require_dedicated_resolver
            && self.is_field_providable_in_subgraph(parent_subgraph_id, field_definition)
            && field_definition.requires_for_subgraph(parent_subgraph_id).is_none()
        {
            IsFieldConnectedToParentResolver::Yes
        } else {
            IsFieldConnectedToParentResolver::No { is_reachable }
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
        &mut self,
        subgraph_id: SubgraphId,
        provides: Option<SpaceFieldSetId>,
        id: QueryFieldId,
        definition: FieldDefinition<'schema>,
    ) -> Option<Option<SpaceFieldSetId>> {
        let field_set_id = provides?;
        match &self.query.step[field_set_id] {
            Cow::Borrowed(provides) => provides
                .iter()
                .find(|item| self.is_field_equivalent(id, item.walk(self.schema)))
                .map(|item| match definition.provides_for_subgraph(subgraph_id) {
                    Some(field_provides) => Some(Cow::Owned(FieldSetRecord::union(
                        field_provides.as_ref(),
                        &item.subselection_record,
                    ))),
                    None if !item.subselection_record.is_empty() => Some(Cow::Borrowed(&item.subselection_record)),
                    _ => None,
                }),
            Cow::Owned(provides) => provides
                .iter()
                .find(|item| self.is_field_equivalent(id, item.walk(self.schema)))
                .map(|item| match definition.provides_for_subgraph(subgraph_id) {
                    Some(field_provides) => Some(Cow::Owned(FieldSetRecord::union(
                        field_provides.as_ref(),
                        &item.subselection_record,
                    ))),
                    None if !item.subselection_record.is_empty() => Some(Cow::Owned(item.subselection_record.clone())),
                    _ => None,
                }),
        }
        .map(|maybe_nested_provides| maybe_nested_provides.map(|field_set| self.query.push_field_set(field_set)))
    }

    pub(super) fn create_requirement(
        &mut self,
        CreateRequirementTask {
            petitioner_field_id,
            dependent_id,
            indispensable,
            parent_query_field_node_id,
            parent_output_type,
            required_field_set,
        }: CreateRequirementTask<'schema>,
    ) {
        for required_item in required_field_set.items() {
            // Find an existing field that satisfies the requirement.
            let mut existing_field = None;
            for (node_id, id) in self
                .query
                .graph
                .edges_directed(parent_query_field_node_id, Direction::Outgoing)
                .filter_map(|edge| {
                    if matches!(edge.weight(), SpaceEdge::Field) {
                        self.query.graph[edge.target()]
                            .as_query_field()
                            .map(|field| (edge.target(), field.id))
                    } else {
                        None
                    }
                })
            {
                // Either we take a field that has already been used for this requirement, or we
                // find a new one. If the former exists, it must always be re-used.
                let field = &self.query[id];
                if field.matching_field_id == Some(required_item.field_id) {
                    existing_field = Some((node_id, id));
                    break;
                }
                if self.is_field_equivalent(id, required_item) {
                    existing_field = Some((node_id, id));
                    break;
                }
            }

            // Create the required field otherwise.
            let (query_field_node_id, query_field_id) =
                if let Some((query_field_node_ix, query_field_id)) = existing_field {
                    self.query[query_field_id].matching_field_id = Some(required_item.field_id);
                    (query_field_node_ix, query_field_id)
                } else {
                    // Create the QueryField Node
                    let query_field_id = self.query.fields.len().into();
                    let type_conditions = {
                        let start = self.query.shared_type_conditions.len();
                        let tyc = required_item.field().definition().parent_entity_id.as_composite_type();
                        if tyc != parent_output_type {
                            self.query.shared_type_conditions.push(tyc);
                        }
                        (start..self.query.shared_type_conditions.len()).into()
                    };
                    let flat_directive_id = None;
                    let definition_id = required_item.field().definition_id;
                    let sorted_argument_ids =
                        QueryOrSchemaSortedFieldArgumentIds::Schema(required_item.field().sorted_argument_ids);
                    self.query.fields.push(QueryField {
                        type_conditions,
                        query_position: None,
                        response_key: None,
                        definition_id: Some(definition_id),
                        matching_field_id: Some(required_item.field_id),
                        sorted_argument_ids,
                        location: self.query[petitioner_field_id].location,
                        flat_directive_id,
                    });

                    let query_field_node_id = self.push_query_field_node(
                        query_field_id,
                        if indispensable {
                            FieldFlags::EXTRA | FieldFlags::INDISPENSABLE
                        } else {
                            FieldFlags::EXTRA
                        },
                    );
                    self.query
                        .graph
                        .add_edge(parent_query_field_node_id, query_field_node_id, SpaceEdge::Field);
                    self.create_providable_fields_task_for_new_field(
                        parent_query_field_node_id,
                        parent_output_type,
                        query_field_node_id,
                        query_field_id,
                    );

                    (query_field_node_id, query_field_id)
                };

            self.query
                .graph
                .add_edge(dependent_id, query_field_node_id, SpaceEdge::Requires);

            if let Some(output_type) = self.query[query_field_id]
                .definition_id
                .and_then(|def| def.walk(self.schema).ty().definition_id.as_composite_type())
            {
                self.create_requirement_task_stack.push(CreateRequirementTask {
                    petitioner_field_id,
                    dependent_id,
                    indispensable,
                    parent_query_field_node_id: query_field_node_id,
                    parent_output_type: output_type,
                    required_field_set: required_item.subselection(),
                })
            }
        }
    }

    pub(super) fn create_providable_fields_task_for_new_field(
        &mut self,
        parent_query_field_node_id: NodeIndex,
        parent_output_type: CompositeTypeId,
        query_field_node_id: NodeIndex,
        query_field_id: QueryFieldId,
    ) {
        if parent_query_field_node_id == self.query.root_node_id {
            self.create_provideable_fields_task_stack
                .push(CreateProvidableFieldsTask {
                    parent: Parent {
                        query_field_node_id: parent_query_field_node_id,
                        output_type: parent_output_type,
                        providable_field_or_root_id: self.query.root_node_id,
                    },
                    query_field_node_id,
                    query_field_id,
                });
        } else {
            // For all the ProvidableField which may provide the parent QueryField, we have
            // to try whether they can provide this newly added nested QueryField
            self.create_provideable_fields_task_stack.extend(
                self.query
                    .graph
                    .edges_directed(parent_query_field_node_id, Direction::Incoming)
                    .filter(|edge| {
                        matches!(edge.weight(), SpaceEdge::Provides)
                            && self.query.graph[edge.source()].is_providable_field()
                    })
                    .map(|edge| CreateProvidableFieldsTask {
                        parent: Parent {
                            query_field_node_id: parent_query_field_node_id,
                            output_type: parent_output_type,
                            providable_field_or_root_id: edge.source(),
                        },
                        query_field_node_id,
                        query_field_id,
                    }),
            );
        }
    }

    fn is_field_equivalent(&self, id: QueryFieldId, required: FieldSetItem<'_>) -> bool {
        let actual = &self.query[id];
        let required = required.field().as_ref();

        let Some(definition_id) = actual.definition_id else {
            return false;
        };
        if definition_id != required.definition_id {
            return false;
        }

        match actual.sorted_argument_ids {
            QueryOrSchemaSortedFieldArgumentIds::Query(argument_ids) => {
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
            QueryOrSchemaSortedFieldArgumentIds::Schema(argument_ids) => {
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
enum ParentProvideResult {
    Providable(ProvidableField),
    UnreachableObject,
    #[default]
    NotProvidable,
}

enum IsFieldConnectedToParentResolver {
    Yes,
    No { is_reachable: bool },
}

impl IsFieldConnectedToParentResolver {
    fn is_yes(&self) -> bool {
        matches!(self, IsFieldConnectedToParentResolver::Yes)
    }
}
