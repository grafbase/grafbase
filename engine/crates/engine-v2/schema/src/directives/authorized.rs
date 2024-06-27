use crate::{InputValueSet, RequiredFieldSetId, StringId};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AuthorizedDirective {
    pub rule: StringId,
    pub arguments: InputValueSet,
    pub fields: Option<RequiredFieldSetId>,
    pub metadata: (),
}
