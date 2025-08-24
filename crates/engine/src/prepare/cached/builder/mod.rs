mod modifiers;
mod mutation_order;
mod query_partition;
mod requires;
mod response_object_sets;
mod shapes;

use operation::Operation;
use query_solver::SolvedQuery;
use schema::Schema;

use super::*;

pub(super) struct Solver<'a> {
    schema: &'a Schema,
    solution: SolvedQuery,
    output: CachedOperation,
}

impl<'a> Solver<'a> {
    pub(super) fn solve(
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
                    partition_input_id: Vec::new(),
                    mutation_partition_order: Vec::new(),
                    shared_type_conditions: std::mem::take(&mut solution.shared_type_conditions),
                    field_shape_refs: Vec::new(),
                    field_arguments: Vec::new(),
                    data_fields: Vec::with_capacity(solution.fields.len()),
                    data_field_output_id: Vec::new(),
                    typename_fields: Vec::new(),
                    root_response_object_set_id: ResponseObjectSetId::from(0usize),
                    response_object_set_definitions: vec![ResponseObjectSetMetadataRecord {
                        ty_id: operation.root_object_id.into(),
                        query_partition_ids: Vec::new(),
                    }],
                    response_data_fields: Default::default(),
                    response_typename_fields: Default::default(),
                    query_modifiers: Default::default(),
                    response_modifier_definitions: Vec::new(),
                    lookup_fields: Vec::new(),
                    lookup_field_output_id: Vec::new(),
                },
                operation,
                shapes: Shapes::default(),
            },
            solution,
        })
    }

    pub(super) fn into_cached_operation(mut self) -> SolveResult<CachedOperation> {
        let (node_map, mut response_object_set_map) = self.generate_query_partitions()?;

        self.generate_mutation_partition_order_after_partition_generation(&node_map)?;

        self.populate_requirements_after_partition_generation(&node_map)?;

        self.populate_modifiers_after_partition_generation(&node_map, &mut response_object_set_map)?;

        self.finalize_response_object_sets_before_shapes(&node_map, response_object_set_map)?;

        self.populate_shapes_after_query_plan();

        Ok(self.output)
    }
}
