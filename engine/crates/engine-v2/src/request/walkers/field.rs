use schema::{FieldId, FieldWalker};

use super::{
    BoundAnyFieldDefinitionWalker, BoundFieldArgumentWalker, BoundSelectionSetWalker, ExecutorWalkContext,
    OperationWalker, PlanSelectionSet,
};
use crate::{
    plan::ExtraFieldId,
    request::{BoundAnyFieldDefinitionId, BoundFieldDefinition, BoundFieldId},
    response::BoundResponseKey,
};

pub type BoundFieldWalker<'a, CtxOrUnit = ()> = OperationWalker<'a, BoundFieldId, (), CtxOrUnit>;

impl<'a, C: Copy> BoundFieldWalker<'a, C> {
    pub fn bound_response_key(&self) -> BoundResponseKey {
        self.as_ref().bound_response_key
    }

    pub fn response_key_str(&self) -> &'a str {
        &self.operation.response_keys[self.as_ref().bound_response_key.into()]
    }

    pub fn definition_id(&self) -> BoundAnyFieldDefinitionId {
        self.as_ref().definition_id
    }

    pub fn definition(&self) -> BoundAnyFieldDefinitionWalker<'a, C> {
        self.walk_with(self.as_ref().definition_id, ())
    }
}

impl<'a> BoundFieldWalker<'a, ()> {
    pub fn selection_set(&self) -> Option<BoundSelectionSetWalker<'a>> {
        self.as_ref().selection_set_id.map(|id| self.walk(id))
    }
}

impl<'a> BoundFieldWalker<'a, ExecutorWalkContext<'a>> {
    pub fn selection_set(&self) -> Option<PlanSelectionSet<'a>> {
        self.as_ref()
            .selection_set_id
            .filter(|id| self.ctx.attribution.selection_set(*id))
            .map(|id| PlanSelectionSet::Query(self.walk(id)))
    }
}

impl<'a> std::fmt::Debug for BoundFieldWalker<'a, ()> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundFieldWalker")
            .field("definition", &self.definition())
            .field("selection_set", &self.selection_set())
            .finish()
    }
}

impl<'a> std::fmt::Debug for BoundFieldWalker<'a, ExecutorWalkContext<'a>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundFieldWalker")
            .field("definition", &self.definition())
            .field("selection_set", &self.selection_set())
            .finish()
    }
}

pub enum PlanField<'a> {
    Query(OperationWalker<'a, (BoundFieldId, &'a BoundFieldDefinition), FieldId, ExecutorWalkContext<'a>>),
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
            PlanField::Query(walker) => walker.walk_with(walker.item.0, ()).response_key_str(),
            PlanField::Extra(walker) => walker.as_attribution_walker().expected_key(),
        }
    }

    pub fn selection_set(&self) -> Option<PlanSelectionSet<'a>> {
        match self {
            PlanField::Query(walker) => walker.walk_with(walker.item.0, ()).selection_set(),
            PlanField::Extra(walker) => walker
                .as_attribution_walker()
                .selection_set()
                .map(|selection_set| PlanSelectionSet::Extra(walker.walk_with(selection_set.id(), ()))),
        }
    }

    pub fn bound_arguments(
        &self,
    ) -> impl ExactSizeIterator<Item = BoundFieldArgumentWalker<'a, ExecutorWalkContext<'a>>> + 'a {
        let arguments = match self {
            PlanField::Query(walker) => walker
                .item
                .1
                .arguments
                .iter()
                .map(move |argument| walker.walk_with(argument, argument.input_value_id))
                .collect(),
            PlanField::Extra(_) => vec![],
        };
        arguments.into_iter()
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
