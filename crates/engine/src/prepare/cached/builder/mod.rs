mod modifiers;
mod requires;
mod shapes;

use id_newtypes::{BitSet, IdRange};
use im::HashMap;
use operation::Operation;
use query_solver::{
    Edge, Node, QueryField, SolvedQuery,
    petgraph::{graph::NodeIndex, visit::EdgeRef},
};
use schema::{Definition, EntityDefinitionId, ResolverDefinitionId, Schema};
use walker::Walk;

use super::*;
use crate::utils::BufferPool;

pub(super) struct Solver<'a> {
    schema: &'a Schema,
    output: CachedOperation,
    solution: SolvedQuery,
    nested_fields_buffer_pool: BufferPool<NestedField>,
    query_partitions_to_create_stack: Vec<QueryPartitionToCreate>,
    query_field_node_to_response_object_set: HashMap<NodeIndex, ResponseObjectSetDefinitionId>,
    // one to one
    node_to_field: Vec<Option<PartitionFieldId>>,
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
        record: PartitionDataFieldRecord,
        node_ix: NodeIndex,
    },
    Typename {
        record: PartitionTypenameFieldRecord,
        node_ix: NodeIndex,
    },
}

impl<'a> Solver<'a> {
    pub(super) fn build(
        schema: &'a Schema,
        document: OperationDocument<'_>,
        mut operation: Operation,
    ) -> SolveResult<Self> {
        let mut solution = query_solver::solve(schema, &mut operation)?;
        Ok(Self {
            schema,
            output: CachedOperation {
                document: document.into_owned(),
                query_plan: QueryPlan {
                    partitions: Vec::new(),
                    mutation_partition_order: Vec::new(),
                    shared_type_conditions: std::mem::take(&mut solution.shared_type_conditions),
                    field_shape_refs: Vec::new(),
                    data_fields: Vec::with_capacity(solution.fields.len()),
                    typename_fields: Vec::new(),
                    response_object_set_definitions: Vec::new(),
                    response_data_fields: Default::default(),
                    response_typename_fields: Default::default(),
                    query_modifiers: Default::default(),
                    response_modifier_definitions: Vec::new(),
                },
                operation,
                shapes: Shapes::default(),
            },
            node_to_field: vec![None; solution.graph.node_count()],
            solution,
            nested_fields_buffer_pool: BufferPool::default(),
            query_partitions_to_create_stack: Vec::new(),
            query_partition_to_node: Vec::new(),
            query_field_node_to_response_object_set: HashMap::new(),
        })
    }

    pub(super) fn solve(mut self) -> SolveResult<CachedOperation> {
        self.output
            .query_plan
            .response_object_set_definitions
            .push(super::ResponseObjectSetDefinitionRecord {
                ty_id: self.output.operation.root_object_id.into(),
            });
        let root_input_id = ResponseObjectSetDefinitionId::from(0usize);
        self.query_field_node_to_response_object_set
            .insert(self.solution.root_node_ix, root_input_id);

        for edge in self.solution.graph.edges(self.solution.root_node_ix) {
            if let Edge::QueryPartition = edge.weight() {
                if let Node::QueryPartition {
                    entity_definition_id,
                    resolver_definition_id,
                } = self.solution.graph[edge.target()]
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

        let mut response_data_fields = BitSet::with_capacity(self.output.query_plan.data_fields.len());
        for (i, field) in self.output.query_plan.data_fields.iter().enumerate() {
            if field.query_position.is_some() {
                response_data_fields.set(i.into(), true);
            }
        }
        self.output.query_plan.response_data_fields = response_data_fields;
        let mut response_typename_fields = BitSet::with_capacity(self.output.query_plan.typename_fields.len());
        for (i, field) in self.output.query_plan.typename_fields.iter().enumerate() {
            if field.query_position.is_some() {
                response_typename_fields.set(i.into(), true);
            }
        }
        self.output.query_plan.response_typename_fields = response_typename_fields;

        self.generate_mutation_partition_order_after_partition_generation()?;

        self.populate_requirements_after_partition_generation()?;

        self.populate_modifiers_after_partition_generation()?;

        self.populate_shapes_after_partition_generation();

        Ok(self.output)
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
        let query_partition_id = QueryPartitionId::from(self.output.query_plan.partitions.len());
        let (_, selection_set_record) = self.generate_selection_set(query_partition_id, source_ix);
        self.output.query_plan.partitions.push(QueryPartitionRecord {
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
        source_ix: NodeIndex,
    ) -> (Option<ResponseObjectSetDefinitionId>, PartitionSelectionSetRecord) {
        let mut response_object_set_id: Option<ResponseObjectSetDefinitionId> = None;
        let mut fields_buffer = self.nested_fields_buffer_pool.pop();

        let mut neighbors = self.solution.graph.neighbors(source_ix).detach();
        while let Some((edge_ix, target_ix)) = neighbors.next(&self.solution.graph) {
            match self.solution.graph[edge_ix] {
                Edge::QueryPartition => {
                    let Node::QueryPartition {
                        entity_definition_id,
                        resolver_definition_id,
                    } = self.solution.graph[target_ix]
                    else {
                        continue;
                    };
                    let new_partition = QueryPartitionToCreate {
                        input_id: *response_object_set_id
                            .get_or_insert_with(|| self.create_new_response_object_set_definition(source_ix)),
                        source_ix: target_ix,
                        resolver_definition_id,
                        entity_definition_id,
                    };
                    self.query_partitions_to_create_stack.push(new_partition);
                }
                Edge::Field => {
                    let Node::Field { id, .. } = self.solution.graph[target_ix] else {
                        continue;
                    };
                    match to_data_field_or_typename_field(&self.solution[id], self.schema, query_partition_id) {
                        MaybePartitionFieldRecord::None => continue,
                        MaybePartitionFieldRecord::Data(mut record) => {
                            if record
                                .definition_id
                                .walk(self.schema)
                                .ty()
                                .definition()
                                .is_composite_type()
                            {
                                let (nested_response_object_set_id, selection_set) =
                                    self.generate_selection_set(query_partition_id, target_ix);
                                record.output_id = nested_response_object_set_id;
                                record.selection_set_record = selection_set;
                            }
                            fields_buffer.push(NestedField::Data {
                                record,
                                node_ix: target_ix,
                            });
                        }
                        MaybePartitionFieldRecord::Typename(record) => {
                            fields_buffer.push(NestedField::Typename {
                                record,
                                node_ix: target_ix,
                            });
                        }
                    }
                }
                Edge::RequiredBySubgraph | Edge::RequiredBySupergraph | Edge::MutationExecutedAfter => (),
            }
        }

        let data_fields_start = self.output.query_plan.data_fields.len();
        let typename_fields_start = self.output.query_plan.typename_fields.len();

        fields_buffer.sort_unstable_by_key(|field| match field {
            NestedField::Data { record, .. } => (
                &self.output.query_plan[record.type_condition_ids],
                record.query_position,
            ),
            NestedField::Typename { record, .. } => (
                &self.output.query_plan[record.type_condition_ids],
                record.query_position,
            ),
        });

        for field in fields_buffer.drain(..) {
            match field {
                NestedField::Data { mut record, node_ix } => {
                    record.parent_field_output_id = response_object_set_id;
                    self.node_to_field[node_ix.index()] =
                        Some(PartitionFieldId::Data(self.output.query_plan.data_fields.len().into()));
                    self.output.query_plan.data_fields.push(record);
                }
                NestedField::Typename { record, node_ix } => {
                    self.node_to_field[node_ix.index()] = Some(PartitionFieldId::Typename(
                        self.output.query_plan.typename_fields.len().into(),
                    ));
                    self.output.query_plan.typename_fields.push(record);
                }
            }
        }
        self.nested_fields_buffer_pool.push(fields_buffer);

        let selection_set = PartitionSelectionSetRecord {
            data_field_ids_ordered_by_type_conditions_then_position: IdRange::from(
                data_fields_start..self.output.query_plan.data_fields.len(),
            ),
            typename_field_ids_ordered_by_type_conditions_then_position: IdRange::from(
                typename_fields_start..self.output.query_plan.typename_fields.len(),
            ),
        };

        (response_object_set_id, selection_set)
    }

    fn create_new_response_object_set_definition(&mut self, source_ix: NodeIndex) -> ResponseObjectSetDefinitionId {
        let Node::Field { id, .. } = self.solution.graph[source_ix] else {
            unreachable!();
        };
        *self
            .query_field_node_to_response_object_set
            .entry(source_ix)
            .or_insert_with(|| {
                self.output
                    .query_plan
                    .response_object_set_definitions
                    .push(super::ResponseObjectSetDefinitionRecord {
                        ty_id: self.solution[id]
                            .definition_id
                            .and_then(|def| def.walk(self.schema).ty().definition_id.as_composite_type())
                            .expect("Could not have a child resolver if it wasn't a composite type"),
                    });
                ResponseObjectSetDefinitionId::from(self.output.query_plan.response_object_set_definitions.len() - 1)
            })
    }

    fn generate_mutation_partition_order_after_partition_generation(&mut self) -> SolveResult<()> {
        if !self.output.operation.attributes.ty.is_mutation() {
            return Ok(());
        }
        let mut partition_to_next_in_order = Vec::new();
        let mut initial_partition = None;
        for neighbor in self.solution.graph.neighbors(self.solution.root_node_ix) {
            if let Node::QueryPartition { .. } = self.solution.graph[neighbor] {
                if let Some(prev) = self
                    .solution
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

        self.output.query_plan.mutation_partition_order = mutation_partition_order;

        Ok(())
    }
}

enum MaybePartitionFieldRecord {
    None,
    Data(PartitionDataFieldRecord),
    Typename(PartitionTypenameFieldRecord),
}

fn to_data_field_or_typename_field(
    field: &QueryField,
    schema: &Schema,
    query_partition_id: QueryPartitionId,
) -> MaybePartitionFieldRecord {
    let Some(response_key) = field.response_key else {
        return MaybePartitionFieldRecord::None;
    };
    if let Some(definition_id) = field.definition_id {
        MaybePartitionFieldRecord::Data(PartitionDataFieldRecord {
            type_condition_ids: field.type_conditions,
            query_partition_id,
            definition_id,
            query_position: field.query_position,
            response_key,
            subgraph_key: field.subgraph_key,
            location: field.location,
            argument_ids: field.argument_ids,
            // All set later
            selection_set_record: PartitionSelectionSetRecord {
                data_field_ids_ordered_by_type_conditions_then_position: IdRange::empty(),
                typename_field_ids_ordered_by_type_conditions_then_position: IdRange::empty(),
            },
            required_fields_record: RequiredFieldSetRecord::default(),
            required_fields_record_by_supergraph: Default::default(),
            output_id: None,
            parent_field_output_id: None,
            selection_set_requires_typename: match definition_id.walk(schema).ty().definition() {
                // If we may encounter an inaccessible object, we have to detect it
                Definition::Union(union) => union.has_inaccessible_member(),
                Definition::Interface(interface) => interface.has_inaccessible_implementor(),
                _ => false,
            },
            shape_ids: IdRange::empty(),
        })
    } else {
        MaybePartitionFieldRecord::Typename(PartitionTypenameFieldRecord {
            type_condition_ids: field.type_conditions,
            response_key,
            query_position: field.query_position,
            location: field.location,
        })
    }
}
