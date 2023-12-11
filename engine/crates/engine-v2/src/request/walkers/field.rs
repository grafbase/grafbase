use schema::{FieldId, FieldWalker};

use super::{BoundAnyFieldDefinitionWalker, BoundFieldArgumentWalker, OperationWalker, PlanExt, PlanSelectionSet};
use crate::{
    plan::ExtraFieldId,
    request::{BoundFieldDefinition, BoundFieldId},
};

pub type BoundFieldWalker<'a, Extension = ()> = OperationWalker<'a, BoundFieldId, (), Extension>;

impl<'a, E: Copy> BoundFieldWalker<'a, E> {
    pub fn response_key_str(&self) -> &'a str {
        &self.operation.response_keys[self.bound_response_key.into()]
    }

    pub fn definition(&self) -> BoundAnyFieldDefinitionWalker<'a, E> {
        self.walk_with(self.definition_id, ())
    }
}

impl<'a> BoundFieldWalker<'a, PlanExt<'a>> {
    pub fn selection_set(&self) -> Option<PlanSelectionSet<'a>> {
        self.selection_set_id
            .filter(|id| self.ext.attribution.selection_set(*id))
            .map(|id| PlanSelectionSet::Query(self.walk(id)))
    }
}

impl<'a> std::fmt::Debug for BoundFieldWalker<'a, PlanExt<'a>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundFieldWalker")
            .field("definition", &self.definition())
            .field("selection_set", &self.selection_set())
            .finish()
    }
}

pub enum PlanField<'a> {
    Query(OperationWalker<'a, (BoundFieldId, &'a BoundFieldDefinition), FieldId, PlanExt<'a>>),
    Extra(OperationWalker<'a, ExtraFieldId, FieldId, PlanExt<'a>>),
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
            PlanField::Query(walker) => walker.walk_with(walker.wrapped.0, ()).response_key_str(),
            PlanField::Extra(walker) => walker.as_attribution_walker().expected_key(),
        }
    }

    pub fn selection_set(&self) -> Option<PlanSelectionSet<'a>> {
        match self {
            PlanField::Query(walker) => walker.walk_with(walker.wrapped.0, ()).selection_set(),
            PlanField::Extra(walker) => walker
                .as_attribution_walker()
                .selection_set()
                .map(|selection_set| PlanSelectionSet::Extra(walker.walk_with(selection_set.id(), ()))),
        }
    }

    pub fn bound_arguments(&self) -> impl ExactSizeIterator<Item = BoundFieldArgumentWalker<'a, PlanExt<'a>>> + 'a {
        let arguments = match self {
            PlanField::Query(walker) => walker
                .wrapped
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
