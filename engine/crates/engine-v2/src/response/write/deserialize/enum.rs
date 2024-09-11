use schema::EnumDefinitionId;
use serde::{de::DeserializeSeed, Deserialize};
use walker::Walk as _;

use crate::response::ResponseValue;

use super::SeedContext;

pub(crate) struct EnumValueSeed<'parent, 'ctx>(pub &'parent SeedContext<'ctx>, pub EnumDefinitionId);

impl<'de> DeserializeSeed<'de> for EnumValueSeed<'_, '_> {
    type Value = ResponseValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let EnumValueSeed(ctx, enum_id) = self;

        let string_value = std::borrow::Cow::<str>::deserialize(deserializer)?;

        match ctx.schema.walk(enum_id).find_value_by_name(string_value.as_ref()) {
            Some(value) => Ok(ResponseValue::StringId {
                id: value.walk(ctx.schema).name_id,
                nullable: false,
            }),
            None => ctx.propagate_error(),
        }
    }
}
