mod adapter;
mod shapes;

use id_newtypes::{IdRange, IdToMany};
use im::HashMap;
use query_planning::{Edge, EdgeIndex, EdgeRef, Node, NodeIndex, SolvedGraph};
use schema::Schema;
use walker::Walk;

use crate::{
    operation::{
        BoundField, BoundFieldArgument, BoundFieldArgumentId, BoundFieldId, BoundVariableDefinition,
        BoundVariableDefinitionId, Operation,
    },
    response::{ConcreteObjectShapeId, PositionedResponseKey, Shapes},
    utils::BufferPool,
};

use super::{
    error::PlanError, DataFieldId, DataFieldRecord, FieldArgumentId, FieldArgumentRecord, FieldId, OperationPlan,
    PlanId, PlanRecord, PlanResult, QueryModifierRecord, ResponseModifierRecord, ResponseObjectSetDefinitionId,
    SelectionSetRecord, TypenameFieldRecord, VariableDefinitionId, VariableDefinitionRecord,
};

impl OperationPlan {
    pub(super) fn build(schema: &Schema, mut operation: Operation) -> PlanResult<Self> {
        let graph =
            query_planning::OperationGraph::new(schema, adapter::OperationAdapter::new(schema, &mut operation))?
                .solve()?;
        OperationPlanBuilder {
            schema,
            operation_plan: OperationPlan {
                data_fields: Vec::with_capacity(operation.fields.len()), // approximate
                typename_fields: Vec::new(),
                field_arguments: operation.field_arguments.drain(..).map(Into::into).collect(),
                variable_definitions: operation.variable_definitions.drain(..).map(Into::into).collect(),
                response_keys: std::mem::take(&mut operation.response_keys),
                plans: Vec::new(),
                query_input_values: std::mem::take(&mut operation.query_input_values),
                query_modifiers: Vec::with_capacity(operation.query_modifiers.len()),
                response_modifiers: Vec::with_capacity(operation.response_modifiers.len()),
                response_object_set_definitions: Vec::new(),
                shapes: Shapes::default(),
                field_refs: Vec::new(),
                data_field_refs: Vec::new(),
                field_shape_refs: Vec::new(),
            },
            operation,
            graph,
            nested_fields_buffer_pool: BufferPool::default(),
            edges_buffer: BufferPool::default(),
            plan_stack: Vec::new(),
            query_field_node_to_field: Vec::new(),
            field_to_providable_field_node: Vec::new(),
            plan_to_resolver_node: Vec::new(),
            bound_field_to_field: Vec::new(),
            query_field_node_to_response_object_set: HashMap::new(),
        }
        .build()
    }
}

struct OperationPlanBuilder<'a> {
    schema: &'a Schema,
    operation_plan: OperationPlan,
    graph: SolvedGraph<BoundFieldId>,
    operation: Operation,
    nested_fields_buffer_pool: BufferPool<NestedField>,
    edges_buffer: BufferPool<EdgeIndex>,
    plan_stack: Vec<PlanToCreate>,
    query_field_node_to_response_object_set: HashMap<NodeIndex, ResponseObjectSetDefinitionId>,
    // one to one, sorted after the plan generation.
    query_field_node_to_field: Vec<(NodeIndex, DataFieldId)>,
    bound_field_to_field: Vec<(BoundFieldId, FieldId)>,
    // Populated during plan generation, drained while populating requirements.
    field_to_providable_field_node: Vec<(DataFieldId, NodeIndex)>,
    // Populated during plan generation, drained while populating requirements.
    plan_to_resolver_node: Vec<(PlanId, NodeIndex)>,
}

struct PlanToCreate {
    input_id: ResponseObjectSetDefinitionId,
    resolver_node_ix: NodeIndex,
    resolver: query_planning::Resolver,
}

enum NestedField {
    Data {
        record: DataFieldRecord,
        bound_field_id: BoundFieldId,
        providable_field_node_ix: NodeIndex,
        query_field_node_ix: NodeIndex,
    },
    Typename {
        record: TypenameFieldRecord,
        bound_field_id: BoundFieldId,
    },
}

impl OperationPlanBuilder<'_> {
    fn build(mut self) -> PlanResult<OperationPlan> {
        self.operation_plan
            .response_object_set_definitions
            .push(super::ResponseObjectSetDefinitionRecord {
                ty_id: self.operation.root_object_id.into(),
            });

        for edge in self.graph.edges(self.graph.root_node_ix) {
            if matches!(edge.weight(), Edge::CreateChildResolver) {
                if let Node::Resolver(resolver) = &self.graph[edge.target()] {
                    self.plan_stack.push(PlanToCreate {
                        input_id: ResponseObjectSetDefinitionId::from(0usize),
                        resolver_node_ix: edge.target(),
                        resolver: resolver.clone(),
                    });
                }
            }
        }

        // Generate all plans and their provided fields.
        while let Some(plan_to_create) = self.plan_stack.pop() {
            self.generate_plan(plan_to_create);
        }

        self.populate_requirements_after_plan_generation()?;

        self.populate_modifiers_after_plan_generation()?;

        self.populate_shapes_after_plan_generation();

        Ok(self.operation_plan)
    }

    fn generate_plan(
        &mut self,
        PlanToCreate {
            input_id,
            resolver_node_ix,
            resolver,
        }: PlanToCreate,
    ) {
        let fields_start = self.operation_plan.data_fields.len();
        let (_, selection_set_record) = self.generate_selection_set(resolver_node_ix);
        let output_ids = self.operation_plan.data_fields[fields_start..]
            .iter()
            .filter_map(|field| field.output_id)
            .collect();
        self.operation_plan.plans.push(PlanRecord {
            entity_definition_id: resolver.entity_definition_id,
            resolver_definition_id: resolver.definition_id,
            selection_set_record,
            input_id,
            output_ids,
            // Populated later
            required_field_ids: IdRange::empty(),
            shape_id: ConcreteObjectShapeId::from(0usize),
        });
        let plan_id = PlanId::from(self.operation_plan.plans.len() - 1);
        self.plan_to_resolver_node.push((plan_id, resolver_node_ix));
    }

    fn generate_selection_set(
        &mut self,
        node_ix: NodeIndex,
    ) -> (Option<ResponseObjectSetDefinitionId>, SelectionSetRecord) {
        let mut fields_buffer = self.nested_fields_buffer_pool.pop();
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
                        let matching_requirement_id = field.matching_requirement_id;
                        match self.operation[bound_field_id].to_field() {
                            Ok(mut record) => {
                                let (ouput_id, selection_set) = self.generate_selection_set(neighbor);
                                record.selection_set_record = selection_set;
                                record.output_id = ouput_id;
                                record.matching_requirement_id = matching_requirement_id;
                                fields_buffer.push(NestedField::Data {
                                    record,
                                    bound_field_id,
                                    providable_field_node_ix: neighbor,
                                    query_field_node_ix,
                                });
                            }
                            Err(record) => {
                                fields_buffer.push(NestedField::Typename { record, bound_field_id });
                            }
                        }
                    }
                }
                // Which field is provided by the current node if any. Resolvers won't provide any
                // but they still have nested fields.
                Edge::Provides => {
                    if let Node::QueryField(field) = &self.graph[neighbor] {
                        provided_query_field_ix = Some((neighbor, field.id));
                    }
                }
                _ => (),
            }
        }

        // We take any non-attributed __typename field and resolver.
        let mut output_id = None;
        if let Some((provided_query_field_ix, bound_field_id)) = provided_query_field_ix {
            let mut edges_to_remove = self.edges_buffer.pop();
            for edge in self.graph.edges(provided_query_field_ix) {
                match edge.weight() {
                    Edge::HasChildResolver => {
                        if let Node::Resolver(resolver) = &self.graph[edge.target()] {
                            self.plan_stack.push(PlanToCreate {
                                input_id: *output_id.get_or_insert_with(|| {
                                    *self
                                        .query_field_node_to_response_object_set
                                        .entry(provided_query_field_ix)
                                        .or_insert_with(|| {
                                            self.operation_plan
                                                .response_object_set_definitions
                                                .push(super::ResponseObjectSetDefinitionRecord {
                                                ty_id: self.operation[bound_field_id]
                                                    .definition_id()
                                                    .and_then(|def| {
                                                        def.walk(self.schema).ty().definition_id.as_composite_type()
                                                    })
                                                    .expect(
                                                        "Could not have a child resolver if it wasn't a composite type",
                                                    ),
                                            });
                                            ResponseObjectSetDefinitionId::from(
                                                self.operation_plan.response_object_set_definitions.len() - 1,
                                            )
                                        })
                                }),
                                resolver_node_ix: edge.target(),
                                resolver: resolver.clone(),
                            });
                        }
                        edges_to_remove.push(edge.id());
                    }
                    Edge::TypenameField => {
                        if let Node::QueryField(field) = &self.graph[edge.target()] {
                            match self.operation[field.id].to_field() {
                                Ok(_) => unreachable!("Cannot be a DataField"),
                                Err(record) => {
                                    fields_buffer.push(NestedField::Typename {
                                        record,
                                        bound_field_id: field.id,
                                    });
                                }
                            }
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

        let data_fields_start = self.operation_plan.data_fields.len();
        let typename_fields_start = self.operation_plan.typename_fields.len();
        for field in fields_buffer.drain(..) {
            match field {
                NestedField::Data {
                    record,
                    bound_field_id,
                    providable_field_node_ix,
                    query_field_node_ix,
                } => {
                    self.operation_plan.data_fields.push(record);
                    let data_field_id = DataFieldId::from(self.operation_plan.data_fields.len() - 1);
                    self.bound_field_to_field.push((bound_field_id, data_field_id.into()));
                    self.field_to_providable_field_node
                        .push((data_field_id, providable_field_node_ix));
                    self.query_field_node_to_field
                        .push((query_field_node_ix, data_field_id))
                }
                NestedField::Typename { record, bound_field_id } => {
                    self.operation_plan.typename_fields.push(record);
                    let typename_field_id = DataFieldId::from(self.operation_plan.typename_fields.len() - 1);
                    self.bound_field_to_field
                        .push((bound_field_id, typename_field_id.into()));
                }
            }
        }
        self.nested_fields_buffer_pool.push(fields_buffer);

        let selection_set = SelectionSetRecord {
            data_field_ids: IdRange::from(data_fields_start..self.operation_plan.data_fields.len()),
            typename_field_ids: IdRange::from(typename_fields_start..self.operation_plan.typename_fields.len()),
        };

        (output_id, selection_set)
    }

    fn populate_requirements_after_plan_generation(&mut self) -> PlanResult<()> {
        self.query_field_node_to_field.sort_unstable();

        for (plan_id, resolver_node_ix) in std::mem::take(&mut self.plan_to_resolver_node) {
            let start = self.operation_plan.data_field_refs.len();
            for edge in self.graph.edges(resolver_node_ix) {
                if !matches!(edge.weight(), Edge::Requires) {
                    continue;
                }
                self.operation_plan
                    .data_field_refs
                    .push(self.get_field_id_for(edge.target()).ok_or_else(|| {
                        tracing::error!("Plan depends on an unknown query field node.");
                        PlanError::InternalError
                    })?);
            }
            self.operation_plan[plan_id].required_field_ids =
                IdRange::from(start..self.operation_plan.data_field_refs.len());
        }

        for (field_id, providable_field_node_ix) in std::mem::take(&mut self.field_to_providable_field_node) {
            let start = self.operation_plan.data_field_refs.len();

            for edge in self.graph.edges(providable_field_node_ix) {
                match edge.weight() {
                    Edge::Provides => {
                        debug_assert!(matches!(self.graph[edge.target()], Node::QueryField(_)));
                        for edge in self.graph.edges(edge.target()) {
                            if matches!(edge.weight(), Edge::Requires) {
                                self.operation_plan.data_field_refs.push(
                                    self.get_field_id_for(edge.target()).ok_or_else(|| {
                                        tracing::error!("Field depends on an unknown query field node.");
                                        PlanError::InternalError
                                    })?,
                                );
                            }
                        }
                    }
                    Edge::Requires => {
                        self.operation_plan
                            .data_field_refs
                            .push(self.get_field_id_for(edge.target()).ok_or_else(|| {
                                tracing::error!("Field depends on an unknown query field node.");
                                PlanError::InternalError
                            })?);
                    }
                    _ => (),
                }
            }

            let required_field_ids = IdRange::from(start..self.operation_plan.data_field_refs.len());
            self.operation_plan[field_id].required_field_ids = required_field_ids;
        }

        Ok(())
    }

    fn get_field_id_for(&self, query_field_node_ix: NodeIndex) -> Option<DataFieldId> {
        self.query_field_node_to_field
            .binary_search_by(|probe| probe.0.cmp(&query_field_node_ix))
            .map(|i| self.query_field_node_to_field[i].1)
            .ok()
    }

    fn populate_modifiers_after_plan_generation(&mut self) -> PlanResult<()> {
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

impl BoundField {
    fn to_field(&self) -> Result<DataFieldRecord, TypenameFieldRecord> {
        match self {
            BoundField::Query(field) => Ok(DataFieldRecord {
                key: field.bound_response_key.into(),
                location: field.location,
                definition_id: field.definition_id,
                argument_ids: IdRange::from_start_and_end(field.argument_ids.start, field.argument_ids.end),
                // All set later
                selection_set_record: SelectionSetRecord {
                    data_field_ids: IdRange::empty(),
                    typename_field_ids: IdRange::empty(),
                },
                required_field_ids: IdRange::empty(),
                output_id: None,
                matching_requirement_id: None,
                selection_set_requires_typename: false,
                shape_ids: IdRange::empty(),
            }),
            BoundField::Extra(field) => Ok(DataFieldRecord {
                key: PositionedResponseKey {
                    // Having no query position is equivalent to being an
                    // extra field.
                    query_position: None,
                    response_key: field.edge.as_response_key().unwrap(),
                },
                location: field.petitioner_location,
                definition_id: field.definition_id,
                argument_ids: IdRange::from_start_and_end(field.argument_ids.start, field.argument_ids.end),
                // All set later
                selection_set_record: SelectionSetRecord {
                    data_field_ids: IdRange::empty(),
                    typename_field_ids: IdRange::empty(),
                },
                required_field_ids: IdRange::empty(),
                output_id: None,
                matching_requirement_id: None,
                selection_set_requires_typename: false,
                shape_ids: IdRange::empty(),
            }),
            BoundField::TypeName(field) => Err(TypenameFieldRecord {
                key: field.bound_response_key.into(),
                location: field.location,
                type_condition_id: field.type_condition.into(),
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
