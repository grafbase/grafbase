use cynic_parser::{
    executable::{
        ids::{FragmentDefinitionId, OperationDefinitionId, SelectionId, VariableDefinitionId},
        iter::{IdIter, Iter},
        Selection, VariableDefinition,
    },
    ExecutableDocument,
};
use display::SelectionSetDisplay;
use indexmap::IndexSet;

mod display;
mod field_iter;

pub use self::{display::QuerySubsetDisplay, field_iter::FieldIter};

/// Part of a query that was submitted to the API.
///
/// This is a group of fields with the same cache settings, and all the
/// ancestors, variables & fragments required for those fields to make a
/// valid query
#[derive(Clone)]
pub struct QuerySubset {
    pub(crate) operation: OperationDefinitionId,
    partition: Partition,
    variables: IndexSet<VariableDefinitionId>,
}

#[derive(Default, Debug, Clone)]
pub(crate) struct Partition {
    pub selections: IndexSet<SelectionId>,
    pub fragments: IndexSet<FragmentDefinitionId>,
}

impl QuerySubset {
    pub(crate) fn new(
        operation: OperationDefinitionId,
        cache_group: Partition,
        variables: IndexSet<VariableDefinitionId>,
    ) -> Self {
        QuerySubset {
            operation,
            partition: cache_group,
            variables,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.partition.selections.is_empty()
    }

    pub fn extend(&mut self, other: &QuerySubset) {
        self.partition
            .selections
            .extend(other.partition.selections.iter().copied());
        self.partition
            .fragments
            .extend(other.partition.fragments.iter().copied());
        self.variables.extend(other.variables.iter().cloned());
    }

    pub fn as_display<'a>(&'a self, document: &'a ExecutableDocument) -> QuerySubsetDisplay<'a> {
        QuerySubsetDisplay {
            subset: self,
            document,
            include_query_name: false,
        }
    }

    pub fn variables<'a>(
        &'a self,
        document: &'a ExecutableDocument,
    ) -> impl Iterator<Item = VariableDefinition<'a>> + 'a {
        self.variables.iter().map(|id| document.read(*id))
    }

    fn selection_set_display<'a>(&'a self, selections: Iter<'a, Selection<'a>>) -> SelectionSetDisplay<'a> {
        SelectionSetDisplay::new(&self.partition.selections, selections, 0)
    }

    pub(crate) fn selection_iter<'doc, 'subset>(
        &'subset self,
        selection_set: Iter<'doc, Selection<'doc>>,
    ) -> FilteredSelectionSet<'doc, 'subset> {
        FilteredSelectionSet {
            visible_selections: &self.partition.selections,
            selections: selection_set.with_ids(),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct FilteredSelectionSet<'doc, 'subset> {
    visible_selections: &'subset IndexSet<SelectionId>,
    selections: IdIter<'doc, Selection<'doc>>,
}

impl FilteredSelectionSet<'_, '_> {
    pub fn requires_synthetic_typename(self) -> bool {
        let mut result = false;
        for selection in self {
            match selection {
                Selection::Field(field) if field.name() == "__typename" && field.alias().is_none() => {
                    // If we already have a typename there's no point in adding another
                    return false;
                }
                Selection::FragmentSpread(_) | Selection::InlineFragment(_) => {
                    // Technically an inline fragment doesn't always need a typename, but its easier to
                    // just assume that it does.  In particular a `@defer` without a condition but with
                    // conditions inside needs a typename at this level, and this assumption saves
                    // us from bothering to recurse
                    result = true;
                }
                _ => {}
            }
        }
        result
    }
}

impl<'doc, 'subset> Iterator for FilteredSelectionSet<'doc, 'subset> {
    type Item = Selection<'doc>;

    fn next(&mut self) -> Option<Self::Item> {
        for (id, selection) in self.selections.by_ref() {
            if self.visible_selections.contains(&id) {
                return Some(selection);
            }
        }
        None
    }
}
