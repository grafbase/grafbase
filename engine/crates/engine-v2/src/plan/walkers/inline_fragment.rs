use engine_parser::Pos;
use schema::SchemaWalker;

use super::PlannedSelectionSetWalker;
use crate::{
    plan::{OperationPlan, PlanId},
    request::{BoundInlineFragment, TypeCondition},
};

pub struct PlannedInlineFragmentWalker<'a> {
    pub(in crate::plan) schema: SchemaWalker<'a, ()>,
    pub(in crate::plan) plan: &'a OperationPlan,
    pub(in crate::plan) plan_id: PlanId,
    pub(in crate::plan) inner: &'a BoundInlineFragment,
}

impl<'a> PlannedInlineFragmentWalker<'a> {
    pub fn location(&self) -> Pos {
        self.inner.location
    }

    pub fn type_condition_name(&self) -> Option<&str> {
        self.inner.type_condition.map(|cond| match cond {
            TypeCondition::Interface(interface_id) => self.schema.walk(interface_id).name(),
            TypeCondition::Object(object_id) => self.schema.walk(object_id).name(),
            TypeCondition::Union(union_id) => self.schema.walk(union_id).name(),
        })
    }

    pub fn selection_set(&self) -> PlannedSelectionSetWalker<'a> {
        PlannedSelectionSetWalker {
            schema: self.schema,
            plan: self.plan,
            plan_id: self.plan_id,
            id: self.inner.selection_set_id,
        }
    }
}
