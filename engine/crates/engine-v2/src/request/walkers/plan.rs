use std::collections::VecDeque;

use engine_parser::types::OperationType;

use super::{
    BoundFieldArgumentWalker, BoundFieldDefinitionWalker, BoundFieldWalker, BoundFragmentSpreadWalker,
    BoundInlineFragmentWalker, BoundSelectionWalker, HasVariables, OperationWalker, PlanFilter, SelectionSet,
};
use crate::{
    execution::Variables,
    plan::{Attribution, EntityType, PlanOutput},
    request::{BoundFieldId, SelectionSetType},
};

pub type PlanWalker<'a> = OperationWalker<'a, (), (), PlanExt<'a>>;
pub type PlanOperationWalker<'a> = OperationWalker<'a, &'a PlanOutput, (), PlanExt<'a>>;
pub type PlanFieldDefinition<'a> = BoundFieldDefinitionWalker<'a, PlanExt<'a>>;
pub type PlanSelection<'a> = BoundSelectionWalker<'a, PlanExt<'a>>;
pub type PlanFragmentSpread<'a> = BoundFragmentSpreadWalker<'a, PlanExt<'a>>;
pub type PlanInlineFragment<'a> = BoundInlineFragmentWalker<'a, PlanExt<'a>>;
pub type PlanField<'a> = BoundFieldWalker<'a, PlanExt<'a>>;
pub type PlanFieldArgument<'a> = BoundFieldArgumentWalker<'a, PlanExt<'a>>;

#[derive(Clone, Copy)]
pub struct PlanExt<'a> {
    pub attibution: &'a Attribution,
    pub variables: &'a Variables<'a>,
}

impl<'a> PlanFilter for PlanExt<'a> {
    fn field(&self, id: BoundFieldId) -> bool {
        self.attibution.field(id)
    }

    fn selection_set(&self, id: crate::request::BoundSelectionSetId) -> bool {
        self.attibution.selection_set(id)
    }
}

impl<'a> HasVariables for PlanExt<'a> {
    fn variables(&self) -> &Variables<'_> {
        self.variables
    }
}

impl<'a> PlanOperationWalker<'a> {
    pub fn ty(&self) -> OperationType {
        self.operation.ty
    }

    pub fn name(&self) -> Option<&'a str> {
        self.operation.name.as_deref()
    }

    pub fn selection_set(&self) -> impl SelectionSet<'a, PlanExt<'a>> {
        PlanOperationSelectionSet(*self)
    }
}

struct PlanOperationSelectionSet<'a>(PlanOperationWalker<'a>);

impl<'a> SelectionSet<'a, PlanExt<'a>> for PlanOperationSelectionSet<'a> {
    fn ty(&self) -> SelectionSetType {
        match self.0.inner.entity_type {
            EntityType::Interface(id) => SelectionSetType::Interface(id),
            EntityType::Object(id) => SelectionSetType::Object(id),
        }
    }
}

impl<'a> IntoIterator for PlanOperationSelectionSet<'a> {
    type Item = BoundSelectionWalker<'a, PlanExt<'a>>;

    type IntoIter = PlanRootFieldsIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PlanRootFieldsIterator {
            walker: self.0.walk(()),
            fields: self.0.inner.fields.iter().copied().collect(),
        }
    }
}

pub struct PlanRootFieldsIterator<'a> {
    walker: OperationWalker<'a, (), (), PlanExt<'a>>,
    fields: VecDeque<BoundFieldId>,
}

impl<'a> Iterator for PlanRootFieldsIterator<'a> {
    type Item = BoundSelectionWalker<'a, PlanExt<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let field = self.fields.pop_front()?;
        Some(BoundSelectionWalker::Field(self.walker.walk(field)))
    }
}
