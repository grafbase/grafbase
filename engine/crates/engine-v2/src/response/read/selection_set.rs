use id_newtypes::IdRange;
use schema::{RequiredFieldId, StringId};

#[derive(Default)]
pub(crate) struct ResponseViews {
    pub selections: Vec<ResponseViewSelection>,
}

id_newtypes::NonZeroU16! {
    ResponseViews.selections[ResponseViewSelectionId] => ResponseViewSelection,
}

pub(crate) type ResponseViewSelectionSet = IdRange<ResponseViewSelectionId>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResponseViewSelection {
    pub name: StringId,
    pub id: RequiredFieldId,
    pub subselection: ResponseViewSelectionSet,
}
