mod argument;
mod data;
mod extension;
mod typename;

use walker::Iter;

use crate::prepare::FieldShapeId;

pub(crate) use argument::*;
pub(crate) use data::*;
pub(crate) use typename::*;

impl<'a> PartitionDataField<'a> {
    pub(crate) fn shapes(&self) -> impl Iter<Item = FieldShapeId> + 'a {
        self.ctx.cached.query_plan[self.as_ref().shape_ids].iter().copied()
    }
}

impl std::fmt::Debug for PartitionDataField<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataPlanField")
            .field("key", &self.response_key)
            .field("location", &self.location)
            .field("definition", &self.definition())
            .field("selection_set", &self.selection_set())
            .finish()
    }
}
