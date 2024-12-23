mod de;
mod ser;

use schema::{InputValueSet, SchemaInputValueView};

use super::QueryInputValue;

pub struct QueryInputValueView<'a> {
    pub(super) value: QueryInputValue<'a>,
    pub(super) selection_set: &'a InputValueSet,
}

pub enum QueryOrSchemaInputValueView<'a> {
    Query(QueryInputValueView<'a>),
    Schema(SchemaInputValueView<'a>),
}
