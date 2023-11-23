use engine_parser::Pos;
use schema::SchemaWalker;

use super::PlannedSelectionSetWalker;
use crate::{
    plan::{OperationPlan, PlanId},
    request::{BoundFragmentSpread, TypeCondition},
};

pub struct PlannedFragmentSpreadWalker<'a> {
    pub(in crate::plan) schema: SchemaWalker<'a, ()>,
    pub(in crate::plan) plan: &'a OperationPlan,
    pub(in crate::plan) plan_id: PlanId,
    pub(in crate::plan) inner: &'a BoundFragmentSpread,
}

impl<'a> PlannedFragmentSpreadWalker<'a> {
    pub fn location(&self) -> Pos {
        self.inner.location
    }

    pub fn fragment(&self) -> PlannedFragmentWalker<'a> {
        PlannedFragmentWalker {
            schema: self.schema,
            plan: self.plan,
            plan_id: self.plan_id,
            inner: self.inner,
        }
    }
}

pub struct PlannedFragmentWalker<'a> {
    pub(in crate::plan) schema: SchemaWalker<'a, ()>,
    pub(in crate::plan) plan: &'a OperationPlan,
    pub(in crate::plan) plan_id: PlanId,
    pub(in crate::plan) inner: &'a BoundFragmentSpread,
}

impl<'a> PlannedFragmentWalker<'a> {
    pub fn name(&self) -> &str {
        let definition = &self.plan.operation[self.inner.fragment_id];
        &definition.name
    }

    pub fn type_condition_name(&self) -> &str {
        let definition = &self.plan.operation[self.inner.fragment_id];
        match definition.type_condition {
            TypeCondition::Interface(interface_id) => self.schema.walk(interface_id).name(),
            TypeCondition::Object(object_id) => self.schema.walk(object_id).name(),
            TypeCondition::Union(union_id) => self.schema.walk(union_id).name(),
        }
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
