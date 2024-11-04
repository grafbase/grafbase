use id_newtypes::{IdRange, IdToMany};
use query_planning::{Edge, EdgeIndex, EdgeRef, Node, NodeIndex, SolvedGraph};
use schema::{ResolverDefinitionId, Schema};

use crate::{
    operation::{
        BoundField, BoundFieldArgument, BoundFieldArgumentId, BoundFieldId, BoundVariableDefinition,
        BoundVariableDefinitionId, Operation,
    },
    utils::BufferPool,
};

use super::{
    error::PlanError, generated::FieldId, prelude::PositionedResponseKey, DataFieldRecord, FieldArgumentId,
    FieldArgumentRecord, FieldRecord, OperationPlan, PlanId, PlanRecord, PlanResult, QueryModifierRecord,
    ResponseModifierRecord, TypenameFieldRecord, VariableDefinitionId, VariableDefinitionRecord,
};

impl OperationPlan {
    pub(super) fn build(
        schema: &Schema,
        mut operation: Operation,
        graph: SolvedGraph<BoundFieldId>,
    ) -> PlanResult<Self> {
        OperationPlanBuilder {
            schema,
            operation_plan: OperationPlan {
                fields: Vec::with_capacity(operation.fields.len()), // rough order of magnitude
                field_arguments: operation.field_arguments.drain(..).map(Into::into).collect(),
                variable_definitions: operation.variable_definitions.drain(..).map(Into::into).collect(),
                plans: Vec::new(),
                field_refs: Vec::new(),
                query_input_values: std::mem::take(&mut operation.query_input_values),
                query_modifiers: Vec::new(),
                response_modifiers: Vec::new(),
            },
            operation,
            graph,
            nested_fields_buffer: BufferPool::default(),
            edges_buffer: BufferPool::default(),
            plan_stack: Vec::new(),
            query_field_node_to_field: Vec::new(),
            field_to_providable_field_node: Vec::new(),
            plan_to_resolver_node: Vec::new(),
            bound_field_to_field: Vec::new(),
        }
        .build()
    }
}

struct OperationPlanBuilder<'a> {
    #[allow(unused)]
    schema: &'a Schema,
    operation_plan: OperationPlan,
    graph: SolvedGraph<BoundFieldId>,
    operation: Operation,
    nested_fields_buffer: BufferPool<NestedField>,
    edges_buffer: BufferPool<EdgeIndex>,
    plan_stack: Vec<PlanToCreate>,
    // one to one, sorted after the plan generation.
    query_field_node_to_field: Vec<(NodeIndex, FieldId)>,
    bound_field_to_field: Vec<(BoundFieldId, FieldId)>,
    // Populated during plan generation, drained while populating requirements.
    field_to_providable_field_node: Vec<(FieldId, NodeIndex)>,
    // Populated during plan generation, drained while populating requirements.
    plan_to_resolver_node: Vec<(PlanId, NodeIndex)>,
}

struct PlanToCreate {
    resolver_node_ix: NodeIndex,
    resolver_definition_id: ResolverDefinitionId,
}

struct NestedField {
    record: FieldRecord,
    bound_field_id: BoundFieldId,
    providable_field_node_ix: Option<NodeIndex>,
    query_field_node_ix: NodeIndex,
}

impl OperationPlanBuilder<'_> {
    fn build(mut self) -> PlanResult<OperationPlan> {
        for edge in self.graph.edges(self.graph.root_node_ix) {
            if matches!(edge.weight(), Edge::CreateChildResolver) {
                if let Node::Resolver(resolver) = &self.graph[edge.target()] {
                    self.plan_stack.push(PlanToCreate {
                        resolver_node_ix: edge.target(),
                        resolver_definition_id: resolver.definition_id,
                    });
                }
            }
        }

        // Generate all plans and their provided fields.
        while let Some(plan_to_create) = self.plan_stack.pop() {
            self.generate_plan(plan_to_create);
        }

        self.populate_requirements_after_plan_generation()?;

        self.popuplate_modifiers_after_plan_generation()?;

        Ok(self.operation_plan)
    }

    fn generate_plan(
        &mut self,
        PlanToCreate {
            resolver_node_ix,
            resolver_definition_id,
        }: PlanToCreate,
    ) {
        let field_ids = self.generate_selection_set_fields(resolver_node_ix);
        self.operation_plan.plans.push(PlanRecord {
            resolver_definition_id,
            field_ids,
            required_field_ids: IdRange::empty(),
        });
        let plan_id = PlanId::from(self.operation_plan.plans.len() - 1);
        self.plan_to_resolver_node.push((plan_id, resolver_node_ix));
    }

    fn generate_selection_set_fields(&mut self, node_ix: NodeIndex) -> IdRange<FieldId> {
        let mut fields = self.nested_fields_buffer.pop();
        let mut provided_query_field_ix = None;
        let mut edges = self.graph.neighbors(node_ix).detach();
        while let Some((edge, neighbor)) = edges.next(&self.graph) {
            match self.graph[edge] {
                // All nested providable fields which themselves provide a query field.
                Edge::CanProvide => {
                    if let Some((query_field_node_ix, field)) =
                        self.graph.edges(neighbor).find_map(|edge| match edge.weight() {
                            Edge::Provides => {
                                if let Node::QueryField(field) = &self.graph[edge.target()] {
                                    Some((edge.target(), field))
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        })
                    {
                        let bound_field_id = field.id;
                        let mut record = FieldRecord::from(&self.operation[bound_field_id]);
                        if let FieldRecord::Data(field) = &mut record {
                            field.selection_set_field_ids = self.generate_selection_set_fields(neighbor);
                        }
                        fields.push(NestedField {
                            record,
                            bound_field_id,
                            providable_field_node_ix: Some(neighbor),
                            query_field_node_ix,
                        });
                    }
                }
                // Which field is provided by the current node if any. Resolvers won't provide any
                // but they still have nested fields.
                Edge::Provides => {
                    if let Node::QueryField(_) = &self.graph[neighbor] {
                        provided_query_field_ix = Some(neighbor);
                    }
                }
                _ => (),
            }
        }

        // We take any non-attributed __typename field and resolver.
        if let Some(provided_query_field_ix) = provided_query_field_ix {
            let mut edges_to_remove = self.edges_buffer.pop();
            for edge in self.graph.edges(provided_query_field_ix) {
                match edge.weight() {
                    Edge::HasChildResolver => {
                        if let Node::Resolver(resolver) = &self.graph[edge.target()] {
                            self.plan_stack.push(PlanToCreate {
                                resolver_node_ix: edge.target(),
                                resolver_definition_id: resolver.definition_id,
                            });
                        }
                        edges_to_remove.push(edge.id());
                    }
                    Edge::TypenameField => {
                        if let Node::QueryField(field) = &self.graph[edge.target()] {
                            let bound_field = &self.operation[field.id];
                            debug_assert!(matches!(bound_field, BoundField::TypeName(_)));
                            fields.push(NestedField {
                                record: bound_field.into(),
                                bound_field_id: field.id,
                                providable_field_node_ix: None,
                                query_field_node_ix: edge.target(),
                            })
                        }
                        edges_to_remove.push(edge.id());
                    }
                    _ => (),
                }
            }
            for edge_ix in edges_to_remove.drain(..) {
                self.graph.remove_edge(edge_ix);
            }
            self.edges_buffer.push(edges_to_remove);
        }

        let start = self.operation_plan.fields.len();
        for NestedField {
            record,
            bound_field_id,
            providable_field_node_ix,
            query_field_node_ix,
        } in fields.drain(..)
        {
            self.operation_plan.fields.push(record);
            let field_id = FieldId::from(self.operation_plan.fields.len() - 1);
            self.bound_field_to_field.push((bound_field_id, field_id));

            if let Some(providable_field_node_ix) = providable_field_node_ix {
                self.field_to_providable_field_node
                    .push((field_id, providable_field_node_ix));
                self.query_field_node_to_field.push((query_field_node_ix, field_id))
            }
        }
        self.nested_fields_buffer.push(fields);

        IdRange::from(start..self.operation_plan.fields.len())
    }

    fn populate_requirements_after_plan_generation(&mut self) -> PlanResult<()> {
        self.query_field_node_to_field.sort_unstable();

        for (plan_id, resolver_node_ix) in std::mem::take(&mut self.plan_to_resolver_node) {
            let start = self.operation_plan.field_refs.len();
            for edge in self.graph.edges(resolver_node_ix) {
                if !matches!(edge.weight(), Edge::Requires) {
                    continue;
                }
                self.operation_plan
                    .field_refs
                    .push(self.get_field_id_for(edge.target()).ok_or_else(|| {
                        tracing::error!("Plan depends on an unknown query field node.");
                        PlanError::InternalError
                    })?);
            }
            self.operation_plan[plan_id].required_field_ids =
                IdRange::from(start..self.operation_plan.field_refs.len());
        }

        for (field_id, providable_field_node_ix) in std::mem::take(&mut self.field_to_providable_field_node) {
            let start = self.operation_plan.field_refs.len();

            for edge in self.graph.edges(providable_field_node_ix) {
                match edge.weight() {
                    Edge::Provides => {
                        debug_assert!(matches!(self.graph[edge.target()], Node::QueryField(_)));
                        for edge in self.graph.edges(edge.target()) {
                            if matches!(edge.weight(), Edge::Requires) {
                                self.operation_plan
                                    .field_refs
                                    .push(self.get_field_id_for(edge.target()).ok_or_else(|| {
                                        tracing::error!("Field depends on an unknown query field node.");
                                        PlanError::InternalError
                                    })?);
                            }
                        }
                    }
                    Edge::Requires => {
                        self.operation_plan
                            .field_refs
                            .push(self.get_field_id_for(edge.target()).ok_or_else(|| {
                                tracing::error!("Field depends on an unknown query field node.");
                                PlanError::InternalError
                            })?);
                    }
                    _ => (),
                }
            }

            let required_field_ids = IdRange::from(start..self.operation_plan.field_refs.len());
            let FieldRecord::Data(field) = &mut self.operation_plan[field_id] else {
                tracing::error!("Typename cannot have requirements.");
                return Err(PlanError::InternalError);
            };
            field.required_field_ids = required_field_ids;
        }

        Ok(())
    }

    fn get_field_id_for(&self, query_field_node_ix: NodeIndex) -> Option<FieldId> {
        self.query_field_node_to_field
            .binary_search_by(|probe| probe.0.cmp(&query_field_node_ix))
            .map(|i| self.query_field_node_to_field[i].1)
            .ok()
    }

    fn popuplate_modifiers_after_plan_generation(&mut self) -> PlanResult<()> {
        let bound_field_to_field = IdToMany::from(std::mem::take(&mut self.bound_field_to_field));

        for modifier in std::mem::take(&mut self.operation.query_modifiers) {
            let start = self.operation_plan.field_refs.len();
            for bound_field_id in &self.operation[modifier.impacted_fields] {
                self.operation_plan
                    .field_refs
                    .extend(bound_field_to_field.find_all(*bound_field_id));
            }
            self.operation_plan.query_modifiers.push(QueryModifierRecord {
                rule: modifier.rule,
                impacted_field_ids: IdRange::from(start..self.operation_plan.field_refs.len()),
            });
        }

        for modifier in std::mem::take(&mut self.operation.response_modifiers) {
            let start = self.operation_plan.field_refs.len();
            for bound_field_id in &self.operation[modifier.impacted_fields] {
                self.operation_plan
                    .field_refs
                    .extend(bound_field_to_field.find_all(*bound_field_id));
            }
            self.operation_plan.response_modifiers.push(ResponseModifierRecord {
                rule: modifier.rule,
                impacted_field_ids: IdRange::from(start..self.operation_plan.field_refs.len()),
            });
        }

        Ok(())
    }
}

impl From<&BoundField> for FieldRecord {
    fn from(field: &BoundField) -> Self {
        match field {
            BoundField::TypeName(field) => FieldRecord::Typename(TypenameFieldRecord {
                key: field.bound_response_key.into(),
                location: field.location,
                type_condition_id: field.type_condition.into(),
            }),
            BoundField::Query(field) => FieldRecord::Data(DataFieldRecord {
                key: field.bound_response_key.into(),
                location: field.location,
                definition_id: field.definition_id,
                argument_ids: IdRange::from_start_and_end(field.argument_ids.start, field.argument_ids.end),
                // All set later
                selection_set_field_ids: IdRange::empty(),
                required_field_ids: IdRange::empty(),
            }),
            BoundField::Extra(field) => FieldRecord::Data(DataFieldRecord {
                key: PositionedResponseKey {
                    // Having no query position is equivalent to being an
                    // extra field.
                    query_position: None,
                    key: field.edge.as_response_key().unwrap(),
                },
                location: field.petitioner_location,
                definition_id: field.definition_id,
                argument_ids: IdRange::from_start_and_end(field.argument_ids.start, field.argument_ids.end),
                // All set later
                selection_set_field_ids: IdRange::empty(),
                required_field_ids: IdRange::empty(),
            }),
        }
    }
}

impl From<BoundFieldArgumentId> for FieldArgumentId {
    fn from(id: BoundFieldArgumentId) -> Self {
        FieldArgumentId::from(usize::from(id))
    }
}

impl From<BoundFieldArgument> for FieldArgumentRecord {
    fn from(arg: BoundFieldArgument) -> Self {
        FieldArgumentRecord {
            definition_id: arg.input_value_definition_id,
            value_id: arg.input_value_id,
        }
    }
}

impl From<BoundVariableDefinitionId> for VariableDefinitionId {
    fn from(id: BoundVariableDefinitionId) -> Self {
        VariableDefinitionId::from(usize::from(id))
    }
}

impl From<BoundVariableDefinition> for VariableDefinitionRecord {
    fn from(var: BoundVariableDefinition) -> Self {
        VariableDefinitionRecord {
            name: var.name,
            name_location: var.name_location,
            default_value_id: var.default_value_id,
            ty_record: var.ty_record,
        }
    }
}
