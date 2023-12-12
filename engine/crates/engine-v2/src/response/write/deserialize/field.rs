use std::sync::atomic::Ordering;

use schema::{DataType, ListWrapping, Wrapping};
use serde::de::DeserializeSeed;

use super::{
    BigIntSeed, BooleanSeed, FloatSeed, IntSeed, JSONSeed, ListSeed, NullableSeed, SeedContext, SelectionSetSeed,
    StringSeed,
};
use crate::{
    plan::ConcreteType,
    request::BoundAnyFieldDefinitionId,
    response::{GraphqlError, ResponsePath, ResponseValue},
};

pub(super) struct FieldSeed<'ctx, 'parent> {
    pub ctx: &'parent SeedContext<'ctx>,
    pub path: ResponsePath,
    pub definition_id: Option<BoundAnyFieldDefinitionId>,
    pub expected_type: &'parent ConcreteType,
    pub wrapping: Wrapping,
}

macro_rules! deserialize_nullable_scalar {
    ($field: expr, $scalar: expr, $deserializer: expr) => {
        NullableSeed {
            definition_id: $field.definition_id,
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
                    definition_id: self.definition_id,
                    path: &self.path,
                    ctx: self.ctx,
                    seed_builder: |path: ResponsePath| FieldSeed {
                        ctx: self.ctx,
                        path,
                        definition_id: self.definition_id,
                        expected_type: self.expected_type,
                        wrapping: self.wrapping.clone(),
                    },
                }
                .deserialize(deserializer),
                ListWrapping::NullableList => NullableSeed {
                    definition_id: self.definition_id,
                    path: &self.path,
                    ctx: self.ctx,
                    seed: ListSeed {
                        definition_id: self.definition_id,
                        path: &self.path,
                        ctx: self.ctx,
                        seed_builder: |path: ResponsePath| FieldSeed {
                            ctx: self.ctx,
                            path,
                            definition_id: self.definition_id,
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
                    definition_id: self.definition_id,
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
                        .definition_id
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
