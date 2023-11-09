use serde::Serialize;

use super::AtlasQuery;
use crate::{
    registry::resolvers::atlas_data_api::{input, JsonMap},
    ContextField, Error,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InsertMany {
    documents: Vec<JsonMap>,
}

impl InsertMany {
    pub fn new(ctx: &ContextField<'_>) -> Result<Self, Error> {
        let documents = input::input_many(ctx)?;

        Ok(Self { documents })
    }
}

impl From<InsertMany> for AtlasQuery {
    fn from(value: InsertMany) -> Self {
        Self::InsertMany(value)
    }
}
