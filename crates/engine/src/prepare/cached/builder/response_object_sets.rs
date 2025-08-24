use crate::prepare::{
    SolveResult,
    cached::builder::{
        Solver,
        query_partition::{NodeMap, ResponseObjectSetMap},
    },
};

impl Solver<'_> {
    pub(super) fn finalize_response_object_sets_before_shapes(
        &mut self,
        map: &NodeMap,
        response_object_set_map: ResponseObjectSetMap,
    ) -> SolveResult<()> {
        // In the case of shared roots, so a query `{ node { a b } }` being planned as `{ node { a } }` and `{ node { b } }`,
        // we may encounter cases where some plan generates a deeply nested object and a later plan adds a required
        // field to it. However when generate the first one we're not aware of any dependencies
        // yet, so we won't attribute any ResponseObjectSetId to the field output but it's the
        // first one being executed. This leads us to merge the second plan's response fields into the first
        // plan's response objects which aren't tracked and thus cannot be retrieved by dependent
        // plans and we fail.
        //
        // To prevent this we ensure that all DataFields that map to the same QueryField have the
        // same output_id if any.
        for (key, set_id) in response_object_set_map.query_field_id_to_response_object_set {
            for field_id in map.query_field_to_data_field.find_all(key).copied() {
                debug_assert!(
                    self.output.query_plan[field_id].output_id.is_none_or(|id| id == set_id),
                    "Inconsitent ResponseObjectSetId"
                );
                self.output.query_plan[field_id].output_id = Some(set_id);
            }
        }

        Ok(())
    }
}
