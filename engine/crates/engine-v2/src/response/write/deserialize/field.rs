use std::sync::atomic::Ordering;

use schema::{DataType, ListWrapping, Wrapping};
use serde::de::DeserializeSeed;

use super::{
    BigIntSeed, BooleanSeed, FloatSeed, IntSeed, JSONSeed, ListSeed, NullableSeed, SeedContextInner, SelectionSetSeed,
    StringSeed,
};
use crate::{
    plan::ConcreteType,
    request::BoundFieldId,
    response::{GraphqlError, ResponsePath, ResponseValue},
};

pub(super) struct FieldSeed<'ctx, 'parent> {
    pub ctx: &'parent SeedContextInner<'ctx>,
    pub path: ResponsePath,
    pub bound_field_id: Option<BoundFieldId>,
    pub expected_type: &'parent ConcreteType,
    pub wrapping: Wrapping,
}

macro_rules! deserialize_nullable_scalar {
    ($field: expr, $scalar: expr, $deserializer: expr) => {
        NullableSeed {
            bound_field_id: $field.bound_field_id,
            path: &$field.path,
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
        let result = if let Some(list_wrapping) = self.wrapping.list_wrapping.pop() {
            match list_wrapping {
                ListWrapping::RequiredList => ListSeed {
                    bound_field_id: self.bound_field_id,
                    path: &self.path,
                    ctx: self.ctx,
                    seed_builder: |path: ResponsePath| FieldSeed {
                        ctx: self.ctx,
                        path,
                        bound_field_id: self.bound_field_id,
                        expected_type: self.expected_type,
                        wrapping: self.wrapping.clone(),
                    },
                }
                .deserialize(deserializer),
                ListWrapping::NullableList => NullableSeed {
                    bound_field_id: self.bound_field_id,
                    path: &self.path,
                    ctx: self.ctx,
                    seed: ListSeed {
                        bound_field_id: self.bound_field_id,
                        path: &self.path,
                        ctx: self.ctx,
                        seed_builder: |path: ResponsePath| FieldSeed {
                            ctx: self.ctx,
                            path,
                            bound_field_id: self.bound_field_id,
                            expected_type: self.expected_type,
                            wrapping: self.wrapping.clone(),
                        },
                    },
                }
                .deserialize(deserializer),
            }
        } else if self.wrapping.inner_is_required {
            match self.expected_type {
                ConcreteType::Scalar(data_type) => match data_type {
                    DataType::String => StringSeed.deserialize(deserializer),
                    DataType::Float => FloatSeed.deserialize(deserializer),
                    DataType::Int => IntSeed.deserialize(deserializer),
                    DataType::BigInt => BigIntSeed.deserialize(deserializer),
                    DataType::JSON => JSONSeed.deserialize(deserializer),
                    DataType::Boolean => BooleanSeed.deserialize(deserializer),
                },
                ConcreteType::SelectionSet(expected) => SelectionSetSeed {
                    ctx: self.ctx,
                    path: &self.path,
                    expected,
                }
                .deserialize(deserializer),
                ConcreteType::ExtraSelectionSet(_) => todo!(),
            }
        } else {
            match self.expected_type {
                ConcreteType::Scalar(data_type) => match data_type {
                    DataType::String => deserialize_nullable_scalar!(self, StringSeed, deserializer),
                    DataType::Float => deserialize_nullable_scalar!(self, FloatSeed, deserializer),
                    DataType::Int => deserialize_nullable_scalar!(self, IntSeed, deserializer),
                    DataType::BigInt => deserialize_nullable_scalar!(self, BigIntSeed, deserializer),
                    DataType::JSON => deserialize_nullable_scalar!(self, JSONSeed, deserializer),
                    DataType::Boolean => deserialize_nullable_scalar!(self, BooleanSeed, deserializer),
                },
                ConcreteType::SelectionSet(expected) => NullableSeed {
                    bound_field_id: self.bound_field_id,
                    path: &self.path,
                    ctx: self.ctx,
                    seed: SelectionSetSeed {
                        ctx: self.ctx,
                        path: &self.path,
                        expected,
                    },
                }
                .deserialize(deserializer),
                ConcreteType::ExtraSelectionSet(_) => todo!(),
            }
        };
        result.map_err(move |err| {
            if !self.ctx.propagating_error.fetch_or(true, Ordering::Relaxed) {
                self.ctx.data.borrow_mut().push_error(GraphqlError {
                    message: err.to_string(),
                    locations: self
                        .bound_field_id
                        .map(|id| self.ctx.walker.walk(id).name_location())
                        .into_iter()
                        .collect(),
                    path: Some(self.path.clone()),
                    ..Default::default()
                });
            }
            err
        })
    }
}
