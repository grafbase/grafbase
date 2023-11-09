use serde::Serialize;
use serde_json::{json, Value};

use super::AtlasQuery;
use crate::{
    registry::resolvers::atlas_data_api::{input, JsonMap},
    ContextField, Error,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMany {
    filter: JsonMap,
    update: JsonMap,
}

impl UpdateMany {
    pub fn new(ctx: &ContextField<'_>) -> Result<Self, Error> {
        let filter = input::filter(ctx)?;
        let update = input::update(ctx)?;

        Ok(Self { filter, update })
    }

    pub fn is_empty(&self) -> bool {
        self.update.is_empty()
    }

    pub fn empty_response(&self) -> Value {
        json!({ "matchedCount": 0, "modifiedCount": 0 })
    }
}

impl From<UpdateMany> for AtlasQuery {
    fn from(value: UpdateMany) -> Self {
        Self::UpdateMany(value)
    }
}
