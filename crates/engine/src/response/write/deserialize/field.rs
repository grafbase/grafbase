use schema::{ListWrapping, MutableWrapping};
use serde::de::DeserializeSeed;

use super::{
    object::{ConcreteShapeSeed, PolymorphicShapeSeed},
    EnumValueSeed, ListSeed, NullableSeed, ScalarTypeSeed, SeedContext,
};
use crate::response::{FieldShapeRecord, ResponseValue, Shape};

#[derive(Clone)]
pub(super) struct FieldSeed<'ctx, 'parent> {
    pub ctx: &'parent SeedContext<'ctx>,
    pub field: &'parent FieldShapeRecord,
    pub wrapping: MutableWrapping,
}

impl<'de> DeserializeSeed<'de> for FieldSeed<'_, '_> {
    type Value = ResponseValue;
    fn deserialize<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let result = if let Some(list_wrapping) = self.wrapping.pop_outermost_list_wrapping() {
            let list_seed = ListSeed {
                ctx: self.ctx,
                field: self.field,
                seed: &self,
                element_is_nullable: self.wrapping.is_nullable(),
            };
            match list_wrapping {
                ListWrapping::RequiredList => list_seed.deserialize(deserializer),
                ListWrapping::NullableList => NullableSeed {
                    ctx: self.ctx,
                    field: self.field,
                    seed: list_seed,
                }
                .deserialize(deserializer),
            }
        } else if self.wrapping.is_required() {
            match self.field.shape {
                Shape::Scalar(ty) => ScalarTypeSeed(ty).deserialize(deserializer),
                Shape::Enum(id) => EnumValueSeed {
                    ctx: self.ctx,
                    id,
                    is_extra: self.field.key.query_position.is_none(),
                    is_nullable: false,
                }
                .deserialize(deserializer),
                Shape::Concrete(shape_id) => ConcreteShapeSeed::new(self.ctx, shape_id).deserialize(deserializer),
                Shape::Polymorphic(shape_id) => PolymorphicShapeSeed::new(self.ctx, shape_id).deserialize(deserializer),
            }
        } else {
            match self.field.shape {
                Shape::Scalar(ty) => NullableSeed {
                    ctx: self.ctx,
                    field: self.field,
                    seed: ScalarTypeSeed(ty),
                }
                .deserialize(deserializer),
                Shape::Enum(enum_definition_id) => NullableSeed {
                    ctx: self.ctx,
                    field: self.field,
                    seed: EnumValueSeed {
                        ctx: self.ctx,
                        id: enum_definition_id,
                        is_extra: self.field.key.query_position.is_none(),
                        is_nullable: true,
                    },
                }
                .deserialize(deserializer),
                Shape::Concrete(shape_id) => NullableSeed {
                    ctx: self.ctx,
                    field: self.field,
                    seed: ConcreteShapeSeed::new(self.ctx, shape_id),
                }
                .deserialize(deserializer),
                Shape::Polymorphic(shape_id) => NullableSeed {
                    ctx: self.ctx,
                    field: self.field,
                    seed: PolymorphicShapeSeed::new(self.ctx, shape_id),
                }
                .deserialize(deserializer),
            }
        };

        result.inspect_err(move |err| {
            self.ctx
                .push_field_deserialization_error_if_not_bubbling_up(self.field, true, err);
        })
    }
}
