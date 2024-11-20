use schema::{ListWrapping, Wrapping};
use serde::de::DeserializeSeed;
use walker::Walk;

use super::{
    object::{ConcreteObjectSeed, PolymorphicObjectSeed},
    EnumValueSeed, ListSeed, NullableSeed, ScalarTypeSeed, SeedContext,
};
use crate::response::{ErrorCode, FieldShapeRecord, GraphqlError, ResponseValue, Shape};

#[derive(Clone)]
pub(super) struct FieldSeed<'ctx, 'parent> {
    pub ctx: &'parent SeedContext<'ctx>,
    pub field: &'parent FieldShapeRecord,
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
                ctx: self.ctx,
                field_id: self.field.id,
                seed: &self,
            };
            match list_wrapping {
                ListWrapping::RequiredList => list_seed.deserialize(deserializer),
                ListWrapping::NullableList => NullableSeed {
                    ctx: self.ctx,
                    field_id: self.field.id,
                    seed: list_seed,
                }
                .deserialize(deserializer),
            }
        } else if self.wrapping.inner_is_required() {
            match self.field.shape {
                Shape::Scalar(ty) => ScalarTypeSeed(ty).deserialize(deserializer),
                Shape::Enum(id) => EnumValueSeed {
                    ctx: self.ctx,
                    id,
                    is_extra: self.field.key.query_position.is_none(),
                }
                .deserialize(deserializer),
                Shape::ConcreteObject(shape_id) => {
                    ConcreteObjectSeed::new(self.ctx, shape_id).deserialize(deserializer)
                }
                Shape::PolymorphicObject(shape_id) => {
                    PolymorphicObjectSeed::new(self.ctx, shape_id).deserialize(deserializer)
                }
            }
        } else {
            match self.field.shape {
                Shape::Scalar(ty) => NullableSeed {
                    ctx: self.ctx,
                    field_id: self.field.id,
                    seed: ScalarTypeSeed(ty),
                }
                .deserialize(deserializer),
                Shape::Enum(enum_definition_id) => NullableSeed {
                    ctx: self.ctx,
                    field_id: self.field.id,
                    seed: EnumValueSeed {
                        ctx: self.ctx,
                        id: enum_definition_id,
                        is_extra: self.field.key.query_position.is_none(),
                    },
                }
                .deserialize(deserializer),
                Shape::ConcreteObject(shape_id) => NullableSeed {
                    ctx: self.ctx,
                    field_id: self.field.id,
                    seed: ConcreteObjectSeed::new(self.ctx, shape_id),
                }
                .deserialize(deserializer),
                Shape::PolymorphicObject(shape_id) => NullableSeed {
                    ctx: self.ctx,
                    field_id: self.field.id,
                    seed: PolymorphicObjectSeed::new(self.ctx, shape_id),
                }
                .deserialize(deserializer),
            }
        };

        result.inspect_err(move |err| {
            if self.ctx.should_create_new_graphql_error() {
                self.ctx.writer.push_error(
                    GraphqlError::new(err.to_string(), ErrorCode::SubgraphInvalidResponseError)
                        .with_location(self.field.id.walk(self.ctx).location)
                        .with_path(self.ctx.response_path()),
                );
            }
        })
    }
}
