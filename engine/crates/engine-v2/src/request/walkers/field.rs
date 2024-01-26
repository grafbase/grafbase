use schema::{FieldId, FieldWalker};

use super::{
    BoundFieldArgumentWalker, BoundSelectionSetWalker, ExecutorWalkContext, OperationWalker, PlanSelectionSet,
};
use crate::{
    plan::ExtraFieldId,
    request::{BoundField, BoundFieldId, Location},
    response::{BoundResponseKey, ResponseKey},
};

pub type BoundFieldWalker<'a, CtxOrUnit = ()> = OperationWalker<'a, BoundFieldId, (), CtxOrUnit>;

impl<'a, F: Copy, C: Copy> OperationWalker<'a, BoundFieldId, F, C> {
    pub fn bound_arguments(&self) -> impl ExactSizeIterator<Item = BoundFieldArgumentWalker<'a, C>> + 'a
    where
        C: 'a,
        F: 'a,
    {
        let walker = *self;
        let arguments = match self.as_ref() {
            BoundField::Field { arguments_id, .. } => &self.operation[*arguments_id],
            BoundField::TypeName { .. } => self.operation.empty_arguments(),
        };
        arguments
            .iter()
            .map(move |argument| walker.walk_with(argument, argument.input_value_id))
    }

    pub fn schema_field(&self) -> Option<FieldWalker<'a>> {
        match self.as_ref() {
            BoundField::Field { field_id, .. } => Some(self.schema_walker.walk(*field_id)),
            BoundField::TypeName { .. } => None,
        }
    }

    pub fn bound_response_key(&self) -> BoundResponseKey {
        self.as_ref().bound_response_key()
    }

    pub fn response_key(&self) -> ResponseKey {
        self.as_ref().bound_response_key().into()
    }

    pub fn response_key_str(&self) -> &'a str {
        &self.operation.response_keys[self.response_key()]
    }

    pub fn name_location(&self) -> Location {
        self.as_ref().name_location()
    }

    pub fn alias(&self) -> Option<&'a str> {
        Some(self.response_key_str()).filter(|&key| match self.as_ref() {
            BoundField::TypeName { .. } => key != "__typename",
            BoundField::Field { field_id, .. } => key != self.schema_walker.walk(*field_id).name(),
        })
    }
}

impl<'a, F: Copy, C: Copy> OperationWalker<'a, BoundFieldId, F, C> {}

impl<'a, F> OperationWalker<'a, BoundFieldId, F, ()> {
    pub fn selection_set(&self) -> Option<BoundSelectionSetWalker<'a>> {
        self.as_ref().selection_set_id().map(|id| self.walk_with(id, ()))
    }
}

impl<'a, F> OperationWalker<'a, BoundFieldId, F, ExecutorWalkContext<'a>> {
    pub fn selection_set(&self) -> Option<PlanSelectionSet<'a>> {
        self.as_ref()
            .selection_set_id()
            .filter(|id| self.ctx.attribution.selection_set(*id))
            .map(|id| PlanSelectionSet::Query(self.walk_with(id, ())))
    }
}

impl<'a> std::fmt::Debug for BoundFieldWalker<'a, ()> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.as_ref() {
            BoundField::TypeName { .. } => "__typename".fmt(f),
            BoundField::Field { field_id, .. } => {
                let mut fmt = f.debug_struct("BoundField");
                let name = self.schema_walker.walk(*field_id).name();
                if self.response_key_str() != name {
                    fmt.field("key", &self.response_key_str());
                }
                fmt.field("name", &name)
                    .field("selection_set", &self.selection_set())
                    .finish()
            }
        }
    }
}

impl<'a> std::fmt::Debug for BoundFieldWalker<'a, ExecutorWalkContext<'a>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.as_ref() {
            BoundField::TypeName { .. } => "__typename".fmt(f),
            BoundField::Field { field_id, .. } => {
                let mut fmt = f.debug_struct("BoundField");
                let name = self.schema_walker.walk(*field_id).name();
                if self.response_key_str() != name {
                    fmt.field("key", &self.response_key_str());
                }
                fmt.field("name", &name)
                    .field("selection_set", &self.selection_set())
                    .finish()
            }
        }
    }
}

pub enum PlanField<'a> {
    Query(OperationWalker<'a, BoundFieldId, FieldId, ExecutorWalkContext<'a>>),
    Extra(OperationWalker<'a, ExtraFieldId, FieldId, ExecutorWalkContext<'a>>),
}

impl<'a> std::ops::Deref for PlanField<'a> {
    type Target = FieldWalker<'a>;

    fn deref(&self) -> &Self::Target {
        match self {
            PlanField::Query(walker) => &walker.schema_walker,
            PlanField::Extra(walker) => &walker.schema_walker,
        }
    }
}

impl<'a> PlanField<'a> {
    pub fn response_key_str(&self) -> &'a str {
        match self {
            PlanField::Query(walker) => walker.response_key_str(),
            PlanField::Extra(walker) => walker.as_attribution_walker().expected_key(),
        }
    }

    pub fn selection_set(&self) -> Option<PlanSelectionSet<'a>> {
        match self {
            PlanField::Query(walker) => walker.selection_set(),
            PlanField::Extra(walker) => walker
                .as_attribution_walker()
                .selection_set()
                .map(|selection_set| PlanSelectionSet::Extra(walker.walk_with(selection_set.id(), ()))),
        }
    }

    pub fn bound_arguments(
        &self,
    ) -> Box<dyn ExactSizeIterator<Item = BoundFieldArgumentWalker<'a, ExecutorWalkContext<'a>>> + 'a> {
        match self {
            PlanField::Query(walker) => Box::new(walker.bound_arguments()),
            PlanField::Extra(_) => Box::new(Vec::with_capacity(0).into_iter()),
        }
    }
}

impl<'a> std::fmt::Debug for PlanField<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut fmt = f.debug_struct("PlanField");
        fmt.field("name", &self.name());
        if matches!(self, PlanField::Extra(_)) {
            fmt.field("extra", &true);
        }
        if self.response_key_str() != self.name() {
            fmt.field("key", &self.response_key_str());
        }
        if let Some(selection_set) = self.selection_set() {
            fmt.field("selection_set", &selection_set);
        }
        fmt.finish()
    }
}
