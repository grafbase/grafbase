use std::{fmt, sync::atomic::Ordering};

use serde::{
    de::{DeserializeSeed, Visitor},
    Deserializer,
};

use super::SeedContextInner;
use crate::{
    request::BoundFieldId,
    response::{GraphqlError, ResponsePath, ResponseValue},
};

pub(super) struct NullableSeed<'ctx, 'parent, Seed> {
    pub path: &'parent ResponsePath,
    pub bound_field_id: BoundFieldId,
    pub ctx: &'parent SeedContextInner<'ctx>,
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
                    self.ctx.response_part.borrow_mut().push_error(GraphqlError {
                        message: err.to_string(),
                        locations: self
                            .ctx
                            .plan
                            .bound_walk_with(self.bound_field_id, ())
                            .name_location()
                            .into_iter()
                            .collect(),
                        path: Some(self.path.clone()),
                        ..Default::default()
                    });
                }
                Ok(ResponseValue::Null)
            }
        }
    }
}
