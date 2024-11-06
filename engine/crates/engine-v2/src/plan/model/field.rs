use walker::Iter;

use crate::response::FieldShapeId;

use super::DataField;

impl<'a> DataField<'a> {
    pub(crate) fn shapes(&self) -> impl Iter<Item = FieldShapeId> + 'a {
        self.ctx.operation_plan[self.as_ref().shape_ids].iter().copied()
    }
}
