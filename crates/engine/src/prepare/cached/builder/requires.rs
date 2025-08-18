use itertools::Itertools;
use query_solver::{
    Edge,
    petgraph::{graph::NodeIndex, visit::EdgeRef},
};
use walker::Walk as _;

use crate::prepare::PartitionFieldId;

use super::{RequiredFieldSetRecord, RequredFieldRecord, SolveResult, Solver, query_partition::NodeMap};

impl Solver<'_> {
    pub(super) fn populate_requirements_after_partition_generation(&mut self, map: &NodeMap) -> SolveResult<()> {
        for (query_partition_id, query_partition_root_node_ix) in map.query_partition_to_node.iter().copied() {
            self.output.query_plan[query_partition_id].required_fields_record =
                self.create_required_field_set(map, query_partition_root_node_ix, Edge::RequiredBySubgraph);
        }

        for (i, field_id) in map.node_to_field.iter().copied().enumerate() {
            let node_id = NodeIndex::new(i);
            match field_id {
                Some(PartitionFieldId::Data(field_id)) => {
                    self.output.query_plan[field_id].required_fields_record =
                        self.create_required_field_set(map, node_id, Edge::RequiredBySubgraph);
                    self.output.query_plan[field_id].required_fields_record_by_supergraph =
                        self.create_required_field_set(map, node_id, Edge::RequiredBySupergraph);
                }
                Some(PartitionFieldId::Lookup(field_id)) => {
                    self.output.query_plan[field_id].required_fields_record_by_supergraph =
                        self.create_required_field_set(map, node_id, Edge::RequiredBySupergraph);
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn create_required_field_set(
        &self,
        map: &NodeMap,
        dependent_node_id: NodeIndex,
        kind: Edge,
    ) -> RequiredFieldSetRecord {
        let mut dependencies = self
            .solution
            .graph
            .edges(dependent_node_id)
            .filter(|edge| edge.weight() == &kind)
            .map(|edge| edge.target())
            .collect::<Vec<_>>();

        // We rely on the fact that fields in the SolutionGraph are created in depth order, so a
        // node will always have a higher id than its parents. This means that the node with the
        // minimum id in the dependencies is necessarily a scalar requirement or a parent
        // field of other field requirements. From this parent we just do a breadth-first search
        // to find other dependencies and build so iteratively a FieldSet structure.
        let mut required_fields = Vec::new();
        while let Some(i) = dependencies.iter().position_min() {
            let node_id = dependencies.swap_remove(i);
            let matching_field_id = self.solution.graph[node_id]
                .as_query_field()
                .and_then(|id| self.solution[id].matching_field_id)
                .expect("We depend on this field, so it must be a QueryField and it must have a SchemaFieldId");
            let data_field_id = map.node_to_field[node_id.index()]
                .expect("We depend on this field, so it must be a DataField")
                .as_data()
                .expect("Cannot depend on Lookup fields as we create them after planning");

            // With shared roots, so a query like `query { x { n1 n2 } }` that ends up being split
            // into `query { x { n1 } }` and `query { x { n2 } }`, we can end up with a require
            // edge targeting both `x`, but we only need either `n1` or `n2` so we could create an
            // unnecessary dependencies over one of the query partition.
            let subselection_record = self.create_subselection(map, node_id, &mut dependencies);
            if subselection_record.is_empty()
                && matching_field_id
                    .walk(self.schema)
                    .definition()
                    .ty()
                    .definition_id
                    .is_composite_type()
            {
                continue;
            }
            required_fields.push(RequredFieldRecord {
                data_field_id,
                matching_field_id,
                subselection_record,
            });
        }
        required_fields.into()
    }

    fn create_subselection(
        &self,
        map: &NodeMap,
        parent: NodeIndex,
        dependencies: &mut Vec<NodeIndex>,
    ) -> RequiredFieldSetRecord {
        let mut subselection = Vec::new();
        let mut stack = vec![parent];
        while let Some(parent) = stack.pop() {
            for edge in self.solution.graph.edges(parent) {
                match edge.weight() {
                    Edge::Field => {
                        let Some(i) = dependencies.iter().position(|ix| *ix == edge.target()) else {
                            continue;
                        };
                        dependencies.swap_remove(i);
                        let matching_field_id = self.solution.graph[edge.target()]
                            .as_query_field()
                            .and_then(|id| self.solution[id].matching_field_id)
                            .expect(
                                "We depend on this field, so it must be a QueryField and it must have a SchemaFieldId",
                            );
                        let data_field_id = map.node_to_field[edge.target().index()]
                            .expect("We depend on this field, so it must be a DataField")
                            .as_data()
                            .expect("Cannot depend on Lookup fields as we create them after planning");

                        subselection.push(RequredFieldRecord {
                            data_field_id,
                            matching_field_id,
                            subselection_record: self.create_subselection(map, edge.target(), dependencies),
                        });
                    }
                    Edge::QueryPartition => {
                        stack.push(edge.target());
                    }
                    _ => {}
                }
            }
        }
        subselection.into()
    }
}
