mod adapter;
mod requires;
mod shapes;

use id_newtypes::{IdRange, IdToMany};
use im::HashMap;
use query_solver::{
    petgraph::{graph::NodeIndex, visit::EdgeRef},
    SolutionEdge as Edge, SolutionGraph, SolutionNode as Node,
};
use schema::{Definition, EntityDefinitionId, ResolverDefinitionId, Schema};
use walker::Walk;

use crate::{
    operation::{
        BoundField, BoundFieldArgument, BoundFieldArgumentId, BoundFieldId, BoundOperation, BoundQueryModifierId,
        BoundVariableDefinition, BoundVariableDefinitionId, SolveError,
    },
    response::{ConcreteShapeId, PositionedResponseKey, Shapes},
    utils::BufferPool,
};

use super::{
    DataFieldId, DataFieldRecord, FieldArgumentId, FieldArgumentRecord, FieldId, QueryModifierDefinitionRecord,
    QueryPartitionId, QueryPartitionRecord, RequiredFieldSetRecord, ResponseModifierRuleToImpactedFields,
    ResponseObjectSetDefinitionId, SelectionSetRecord, SolveResult, SolvedOperation, TypenameFieldRecord,
    VariableDefinitionId, VariableDefinitionRecord,
};

pub(super) struct Solver<'a> {
    schema: &'a Schema,
    operation: SolvedOperation,
    root_node_ix: NodeIndex,
    graph: SolutionGraph<BoundFieldId>,
    bound_operation: BoundOperation,
    nested_fields_buffer_pool: BufferPool<NestedField>,
    query_partitions_to_create_stack: Vec<QueryPartitionToCreate>,
    query_field_node_to_response_object_set: HashMap<NodeIndex, ResponseObjectSetDefinitionId>,
    // one to one, sorted after the plan generation.
    node_to_field: Vec<(NodeIndex, DataFieldId)>,
    bound_field_to_field: Vec<(BoundFieldId, FieldId)>,
    // Populated during plan generation, drained while populating requirements.
    field_to_node: Vec<(DataFieldId, NodeIndex)>,
    // Populated during plan generation
    query_partition_to_node: Vec<(QueryPartitionId, NodeIndex)>,
}

struct QueryPartitionToCreate {
    input_id: ResponseObjectSetDefinitionId,
    source_ix: NodeIndex,
    entity_definition_id: EntityDefinitionId,
    resolver_definition_id: ResolverDefinitionId,
}

enum NestedField {
    Data {
        record: DataFieldRecord,
        bound_field_id: BoundFieldId,
        node_ix: NodeIndex,
    },
    Typename {
        record: TypenameFieldRecord,
        bound_field_id: BoundFieldId,
    },
}

impl<'a> Solver<'a> {
    pub(super) fn build(schema: &'a Schema, mut bound_operation: BoundOperation) -> SolveResult<Self> {
        let solution = query_solver::solve(schema, adapter::OperationAdapter::new(schema, &mut bound_operation))?;
        let root_node_ix = solution.root_node_ix;
        let graph = solution.graph;
        Ok(Self {
            schema,
            operation: SolvedOperation {
                root_object_id: bound_operation.root_object_id,
                data_fields: Vec::with_capacity(bound_operation.fields.len()), // approximate
                typename_fields: Vec::new(),
                field_arguments: bound_operation.field_arguments.drain(..).map(Into::into).collect(),
                variable_definitions: bound_operation.variable_definitions.drain(..).map(Into::into).collect(),
                response_keys: std::mem::take(&mut bound_operation.response_keys),
                query_partitions: Vec::new(),
                mutation_partition_order: Vec::new(),
                query_input_values: std::mem::take(&mut bound_operation.query_input_values),
                query_modifier_definitions: Vec::with_capacity(bound_operation.query_modifiers.len()),
                response_modifier_rule_to_impacted_fields: Vec::with_capacity(bound_operation.response_modifiers.len()),
                response_object_set_definitions: Vec::new(),
                shapes: Shapes::default(),
                field_refs: Vec::new(),
                data_field_refs: Vec::new(),
                field_shape_refs: Vec::new(),
            },
            node_to_field: Vec::with_capacity(graph.node_count()),
            field_to_node: Vec::with_capacity(graph.node_count()),
            bound_field_to_field: Vec::with_capacity(bound_operation.fields.len()),
            bound_operation,
            root_node_ix,
            graph,
            nested_fields_buffer_pool: BufferPool::default(),
            query_partitions_to_create_stack: Vec::new(),
            query_partition_to_node: Vec::new(),
            query_field_node_to_response_object_set: HashMap::new(),
        })
    }

    pub(super) fn solve(mut self) -> SolveResult<SolvedOperation> {
        self.operation
            .response_object_set_definitions
            .push(super::ResponseObjectSetDefinitionRecord {
                ty_id: self.bound_operation.root_object_id.into(),
            });
        let root_input_id = ResponseObjectSetDefinitionId::from(0usize);
        self.query_field_node_to_response_object_set
            .insert(self.root_node_ix, root_input_id);

        for edge in self.graph.edges(self.root_node_ix) {
            if let Edge::QueryPartition = edge.weight() {
                if let Node::QueryPartition {
                    entity_definition_id,
                    resolver_definition_id,
                } = self.graph[edge.target()]
                {
                    self.query_partitions_to_create_stack.push(QueryPartitionToCreate {
                        input_id: root_input_id,
                        source_ix: edge.target(),
                        entity_definition_id,
                        resolver_definition_id,
                    });
                }
            }
        }

        while let Some(partition_to_create) = self.query_partitions_to_create_stack.pop() {
            self.generate_query_partition(partition_to_create);
        }

        self.generate_mutation_partition_order_after_partition_generation()?;

        self.populate_requirements_after_partition_generation()?;

        self.populate_modifiers_after_partition_generation()?;

        self.populate_shapes_after_partition_generation();

        Ok(self.operation)
    }

    fn generate_query_partition(
        &mut self,
        QueryPartitionToCreate {
            input_id,
            source_ix,
            entity_definition_id,
            resolver_definition_id,
        }: QueryPartitionToCreate,
    ) {
        let query_partition_id = QueryPartitionId::from(self.operation.query_partitions.len());
        let (_, selection_set_record) = self.generate_selection_set(query_partition_id, None, source_ix);
        self.operation.query_partitions.push(QueryPartitionRecord {
            entity_definition_id,
            resolver_definition_id,
            selection_set_record,
            input_id,
            // Populated later
            required_fields_record: Default::default(),
            shape_id: ConcreteShapeId::from(0usize),
        });
        self.query_partition_to_node.push((query_partition_id, source_ix));
    }

    fn generate_selection_set(
        &mut self,
        query_partition_id: QueryPartitionId,
        mut output_id: Option<ResponseObjectSetDefinitionId>,
        source_ix: NodeIndex,
    ) -> (Option<ResponseObjectSetDefinitionId>, SelectionSetRecord) {
        let mut fields_buffer = self.nested_fields_buffer_pool.pop();

        let mut neighbors = self.graph.neighbors(source_ix).detach();
        while let Some((edge_ix, target_ix)) = neighbors.next(&self.graph) {
            match self.graph[edge_ix] {
                Edge::QueryPartition => {
                    let Node::QueryPartition {
                        entity_definition_id,
                        resolver_definition_id,
                    } = self.graph[target_ix]
                    else {
                        continue;
                    };
                    let new_partition = QueryPartitionToCreate {
                        input_id: *output_id
                            .get_or_insert_with(|| self.create_new_response_object_set_definition(source_ix)),
                        source_ix: target_ix,
                        resolver_definition_id,
                        entity_definition_id,
                    };
                    self.query_partitions_to_create_stack.push(new_partition);
                }
                Edge::Field => {
                    let Node::Field { id, .. } = self.graph[target_ix] else {
                        continue;
                    };
                    match self.bound_operation[id].to_data_field_or_typename_field(self.schema, query_partition_id) {
                        Ok(mut record) => {
                            // If there is any edge with super-graph requirements, means there we'll
                            // need to read this field.
                            let output_id = if self
                                .graph
                                .edges(target_ix)
                                .any(|edge| matches!(edge.weight(), Edge::RequiredBySupergraph))
                            {
                                let definition = record.definition_id.walk(self.schema);

                                if definition
                                    .directives()
                                    .filter_map(|directive| directive.as_authorized())
                                    .any(|auth| auth.fields().is_some())
                                {
                                    output_id.get_or_insert_with(|| {
                                        self.create_new_response_object_set_definition(source_ix)
                                    });
                                }
                                if definition
                                    .directives()
                                    .filter_map(|directive| directive.as_authorized())
                                    .any(|auth| auth.node().is_some())
                                {
                                    Some(self.create_new_response_object_set_definition(target_ix))
                                } else {
                                    None
                                }
                            } else {
                                None
                            };
                            let (output_id, selection_set) =
                                self.generate_selection_set(query_partition_id, output_id, target_ix);
                            record.selection_set_record = selection_set;
                            record.output_id = output_id;
                            fields_buffer.push(NestedField::Data {
                                record,
                                bound_field_id: id,
                                node_ix: target_ix,
                            });
                        }
                        Err(record) => {
                            fields_buffer.push(NestedField::Typename {
                                record,
                                bound_field_id: id,
                            });
                        }
                    }
                }
                Edge::RequiredBySubgraph | Edge::RequiredBySupergraph | Edge::MutationExecutedAfter => (),
            }
        }

        let data_fields_start = self.operation.data_fields.len();
        let typename_fields_start = self.operation.typename_fields.len();

        fields_buffer.sort_unstable_by_key(|field| match field {
            NestedField::Data { record, .. } => (
                record.definition_id.walk(self.schema).parent_entity_id.into(),
                record.key,
            ),
            NestedField::Typename { record, .. } => (record.type_condition_id, record.key),
        });

        for field in fields_buffer.drain(..) {
            match field {
                NestedField::Data {
                    mut record,
                    bound_field_id,
                    node_ix,
                } => {
                    record.parent_field_output_id = output_id;
                    let data_field_id = DataFieldId::from(self.operation.data_fields.len());
                    self.node_to_field.push((node_ix, data_field_id));
                    self.operation.data_fields.push(record);
                    self.bound_field_to_field.push((bound_field_id, data_field_id.into()));
                    self.field_to_node.push((data_field_id, node_ix));
                }
                NestedField::Typename { record, bound_field_id } => {
                    self.operation.typename_fields.push(record);
                    let typename_field_id = DataFieldId::from(self.operation.typename_fields.len() - 1);
                    self.bound_field_to_field
                        .push((bound_field_id, typename_field_id.into()));
                }
            }
        }
        self.nested_fields_buffer_pool.push(fields_buffer);

        let selection_set = SelectionSetRecord {
            data_field_ids_ordered_by_parent_entity_id_then_key: IdRange::from(
                data_fields_start..self.operation.data_fields.len(),
            ),
            typename_field_ids_ordered_by_type_condition_id_then_key: IdRange::from(
                typename_fields_start..self.operation.typename_fields.len(),
            ),
        };

        (output_id, selection_set)
    }

    fn create_new_response_object_set_definition(&mut self, source_ix: NodeIndex) -> ResponseObjectSetDefinitionId {
        let Node::Field { id, .. } = self.graph[source_ix] else {
            unreachable!();
        };
        *self
            .query_field_node_to_response_object_set
            .entry(source_ix)
            .or_insert_with(|| {
                self.operation
                    .response_object_set_definitions
                    .push(super::ResponseObjectSetDefinitionRecord {
                        ty_id: self.bound_operation[id]
                            .definition_id()
                            .and_then(|def| def.walk(self.schema).ty().definition_id.as_composite_type())
                            .expect("Could not have a child resolver if it wasn't a composite type"),
                    });
                ResponseObjectSetDefinitionId::from(self.operation.response_object_set_definitions.len() - 1)
            })
    }

    fn populate_modifiers_after_partition_generation(&mut self) -> SolveResult<()> {
        let bound_field_to_field = IdToMany::from(std::mem::take(&mut self.bound_field_to_field));

        for (i, modifier) in std::mem::take(&mut self.bound_operation.query_modifiers)
            .into_iter()
            .enumerate()
        {
            let id = BoundQueryModifierId::from(i);
            let start = self.operation.field_refs.len();
            for bound_field_id in &self.bound_operation[modifier.impacted_fields] {
                self.operation
                    .field_refs
                    .extend(bound_field_to_field.find_all(*bound_field_id));
            }
            self.operation
                .query_modifier_definitions
                .push(QueryModifierDefinitionRecord {
                    rule: modifier.rule,
                    impacts_root_object: self.bound_operation.root_query_modifier_ids.contains(&id),
                    impacted_field_ids: IdRange::from(start..self.operation.field_refs.len()),
                });
        }

        for modifier in std::mem::take(&mut self.bound_operation.response_modifiers) {
            let start = self.operation.data_field_refs.len();
            for bound_field_id in &self.bound_operation[modifier.impacted_fields] {
                self.operation
                    .data_field_refs
                    .extend(bound_field_to_field.find_all(*bound_field_id).map(|field| match field {
                        FieldId::Data(id) => *id,
                        FieldId::Typename(_) => unreachable!(),
                    }));
            }
            self.operation
                .response_modifier_rule_to_impacted_fields
                .push(ResponseModifierRuleToImpactedFields {
                    rule: modifier.rule,
                    impacted_field_ids: IdRange::from(start..self.operation.data_field_refs.len()),
                });
        }

        Ok(())
    }

    fn generate_mutation_partition_order_after_partition_generation(&mut self) -> SolveResult<()> {
        if !self.bound_operation.ty.is_mutation() {
            return Ok(());
        }
        let mut partition_to_next_in_order = Vec::new();
        let mut initial_partition = None;
        for neighbor in self.graph.neighbors(self.root_node_ix) {
            if let Node::QueryPartition { .. } = self.graph[neighbor] {
                if let Some(prev) = self
                    .graph
                    .edges(neighbor)
                    .find(|edge| matches!(edge.weight(), Edge::MutationExecutedAfter))
                {
                    partition_to_next_in_order.push((prev.target(), neighbor));
                } else {
                    initial_partition = Some(neighbor);
                }
            }
        }

        let Some(initial_partition) = initial_partition else {
            tracing::error!("Mutation without initial query partition.");
            return Err(SolveError::InternalError);
        };

        self.query_partition_to_node.sort_unstable_by(|a, b| a.1.cmp(&b.1));

        fn get_query_partition_id(builder: &Solver<'_>, node_ix: NodeIndex) -> SolveResult<QueryPartitionId> {
            builder
                .query_partition_to_node
                .binary_search_by(|probe| probe.1.cmp(&node_ix))
                .map(|i| builder.query_partition_to_node[i].0)
                .map_err(|_| {
                    tracing::error!("Could not find query partition id for node.");
                    SolveError::InternalError
                })
        }

        let mut mutation_partition_order = Vec::with_capacity(partition_to_next_in_order.len());
        mutation_partition_order.push(get_query_partition_id(self, initial_partition)?);
        partition_to_next_in_order.sort_unstable();

        let mut last = initial_partition;
        while let Ok(i) = partition_to_next_in_order.binary_search_by(|probe| probe.0.cmp(&last)) {
            let (_, next) = partition_to_next_in_order[i];
            mutation_partition_order.push(get_query_partition_id(self, next)?);
            last = next;
        }

        self.operation.mutation_partition_order = mutation_partition_order;

        Ok(())
    }
}

impl BoundField {
    fn to_data_field_or_typename_field(
        &self,
        schema: &Schema,
        query_partition_id: QueryPartitionId,
    ) -> Result<DataFieldRecord, TypenameFieldRecord> {
        match self {
            BoundField::Query(field) => Ok(DataFieldRecord {
                key: field.key.with_position(field.query_position),
                subgraph_key: field.subgraph_key,
                location: field.location,
                definition_id: field.definition_id,
                argument_ids: IdRange::from_start_and_end(field.argument_ids.start, field.argument_ids.end),
                query_partition_id,
                // All set later
                selection_set_record: SelectionSetRecord {
                    data_field_ids_ordered_by_parent_entity_id_then_key: IdRange::empty(),
                    typename_field_ids_ordered_by_type_condition_id_then_key: IdRange::empty(),
                },
                required_fields_record: RequiredFieldSetRecord::default(),
                required_fields_record_by_supergraph: Default::default(),
                output_id: None,
                parent_field_output_id: None,
                selection_set_requires_typename: match field.definition_id.walk(schema).ty().definition() {
                    // If we may encounter an inaccessible object, we have to detect it
                    Definition::Union(union) => union.has_inaccessible_member(),
                    Definition::Interface(interface) => interface.has_inaccessible_implementors(),
                    _ => false,
                },
                shape_ids: IdRange::empty(),
            }),
            BoundField::Extra(field) => Ok(DataFieldRecord {
                key: PositionedResponseKey {
                    // Having no query position is equivalent to being an
                    // extra field.
                    query_position: None,
                    response_key: field.key.unwrap(),
                },
                subgraph_key: field.key.unwrap(),
                location: field.petitioner_location,
                definition_id: field.definition_id,
                argument_ids: IdRange::from_start_and_end(field.argument_ids.start, field.argument_ids.end),
                query_partition_id,
                // All set later
                selection_set_record: SelectionSetRecord {
                    data_field_ids_ordered_by_parent_entity_id_then_key: IdRange::empty(),
                    typename_field_ids_ordered_by_type_condition_id_then_key: IdRange::empty(),
                },
                required_fields_record: RequiredFieldSetRecord::default(),
                required_fields_record_by_supergraph: Default::default(),
                output_id: None,
                parent_field_output_id: None,
                selection_set_requires_typename: false,
                shape_ids: IdRange::empty(),
            }),
            BoundField::TypeName(field) => Err(TypenameFieldRecord {
                key: field.key.with_position(field.query_position),
                location: field.location,
                type_condition_id: field.type_condition,
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
