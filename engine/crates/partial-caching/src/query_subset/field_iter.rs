use cynic_parser::executable::{FieldSelection, Selection};

use crate::{query_subset::FilteredSelectionSet, QuerySubset};

/// An iterator over the fields of a selection set.
///
/// This will recurse into any selection sets nested inside fragments.
pub struct FieldIter<'a> {
    iter_stack: Vec<FilteredSelectionSet<'a, 'a>>,
    subset: &'a QuerySubset,
}

impl<'a> FieldIter<'a> {
    pub fn new(selection_set: FilteredSelectionSet<'a, 'a>, subset: &'a QuerySubset) -> Self {
        FieldIter {
            iter_stack: vec![selection_set],
            subset,
        }
    }
}

impl<'a> Iterator for FieldIter<'a> {
    type Item = FieldSelection<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(current_iter) = self.iter_stack.last_mut() {
            let Some(selection) = current_iter.next() else {
                self.iter_stack.pop();
                continue;
            };

            match selection {
                Selection::Field(field) => return Some(field),
                Selection::InlineFragment(fragment) => {
                    self.iter_stack
                        .push(self.subset.selection_iter(fragment.selection_set()));
                }
                Selection::FragmentSpread(spread) => {
                    let Some(fragment) = spread.fragment() else { continue };

                    self.iter_stack
                        .push(self.subset.selection_iter(fragment.selection_set()));
                }
            }
        }

        None
    }
}
