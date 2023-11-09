use serde::Serialize;

use super::AtlasQuery;
use crate::{
    registry::{
        resolvers::{
            atlas_data_api::{input, projection, JsonMap},
            ResolvedValue, ResolverContext,
        },
        type_kinds::SelectionSetTarget,
    },
    ContextField, Error,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FindOne {
    filter: JsonMap,
    projection: JsonMap,
}

impl FindOne {
    pub fn new(ctx: &ContextField<'_>, resolver_ctx: &ResolverContext<'_>) -> Result<Self, Error> {
        let selection_set: SelectionSetTarget<'_> = resolver_ctx.ty.try_into().unwrap();
        let available_fields = selection_set.field_map().unwrap();
        let selection = ctx.look_ahead().selection_fields();

        let projection = projection::project(ctx, selection.into_iter(), available_fields)?;
        let filter = input::by(ctx)?;

        Ok(Self { filter, projection })
    }

    pub fn convert_result(&self, result: &mut serde_json::Value) -> ResolvedValue {
        let value = result
            .as_object_mut()
            .and_then(|object| object.remove("document"))
            .unwrap_or(serde_json::Value::Null);

        ResolvedValue::new(value)
    }
}

impl From<FindOne> for AtlasQuery {
    fn from(value: FindOne) -> Self {
        Self::FindOne(value)
    }
}
