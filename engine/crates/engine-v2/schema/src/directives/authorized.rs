use crate::{InputValueSet, RequiredFieldSetId, SchemaInputValueId};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AuthorizedDirective {
    pub arguments: InputValueSet,
    pub fields: Option<RequiredFieldSetId>,
    pub node: Option<RequiredFieldSetId>,
    pub metadata: Option<SchemaInputValueId>,
}
