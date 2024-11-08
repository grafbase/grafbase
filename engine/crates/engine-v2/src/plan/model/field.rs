use walker::Iter;

use crate::response::FieldShapeId;

use super::DataPlanField;

impl<'a> DataPlanField<'a> {
    pub(crate) fn shapes(&self) -> impl Iter<Item = FieldShapeId> + 'a {
        self.ctx.operation_plan[self.as_ref().shape_ids].iter().copied()
    }
}

impl std::fmt::Debug for DataPlanField<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataPlanField")
            .field("key", &self.key)
            .field("location", &self.location)
            .field("definition", &self.definition())
            .field("arguments", &self.arguments())
            .field("selection_set", &self.selection_set())
            .finish()
    }
}
