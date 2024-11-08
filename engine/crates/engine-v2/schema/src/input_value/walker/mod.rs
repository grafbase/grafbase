mod de;
mod debug;
mod display;
mod ord;
mod ser;
mod view;

use crate::Schema;

use super::{InputValueSet, SchemaInputValueRecord};
pub use view::*;

#[derive(Copy, Clone)]
pub struct SchemaInputValue<'a> {
    pub(super) schema: &'a Schema,
    pub(super) ref_: &'a SchemaInputValueRecord,
}

impl<'a> SchemaInputValue<'a> {
    pub fn with_selection_set<'s, 'w>(self, selection_set: &'s InputValueSet) -> SchemaInputValueView<'w>
    where
        'a: 'w,
        's: 'w,
    {
        SchemaInputValueView {
            value: self,
            selection_set,
        }
    }
}
