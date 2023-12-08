use std::{collections::HashMap, fmt, sync::atomic::Ordering};

use serde::{
    de::{DeserializeSeed, Visitor},
    Deserializer,
};

use super::SeedContext;
use crate::{
    request::BoundAnyFieldDefinitionId,
    response::{GraphqlError, ResponsePath, ResponseValue},
};

pub(super) struct NullableSeed<'ctx, 'parent, Seed> {
    pub path: &'parent ResponsePath,
    pub definition_id: BoundAnyFieldDefinitionId,
    pub ctx: &'parent SeedContext<'ctx>,
    pub seed: Seed,
}

impl<'de, 'ctx, 'parent, Seed> DeserializeSeed<'de> for NullableSeed<'ctx, 'parent, Seed>
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

impl<'de, 'ctx, 'parent, Seed> Visitor<'de> for NullableSeed<'ctx, 'parent, Seed>
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
                if !self.ctx.propagating_error.fetch_and(false, Ordering::Relaxed) {
                    self.ctx.data.borrow_mut().push_error(GraphqlError {
                        message: err.to_string(),
                        locations: vec![self.ctx.walker.walk(self.definition_id).name_location()],
                        path: Some(self.path.clone()),
                        extensions: HashMap::with_capacity(0),
                    });
                }
                Ok(ResponseValue::Null)
            }
        }
    }
}
