#[derive(Debug, Clone)]
pub enum Data {
    Json(Vec<u8>),
    Cbor(Vec<u8>),
}

pub type FieldId = u16;
pub type FieldIdRange = (FieldId, FieldId);
pub type ArgumentsId = u16;

#[repr(C)]
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct SelectionSet {
    pub requires_typename: bool,
    pub fields_ordered_by_parent_entity: FieldIdRange,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Field {
    pub alias: Option<String>,
    pub definition_id: super::DefinitionId,
    pub arguments: Option<ArgumentsId>,
    pub selection_set: Option<SelectionSet>,
}

#[derive(Debug, Clone)]
pub struct Response {
    pub data: Option<Data>,
    pub errors: Vec<super::Error>,
}

#[derive(Debug, Clone)]
pub enum SubscriptionItem {
    Single(Response),
    Multiple(Vec<Response>),
}
