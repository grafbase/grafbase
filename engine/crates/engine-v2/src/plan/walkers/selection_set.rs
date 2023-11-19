use std::collections::VecDeque;

use engine_parser::Pos;
use schema::{ObjectId, SchemaWalker};

use super::{GroupedFieldSet, PlannedFieldWalker, PlannedFragmentSpreadWalker, PlannedInlineFragmentWalker};
use crate::{
    plan::{OperationPlan, PlanId},
    request::{BoundSelection, BoundSelectionSetId},
};

pub struct PlannedSelectionSetWalker<'a> {
    pub(in crate::plan) schema: SchemaWalker<'a, ()>,
    pub(in crate::plan) plan: &'a OperationPlan,
    pub(in crate::plan) plan_id: PlanId,
    pub(in crate::plan) id: BoundSelectionSetId,
}

impl<'a> PlannedSelectionSetWalker<'a> {
    pub fn new(
        schema: SchemaWalker<'a, ()>,
        plan: &'a OperationPlan,
        plan_id: PlanId,
        id: BoundSelectionSetId,
    ) -> Self {
        Self {
            schema,
            plan,
            plan_id,
            id,
        }
    }

    pub fn collect_fields(&self, concrete_object_id: ObjectId) -> GroupedFieldSet<'a> {
        let mut grouped_field_set = GroupedFieldSet::new(self.schema, self.plan, self.plan_id, concrete_object_id);
        grouped_field_set.collect_fields(self.id);
        grouped_field_set
    }
}

pub enum PlannedSelectionWalker<'a> {
    Field(PlannedFieldWalker<'a>),
    FragmentSpread(PlannedFragmentSpreadWalker<'a>),
    InlineFragment(PlannedInlineFragmentWalker<'a>),
}

impl<'a> PlannedSelectionWalker<'a> {
    pub fn location(&self) -> Pos {
        match self {
            PlannedSelectionWalker::Field(field) => field.location(),
            PlannedSelectionWalker::FragmentSpread(spread) => spread.location(),
            PlannedSelectionWalker::InlineFragment(fragment) => fragment.location(),
        }
    }
}

impl<'a> IntoIterator for PlannedSelectionSetWalker<'a> {
    type Item = PlannedSelectionWalker<'a>;

    type IntoIter = PlannedSelectionIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PlannedSelectionIterator {
            schema: self.schema,
            plan: self.plan,
            plan_id: self.plan_id,
            selections: self.plan.operation[self.id].items.iter().collect(),
        }
    }
}

pub struct PlannedSelectionIterator<'a> {
    schema: SchemaWalker<'a, ()>,
    plan: &'a OperationPlan,
    plan_id: PlanId,
    selections: VecDeque<&'a BoundSelection>,
}

impl<'a> Iterator for PlannedSelectionIterator<'a> {
    type Item = PlannedSelectionWalker<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(selection) = self.selections.pop_front() {
            match selection {
                BoundSelection::Field(id) => {
                    if self.plan.attribution[*id] == self.plan_id {
                        let bound_field = &self.plan.operation[*id];
                        return Some(PlannedSelectionWalker::Field(PlannedFieldWalker {
                            schema_field: self
                                .schema
                                .walk(self.plan.operation[bound_field.definition_id].field_id),
                            plan: self.plan,
                            plan_id: self.plan_id,
                            bound_field,
                            id: *id,
                        }));
                    }
                }
                BoundSelection::FragmentSpread(spread) => {
                    if self.plan.attribution[spread.selection_set_id].contains(&self.plan_id) {
                        return Some(PlannedSelectionWalker::FragmentSpread(PlannedFragmentSpreadWalker {
                            schema: self.schema,
                            plan: self.plan,
                            plan_id: self.plan_id,
                            inner: spread,
                        }));
                    }
                }
                BoundSelection::InlineFragment(fragment) => {
                    if self.plan.attribution[fragment.selection_set_id].contains(&self.plan_id) {
                        return Some(PlannedSelectionWalker::InlineFragment(PlannedInlineFragmentWalker {
                            schema: self.schema,
                            plan: self.plan,
                            plan_id: self.plan_id,
                            inner: fragment,
                        }));
                    }
                }
            }
        }
        None
    }
}
