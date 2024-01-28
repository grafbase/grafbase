use std::sync::atomic::Ordering;

use schema::{ListWrapping, Wrapping};
use serde::de::DeserializeSeed;

use super::{ListSeed, NullableSeed, SeedContextInner, SelectionSetSeed};
use crate::{
    plan::ConcreteType,
    request::BoundAnyFieldDefinitionId,
    response::{GraphqlError, ResponsePath, ResponseValue},
};

pub(super) struct FieldSeed<'ctx, 'parent> {
    pub ctx: &'parent SeedContextInner<'ctx>,
    pub path: ResponsePath,
    pub definition_id: Option<BoundAnyFieldDefinitionId>,
    pub expected_type: &'parent ConcreteType,
    pub wrapping: Wrapping,
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
                ConcreteType::Scalar(scalar_type) => (*scalar_type).deserialize(deserializer).map(Into::into),
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
                ConcreteType::Scalar(scalar_type) => NullableSeed {
                    definition_id: self.definition_id,
                    path: &self.path,
                    ctx: self.ctx,
                    seed: *scalar_type,
                }
                .deserialize(deserializer)
                .map(Into::into),
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
