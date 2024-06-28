use crate::{InputValueSet, RequiredFieldSetId};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AuthorizedDirective {
    pub arguments: InputValueSet,
    pub fields: Option<RequiredFieldSetId>,
    pub metadata: (),
}
