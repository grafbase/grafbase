mod de;
mod debug;
mod display;
mod ord;
mod ser;

use crate::Schema;

use super::SchemaInputValueRecord;

#[derive(Copy, Clone)]
pub struct SchemaInputValue<'a> {
    pub(super) schema: &'a Schema,
    pub(super) value: &'a SchemaInputValueRecord,
}
