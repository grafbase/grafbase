use engine_parser::Pos;
use schema::FieldWalker;

use super::PlannedSelectionSetWalker;
use crate::{
    plan::{OperationPlan, PlanId},
    request::{BoundField, BoundFieldId, BoundFieldWalker, OperationFieldArgumentWalker},
};

pub struct PlannedFieldWalker<'a> {
    pub(in crate::plan) schema_field: FieldWalker<'a>,
    pub(in crate::plan) plan: &'a OperationPlan,
    pub(in crate::plan) plan_id: PlanId,
    pub(in crate::plan) bound_field: &'a BoundField,
    pub(in crate::plan) id: BoundFieldId,
}

impl<'a> PlannedFieldWalker<'a> {
    pub fn location(&self) -> Pos {
        self.plan.operation[self.bound_field.definition_id].name_location
    }

    pub fn bound_arguments<'s>(&'s self) -> impl ExactSizeIterator<Item = OperationFieldArgumentWalker<'s>> + 's
    where
        'a: 's,
    {
        self.bound_walker().bound_arguments()
    }

    pub fn selection_set(&self) -> Option<PlannedSelectionSetWalker<'a>> {
        if self.plan.attribution[self.bound_field.selection_set_id].contains(&self.plan_id) {
            Some(PlannedSelectionSetWalker {
                schema: self.schema_field.walk(()),
                plan: self.plan,
                plan_id: self.plan_id,
                id: self.bound_field.selection_set_id,
            })
        } else {
            None
        }
    }

    fn bound_walker(&self) -> BoundFieldWalker<'_> {
        BoundFieldWalker::new(self.schema_field, &self.plan.operation, self.bound_field, self.id)
    }
}

impl<'a> std::ops::Deref for PlannedFieldWalker<'a> {
    type Target = FieldWalker<'a>;

    fn deref(&self) -> &Self::Target {
        &self.schema_field
    }
}
