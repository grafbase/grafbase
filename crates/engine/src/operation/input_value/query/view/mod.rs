mod de;
mod ser;

use schema::{InputValueSet, SchemaInputValueView};

use super::QueryInputValue;

pub(crate) struct QueryInputValueView<'a> {
    pub(super) value: QueryInputValue<'a>,
    pub(super) selection_set: &'a InputValueSet,
}

pub(crate) enum QueryOrSchemaInputValueView<'a> {
    Query(QueryInputValueView<'a>),
    Schema(SchemaInputValueView<'a>),
}
