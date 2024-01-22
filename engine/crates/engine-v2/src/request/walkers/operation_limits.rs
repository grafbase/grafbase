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
}
