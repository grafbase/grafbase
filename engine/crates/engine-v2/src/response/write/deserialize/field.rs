use std::sync::atomic::Ordering;

use schema::{ListWrapping, Wrapping};
use serde::de::DeserializeSeed;

use super::{ListSeed, NullableSeed, ScalarTypeSeed, SeedContext, SelectionSetSeed};
use crate::{
    plan::{CollectedField, FieldType},
    response::{GraphqlError, ResponseValue},
};

#[derive(Clone)]
pub(super) struct FieldSeed<'ctx, 'parent> {
    pub ctx: &'parent SeedContext<'ctx>,
    pub field: &'parent CollectedField,
    pub wrapping: Wrapping,
}

impl<'de, 'ctx, 'parent> DeserializeSeed<'de> for FieldSeed<'ctx, 'parent> {
    type Value = ResponseValue;
    fn deserialize<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let result = if let Some(list_wrapping) = self.wrapping.pop_list_wrapping() {
            let list_seed = ListSeed {
                field_id: self.field.id,
                ctx: self.ctx,
                seed: &self,
            };
            match list_wrapping {
                ListWrapping::RequiredList => list_seed.deserialize(deserializer),
                ListWrapping::NullableList => NullableSeed {
                    field_id: self.field.id,
                    ctx: self.ctx,
                    seed: list_seed,
                }
                .deserialize(deserializer),
            }
        } else if self.wrapping.inner_is_required() {
            match &self.field.ty {
                FieldType::Scalar(scalar_type) => ScalarTypeSeed(*scalar_type).deserialize(deserializer),
                FieldType::SelectionSet(collected) => SelectionSetSeed {
                    ctx: self.ctx,
                    collected,
                }
                .deserialize(deserializer),
            }
        } else {
            match &self.field.ty {
                FieldType::Scalar(scalar_type) => NullableSeed {
                    field_id: self.field.id,
                    ctx: self.ctx,
                    seed: ScalarTypeSeed(*scalar_type),
                }
                .deserialize(deserializer),
                FieldType::SelectionSet(collected) => NullableSeed {
                    field_id: self.field.id,
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
                self.ctx.writer.push_error(GraphqlError {
                    message: err.to_string(),
                    locations: vec![self.ctx.plan[self.field.id].location()],
                    path: Some(self.ctx.response_path()),
                    ..Default::default()
                });
            }
            err
        })
    }
}
