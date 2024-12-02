use std::fmt;

use serde::{
    de::{DeserializeSeed, Visitor},
    Deserializer,
};

use super::SeedContext;
use crate::response::{FieldShapeRecord, ResponseValue};

pub(super) struct NullableSeed<'ctx, 'parent, Seed> {
    pub ctx: &'parent SeedContext<'ctx>,
    pub field: &'parent FieldShapeRecord,
    pub seed: Seed,
}

impl<'de, Seed> DeserializeSeed<'de> for NullableSeed<'_, '_, Seed>
where
    Seed: DeserializeSeed<'de, Value = ResponseValue>,
{
    type Value = ResponseValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_option(self)
    }
}

impl<'de, Seed> Visitor<'de> for NullableSeed<'_, '_, Seed>
where
    Seed: DeserializeSeed<'de, Value = ResponseValue>,
{
    type Value = ResponseValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a nullable value")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ResponseValue::Null)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ResponseValue::Null)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        match self.seed.deserialize(deserializer) {
            Ok(value) => Ok(value),
            Err(err) => {
                self.ctx.push_field_serde_error(self.field, false, || err.to_string());
                Ok(ResponseValue::Null)
            }
        }
    }
}
