use std::collections::HashSet;

use schema::StringId;

use super::{BoundSelectionSetWalker, BoundSelectionWalker};

impl<'a> BoundSelectionSetWalker<'a, ()> {
    pub(crate) fn max_depth(&self) -> u16 {
        (*self)
            .into_iter()
            .map(|selection| match selection {
                BoundSelectionWalker::Field(field) => {
                    field
                        .selection_set()
                        .map(|selection_set| selection_set.max_depth())
                        .unwrap_or_default()
                        + 1
                }
                BoundSelectionWalker::InlineFragment(inline) => inline.selection_set().max_depth(),
                BoundSelectionWalker::FragmentSpread(spread) => spread.selection_set().max_depth(),
            })
            .max()
            .expect("must be defined")
    }

    pub(crate) fn alias_count(&self) -> u16 {
        (*self)
            .into_iter()
            .map(|selection| match selection {
                BoundSelectionWalker::Field(field) => {
                    (field.definition().as_field().expect("must be a field").name() == field.response_key_str()) as u16
                }
                BoundSelectionWalker::InlineFragment(inline) => inline.selection_set().alias_count(),
                BoundSelectionWalker::FragmentSpread(spread) => spread.selection_set().alias_count(),
            })
            .sum()
    }

    pub(crate) fn root_field_count(&self) -> u16 {
        (*self)
            .into_iter()
            .map(|selection| match selection {
                BoundSelectionWalker::Field(_) => 1,
                BoundSelectionWalker::InlineFragment(inline) => inline.selection_set().root_field_count(),
                BoundSelectionWalker::FragmentSpread(spread) => spread.selection_set().root_field_count(),
            })
            .sum()
    }

    pub(crate) fn complexity(&self) -> u16 {
        (*self)
            .into_iter()
            .map(|selection| match selection {
                BoundSelectionWalker::Field(field) => {
                    field
                        .selection_set()
                        .map(|selection_set| selection_set.complexity())
                        .unwrap_or_default()
                        + 1
                }
                BoundSelectionWalker::InlineFragment(inline) => inline.selection_set().complexity(),
                BoundSelectionWalker::FragmentSpread(spread) => spread.selection_set().complexity(),
            })
            .sum()
    }

    pub(crate) fn height(&self, fields_seen: &mut HashSet<StringId>) -> u16 {
        (*self)
            .into_iter()
            .map(|selection| match selection {
                BoundSelectionWalker::Field(field) => {
                    (if fields_seen.insert(field.definition().as_field().expect("must be a field").name_string_id()) {
                        0
                    } else {
                        1
                    }) + field
                        .selection_set()
                        .map(|selection_set| selection_set.height(&mut HashSet::new()))
                        .unwrap_or_default()
                }
                BoundSelectionWalker::InlineFragment(inline) => inline.selection_set().height(fields_seen),
                BoundSelectionWalker::FragmentSpread(spread) => spread.selection_set().height(fields_seen),
            })
            .sum()
    }
}
