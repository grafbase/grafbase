use id_derives::{Id, IndexImpls};
use id_newtypes::IdRange;
use schema::{RequiredFieldId, StringId};

#[derive(Default, IndexImpls)]
pub(crate) struct ResponseViews {
    #[indexed_by(ResponseViewSelectionId)]
    pub selections: Vec<ResponseViewSelection>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, Id)]
pub struct ResponseViewSelectionId(std::num::NonZero<u16>);

pub(crate) type ResponseViewSelectionSet = IdRange<ResponseViewSelectionId>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResponseViewSelection {
    pub name: StringId,
    pub id: RequiredFieldId,
    pub subselection: ResponseViewSelectionSet,
}
