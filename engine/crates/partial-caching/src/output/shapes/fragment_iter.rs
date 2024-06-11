use cynic_parser::executable::{FragmentDefinition, InlineFragment, Selection};

use crate::{query_subset::FilteredSelectionSet, QuerySubset};

use super::building::MergedSelection;

/// An iterator over the fragments of a selection set.
///
/// This will recurse into any selection sets nested inside fragments, but not fields.
pub struct FragmentIter<'doc, 'ctx> {
    root_selection: std::slice::Iter<'ctx, MergedSelection<'doc>>,
    iter_stack: Vec<FilteredSelectionSet<'doc, 'ctx>>,
    subset: &'ctx QuerySubset,
}

pub enum Fragment<'a> {
    Inline(InlineFragment<'a>),
    Named(FragmentDefinition<'a>),
}

impl<'doc, 'ctx> FragmentIter<'doc, 'ctx> {
    pub fn new(root_selection: &'ctx [MergedSelection<'doc>], subset: &'ctx QuerySubset) -> Self {
        FragmentIter {
            root_selection: root_selection.iter(),
            iter_stack: vec![],
            subset,
        }
    }

    fn handle_selection(&mut self, selection: Selection<'doc>) -> Option<Fragment<'doc>> {
        match selection {
            Selection::Field(_) => None,
            Selection::InlineFragment(fragment) => {
                self.iter_stack
                    .push(self.subset.selection_iter(fragment.selection_set()));

                Some(Fragment::Inline(fragment))
            }
            Selection::FragmentSpread(spread) => {
                let fragment = spread.fragment()?;

                self.iter_stack
                    .push(self.subset.selection_iter(fragment.selection_set()));

                Some(Fragment::Named(fragment))
            }
        }
    }
}

impl<'doc, 'ctx> Iterator for FragmentIter<'doc, 'ctx> {
    type Item = Fragment<'doc>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(current_iter) = self.iter_stack.last_mut() {
            let Some(selection) = current_iter.next() else {
                self.iter_stack.pop();
                continue;
            };

            if let Some(fragment) = self.handle_selection(selection) {
                return Some(fragment);
            };
        }

        while let Some(merged_selection) = self.root_selection.next() {
            if let Some(fragment) = self.handle_selection(merged_selection.selection) {
                return Some(fragment);
            };
        }

        None
    }
}

impl<'a> Fragment<'a> {
    pub fn type_condition(&self) -> Option<&'a str> {
        match self {
            Fragment::Inline(fragment) => fragment.type_condition(),
            Fragment::Named(fragment) => Some(fragment.type_condition()),
        }
    }
}
