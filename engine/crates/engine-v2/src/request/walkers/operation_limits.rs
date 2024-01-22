use engine_parser::Pos;

use super::{BoundSelectionSetWalker, BoundSelectionWalker};

impl<'a> BoundSelectionSetWalker<'a, ()> {
    // this merely traverses the selection set recursively and merge all cache_config present in the
    // selected fields
    pub(crate) fn max_depth(&self) -> (u16, Pos) {
        (*self)
            .into_iter()
            .map(|selection| match selection {
                BoundSelectionWalker::Field(field) => {
                    let (depth, location) = field
                        .selection_set()
                        .map(|selection_set| selection_set.max_depth())
                        .map(|(value, pos)| (value, pos))
                        .unwrap_or_else(|| (0, field.definition().name_location()));
                    (depth + 1, location)
                }
                BoundSelectionWalker::InlineFragment(inline) => inline.selection_set().max_depth(),
                BoundSelectionWalker::FragmentSpread(spread) => spread.selection_set().max_depth(),
            })
            .max()
            .expect("must be defined")
    }
}
