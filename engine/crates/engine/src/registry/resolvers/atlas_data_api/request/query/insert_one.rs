use serde::Serialize;

use super::AtlasQuery;
use crate::{
    registry::resolvers::atlas_data_api::{input, JsonMap},
    ContextField, Error,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InsertOne {
    document: JsonMap,
}

impl InsertOne {
    pub fn new(ctx: &ContextField<'_>) -> Result<Self, Error> {
        let document = input::input(ctx)?;

        Ok(Self { document })
    }
}

impl From<InsertOne> for AtlasQuery {
    fn from(value: InsertOne) -> Self {
        Self::InsertOne(value)
    }
}
