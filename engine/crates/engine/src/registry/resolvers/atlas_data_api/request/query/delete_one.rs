use serde::Serialize;

use super::AtlasQuery;
use crate::{
    registry::resolvers::atlas_data_api::{input, JsonMap},
    ContextField, Error,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteOne {
    filter: JsonMap,
}

impl DeleteOne {
    pub fn new(ctx: &ContextField<'_>) -> Result<Self, Error> {
        let filter = input::by(ctx)?;

        Ok(Self { filter })
    }
}

impl From<DeleteOne> for AtlasQuery {
    fn from(value: DeleteOne) -> Self {
        Self::DeleteOne(value)
    }
}
