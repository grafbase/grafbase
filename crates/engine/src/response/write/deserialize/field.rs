use schema::{ListWrapping, MutableWrapping};
use serde::de::DeserializeSeed;
use walker::Walk;

use super::{
    EnumValueSeed, ListSeed, ScalarTypeSeed, SeedContext,
    object::{ConcreteShapeSeed, PolymorphicShapeSeed},
};
use crate::{
    prepare::{FieldShapeRecord, Shape},
    response::{GraphqlError, ResponseValue},
};

#[derive(Clone)]
pub(super) struct FieldSeed<'ctx, 'seed> {
    pub ctx: &'seed SeedContext<'ctx>,
    pub field: &'ctx FieldShapeRecord,
    pub wrapping: MutableWrapping,
}

impl<'de> DeserializeSeed<'de> for FieldSeed<'_, '_> {
    type Value = ResponseValue;
    fn deserialize<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let result = if let Some(list_wrapping) = self.wrapping.pop_outermost_list_wrapping() {
            ListSeed {
                ctx: self.ctx,
                parent_field: self.field,
                seed: &self,
                is_required: matches!(list_wrapping, ListWrapping::RequiredList),
                element_is_nullable: self.wrapping.is_nullable(),
            }
            .deserialize(deserializer)
        } else {
            match self.field.shape {
                Shape::Scalar(ty) => ScalarTypeSeed {
                    ctx: self.ctx,
                    parent_field: self.field,
                    is_required: self.wrapping.is_required(),
                    ty,
                }
                .deserialize(deserializer),
                Shape::Enum(id) => EnumValueSeed {
                    ctx: self.ctx,
                    definition_id: id,
                    parent_field: self.field,
                    is_extra: self.field.key.query_position.is_none(),
                    is_required: self.wrapping.is_required(),
                }
                .deserialize(deserializer),
                Shape::Concrete(shape_id) => {
                    ConcreteShapeSeed::new(self.ctx, self.field, self.wrapping.is_required(), shape_id)
                        .deserialize(deserializer)
                }
                Shape::Polymorphic(shape_id) => {
                    PolymorphicShapeSeed::new(self.ctx, self.field, self.wrapping.is_required(), shape_id)
                        .deserialize(deserializer)
                }
            }
        };

        result.inspect_err(move |err| {
            if !self.ctx.bubbling_up_serde_error.get() && self.field.key.query_position.is_some() {
                self.ctx.bubbling_up_serde_error.set(true);
                tracing::error!(
                    "Deserialization failure of subgraph response at path '{}': {err}",
                    self.ctx.display_path()
                );
                let mut resp = self.ctx.response.borrow_mut();
                resp.propagate_null(&self.ctx.path());
                resp.push_error(
                    GraphqlError::invalid_subgraph_response()
                        .with_path(self.ctx.path().as_ref())
                        .with_location(self.field.id.walk(self.ctx).location()),
                );
            }
        })
    }
}
