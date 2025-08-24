use query_solver::{
    Edge, Node,
    petgraph::{graph::NodeIndex, visit::EdgeRef as _},
};

use crate::prepare::{
    QueryPartitionId, SolveError, SolveResult,
    cached::builder::{Solver, query_partition::NodeMap},
};

impl<'a> Solver<'a> {
    pub(super) fn generate_mutation_partition_order_after_partition_generation(
        &mut self,
        map: &NodeMap,
    ) -> SolveResult<()> {
        if !self.output.operation.attributes.ty.is_mutation() {
            return Ok(());
        }
        let mut partition_to_next_in_order = Vec::new();
        let mut initial_partition = None;
        for neighbor in self.solution.graph.neighbors(self.solution.root_node_id) {
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

        fn get_query_partition_id(map: &NodeMap, node_id: NodeIndex) -> SolveResult<QueryPartitionId> {
            map.query_partition_to_node
                .iter()
                .find(|(_, id)| *id == node_id)
                .map(|(qpid, _)| *qpid)
                .ok_or_else(|| {
                    tracing::error!("Could not find query partition id for node.");
                    SolveError::InternalError
                })
        }

        let mut mutation_partition_order = Vec::with_capacity(partition_to_next_in_order.len());
        mutation_partition_order.push(get_query_partition_id(map, initial_partition)?);
        partition_to_next_in_order.sort_unstable();

        let mut last = initial_partition;
        while let Ok(i) = partition_to_next_in_order.binary_search_by(|probe| probe.0.cmp(&last)) {
            let (_, next) = partition_to_next_in_order[i];
            mutation_partition_order.push(get_query_partition_id(map, next)?);
            last = next;
        }

        self.output.query_plan.mutation_partition_order = mutation_partition_order;

        Ok(())
    }
}
