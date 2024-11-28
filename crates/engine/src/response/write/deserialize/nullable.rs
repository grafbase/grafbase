use std::fmt;

use serde::{
    de::{DeserializeSeed, Visitor},
    Deserializer,
};
use walker::Walk;

use super::SeedContext;
use crate::{
    operation::DataFieldId,
    response::{ErrorCode, GraphqlError, ResponseValue},
};

pub(super) struct NullableSeed<'ctx, 'parent, Seed> {
    pub ctx: &'parent SeedContext<'ctx>,
    pub field_id: DataFieldId,
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
            Ok(value) => Ok(value.into_nullable()),
            Err(err) => {
                if self.ctx.stop_propagating_and_should_create_new_graphql_error() {
                    self.ctx.writer.push_error(
                        GraphqlError::new(err.to_string(), ErrorCode::SubgraphInvalidResponseError)
                            .with_location(self.field_id.walk(self.ctx).location)
                            .with_path(self.ctx.response_path()),
                    );
                }
                Ok(ResponseValue::Null)
            }
        }
    }
}
