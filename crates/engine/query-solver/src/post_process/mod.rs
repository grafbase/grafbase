mod mutation_order;
mod partition_cycles;
mod response_key;

use std::marker::PhantomData;

use operation::{Operation, OperationContext};
use schema::Schema;

use crate::{query::SolvedQuery, solve::CrudeSolvedQuery};

pub(crate) fn post_process(schema: &Schema, operation: &mut Operation, mut query: CrudeSolvedQuery) -> SolvedQuery {
    response_key::adjust_response_keys_to_avoid_collisions(schema, operation, &mut query);

    if Some(operation.root_object_id) == schema.graph.root_operation_types_record.mutation_id {
        let root_fields = mutation_order::ensure_mutation_execution_order(&mut query);
        // We already handled query partitions in a more specific way, so we don't want this
        // function to touch them. So it starts from the root field's selection sets instead of
        // the root selection set.
        partition_cycles::split_query_partition_dependency_cycles(&mut query, root_fields);
    } else {
        let starting_nodes = vec![query.root_node_ix];
        partition_cycles::split_query_partition_dependency_cycles(&mut query, starting_nodes);
    }

    let query = SolvedQuery {
        step: PhantomData,
        graph: query.graph,
        root_node_ix: query.root_node_ix,
        fields: query.fields,
        shared_type_conditions: query.shared_type_conditions,
        deduplicated_flat_sorted_executable_directives: query.deduplicated_flat_sorted_executable_directives,
        root_selection_set_id: query.root_selection_set_id,
        selection_sets: query.selection_sets,
    };

    tracing::debug!(
        "Solution:\n{}",
        query.to_pretty_dot_graph(OperationContext { schema, operation })
    );

    query
}
