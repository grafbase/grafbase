use schema::EnumDefinitionId;
use serde::{de::DeserializeSeed, de::Error, Deserialize};
use walker::Walk;

use crate::response::ResponseValue;

use super::SeedContext;

pub(crate) struct EnumValueSeed<'parent, 'ctx> {
    pub ctx: &'parent SeedContext<'ctx>,
    pub id: EnumDefinitionId,
    pub is_extra: bool,
}

impl<'de> DeserializeSeed<'de> for EnumValueSeed<'_, '_> {
    type Value = ResponseValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let EnumValueSeed { ctx, id, is_extra } = self;

        let string_value = std::borrow::Cow::<str>::deserialize(deserializer)?;

        tracing::debug!("EnumDefinition {:#?}", id.walk(ctx.schema));
        match id.walk(ctx.schema).find_value_by_name(string_value.as_ref()) {
            // If inaccessible propagating an error without any message.
            Some(value) if !is_extra && value.is_inaccessible() => ctx.propagate_error(),
            Some(value) => Ok(ResponseValue::StringId {
                id: value.name_id,
                nullable: false,
            }),
            None => Err(D::Error::custom(format!("Unknown enum value: {string_value}"))),
        }
    }
}
