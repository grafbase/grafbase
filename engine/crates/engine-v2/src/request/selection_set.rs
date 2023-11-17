use std::fmt::Debug;

use super::OperationFieldId;
use crate::formatter::{ContextAwareDebug, FormatterContext, FormatterContextHolder};

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct OperationSelectionSet {
    pub items: Vec<OperationSelection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationSelection {
    pub operation_field_id: OperationFieldId,
    pub subselection: OperationSelectionSet,
}

impl OperationSelectionSet {
    pub fn empty() -> Self {
        Self { items: vec![] }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &OperationSelection> {
        self.items.iter()
    }
}

impl Extend<OperationSelection> for OperationSelectionSet {
    fn extend<T: IntoIterator<Item = OperationSelection>>(&mut self, iter: T) {
        self.items.extend(iter);
    }
}

impl FromIterator<OperationSelection> for OperationSelectionSet {
    fn from_iter<T: IntoIterator<Item = OperationSelection>>(iter: T) -> Self {
        Self {
            items: iter.into_iter().collect::<Vec<_>>(),
        }
    }
}

impl IntoIterator for OperationSelectionSet {
    type Item = OperationSelection;

    type IntoIter = <Vec<OperationSelection> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a> IntoIterator for &'a OperationSelectionSet {
    type Item = &'a OperationSelection;

    type IntoIter = <&'a Vec<OperationSelection> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl ContextAwareDebug for OperationSelectionSet {
    fn fmt(&self, ctx: &FormatterContext<'_>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RequestSelectionSet")
            .field("items", &ctx.debug(&self.items))
            .finish()
    }
}

impl ContextAwareDebug for OperationSelection {
    fn fmt(&self, ctx: &FormatterContext<'_>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let field = &ctx.opeartion[self.operation_field_id];
        f.debug_struct("RequestSelection")
            .field("name", &ctx.strings[field.name].to_string())
            .field("subselection", &ctx.debug(&self.subselection))
            .finish()
    }
}
