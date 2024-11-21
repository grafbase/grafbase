use itertools::Itertools;
use query_solver::{
    petgraph::{graph::NodeIndex, visit::EdgeRef},
    SolutionEdge as Edge,
};

use crate::operation::{DataFieldId, RequiredFieldSetItemRecord, RequiredFieldSetRecord, SolveResult};

use super::Solver;

impl Solver<'_> {
    pub(super) fn populate_requirements_after_partition_generation(&mut self) -> SolveResult<()> {
        self.node_to_field.sort_unstable();

        debug_assert!(!self.query_partition_to_node.is_empty());
        let query_partition_to_node = std::mem::take(&mut self.query_partition_to_node);
        for (query_partition_id, query_partition_root_node_ix) in query_partition_to_node.iter().copied() {
            self.operation[query_partition_id].required_fields_record =
                self.create_required_field_set(query_partition_root_node_ix, Edge::RequiredBySubgraph);
        }
        self.query_partition_to_node = query_partition_to_node;

        for (field_id, node_ix) in std::mem::take(&mut self.field_to_node) {
            self.operation[field_id].required_fields_record =
                self.create_required_field_set(node_ix, Edge::RequiredBySubgraph);
            self.operation[field_id].required_fields_record_by_supergraph =
                self.create_required_field_set(node_ix, Edge::RequiredBySupergraph);
        }

        Ok(())
    }

    fn create_required_field_set(&self, dependent_node_ix: NodeIndex, kind: Edge) -> RequiredFieldSetRecord {
        let mut dependencies = self
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
            for edge in self.graph.edges(parent) {
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

    fn get_field_id_for(&self, node_ix: NodeIndex) -> Option<DataFieldId> {
        self.node_to_field
            .binary_search_by(|probe| probe.0.cmp(&node_ix))
            .map(|i| self.node_to_field[i].1)
            .ok()
    }
}
