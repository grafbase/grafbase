use std::sync::atomic::Ordering;

use schema::{DataType, ListWrapping, Wrapping};
use serde::de::DeserializeSeed;

use super::{
    BigIntSeed, BooleanSeed, FloatSeed, IntSeed, JSONSeed, ListSeed, NullableSeed, SeedContextInner, SelectionSetSeed,
    StringSeed,
};
use crate::{
    plan::{CollectedField, FieldType},
    response::{GraphqlError, ResponseValue},
};

#[derive(Clone)]
pub(super) struct FieldSeed<'ctx, 'parent> {
    pub ctx: &'parent SeedContextInner<'ctx>,
    pub field: &'parent CollectedField,
    pub wrapping: Wrapping,
}

macro_rules! deserialize_nullable_scalar {
    ($field: expr, $scalar: expr, $deserializer: expr) => {
        NullableSeed {
            bound_field_id: $field.field.bound_field_id,
            ctx: $field.ctx,
            seed: $scalar,
        }
        .deserialize($deserializer)
    };
}

impl<'de, 'ctx, 'parent> DeserializeSeed<'de> for FieldSeed<'ctx, 'parent> {
    type Value = ResponseValue;
    fn deserialize<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let result = if let Some(list_wrapping) = self.wrapping.pop_list_wrapping() {
            let list_seed = ListSeed {
                bound_field_id: self.field.bound_field_id,
                ctx: self.ctx,
                seed: &self,
            };
            match list_wrapping {
                ListWrapping::RequiredList => list_seed.deserialize(deserializer),
                ListWrapping::NullableList => NullableSeed {
                    bound_field_id: self.field.bound_field_id,
                    ctx: self.ctx,
                    seed: list_seed,
                }
                .deserialize(deserializer),
            }
        } else if self.wrapping.inner_is_required() {
            match &self.field.ty {
                FieldType::Scalar(data_type) => match data_type {
                    DataType::String => StringSeed.deserialize(deserializer),
                    DataType::Float => FloatSeed.deserialize(deserializer),
                    DataType::Int => IntSeed.deserialize(deserializer),
                    DataType::BigInt => BigIntSeed.deserialize(deserializer),
                    DataType::JSON => JSONSeed.deserialize(deserializer),
                    DataType::Boolean => BooleanSeed.deserialize(deserializer),
                },
                FieldType::SelectionSet(collected) => SelectionSetSeed {
                    ctx: self.ctx,
                    collected,
                }
                .deserialize(deserializer),
            }
        } else {
            match &self.field.ty {
                FieldType::Scalar(data_type) => match data_type {
                    DataType::String => deserialize_nullable_scalar!(self, StringSeed, deserializer),
                    DataType::Float => deserialize_nullable_scalar!(self, FloatSeed, deserializer),
                    DataType::Int => deserialize_nullable_scalar!(self, IntSeed, deserializer),
                    DataType::BigInt => deserialize_nullable_scalar!(self, BigIntSeed, deserializer),
                    DataType::JSON => deserialize_nullable_scalar!(self, JSONSeed, deserializer),
                    DataType::Boolean => deserialize_nullable_scalar!(self, BooleanSeed, deserializer),
                },
                FieldType::SelectionSet(collected) => NullableSeed {
                    bound_field_id: self.field.bound_field_id,
                    ctx: self.ctx,
                    seed: SelectionSetSeed {
                        ctx: self.ctx,
                        collected,
                    },
                }
                .deserialize(deserializer),
            }
        };

        result.map_err(move |err| {
            if !self.ctx.propagating_error.fetch_or(true, Ordering::Relaxed) {
                self.ctx.response_part.borrow_mut().push_error(GraphqlError {
                    message: err.to_string(),
                    locations: self
                        .ctx
                        .plan
                        .bound_walk_with(self.field.bound_field_id, ())
                        .name_location()
                        .into_iter()
                        .collect(),
                    path: Some(self.ctx.response_path()),
                    ..Default::default()
                });
            }
            err
        })
    }
}
