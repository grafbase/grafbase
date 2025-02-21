use itertools::Itertools;
use query_solver::{
    Edge,
    petgraph::{graph::NodeIndex, visit::EdgeRef},
};

use crate::prepare::PartitionFieldId;

use super::{PartitionDataFieldId, RequiredFieldSetItemRecord, RequiredFieldSetRecord, SolveResult};

use super::Solver;

impl Solver<'_> {
    pub(super) fn populate_requirements_after_partition_generation(&mut self) -> SolveResult<()> {
        debug_assert!(!self.query_partition_to_node.is_empty());

        let query_partition_to_node = std::mem::take(&mut self.query_partition_to_node);
        for (query_partition_id, query_partition_root_node_ix) in query_partition_to_node.iter().copied() {
            self.output.query_plan[query_partition_id].required_fields_record =
                self.create_required_field_set(query_partition_root_node_ix, Edge::RequiredBySubgraph);
        }
        self.query_partition_to_node = query_partition_to_node;

        for (node_ix, field_id) in self.node_to_field.iter().enumerate() {
            let Some(PartitionFieldId::Data(field_id)) = *field_id else {
                continue;
            };
            self.output.query_plan[field_id].required_fields_record =
                self.create_required_field_set(NodeIndex::new(node_ix), Edge::RequiredBySubgraph);
            self.output.query_plan[field_id].required_fields_record_by_supergraph =
                self.create_required_field_set(NodeIndex::new(node_ix), Edge::RequiredBySupergraph);
        }

        Ok(())
    }

    fn create_required_field_set(&self, dependent_node_ix: NodeIndex, kind: Edge) -> RequiredFieldSetRecord {
        let mut dependencies = self
            .solution
            .graph
            .edges(dependent_node_ix)
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
            let start = dependencies.swap_remove(i);
            required_fields.push(RequiredFieldSetItemRecord {
                data_field_id: self.get_field_id_for(start).unwrap(),
                subselection_record: self.create_subselection(start, &mut dependencies),
            });
        }
        required_fields.into()
    }

    fn create_subselection(&self, parent: NodeIndex, dependencies: &mut Vec<NodeIndex>) -> RequiredFieldSetRecord {
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
                        subselection.push(RequiredFieldSetItemRecord {
                            data_field_id: self.get_field_id_for(edge.target()).unwrap(),
                            subselection_record: self.create_subselection(edge.target(), dependencies),
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

    fn get_field_id_for(&self, node_ix: NodeIndex) -> Option<PartitionDataFieldId> {
        self.node_to_field[node_ix.index()]
            .as_ref()
            .and_then(PartitionFieldId::as_data)
    }
}
