use serde::Serialize;

use crate::{
    registry::resolvers::atlas_data_api::{input, JsonMap},
    Context, Error,
};

use super::AtlasQuery;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteMany {
    filter: JsonMap,
}

impl DeleteMany {
    pub fn new(ctx: &Context<'_>) -> Result<Self, Error> {
        let filter = input::filter(ctx)?;

        Ok(Self { filter })
    }
}

impl From<DeleteMany> for AtlasQuery {
    fn from(value: DeleteMany) -> Self {
        Self::DeleteMany(value)
    }
}
