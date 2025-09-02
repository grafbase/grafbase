use schema::{ListWrapping, MutableWrapping, ScalarType};
use serde::{Deserialize, de::DeserializeSeed};
use walker::Walk;

use super::{
    EnumValueSeed, ListSeed, ScalarTypeSeed, SeedState,
    object::{ConcreteShapeSeed, PolymorphicShapeSeed},
};
use crate::{
    prepare::{FieldShapeRecord, Shape},
    response::{GraphqlError, ResponseValue},
};

#[derive(Clone)]
pub(super) struct FieldSeed<'ctx, 'parent, 'state> {
    pub state: &'state SeedState<'ctx, 'parent>,
    pub field: &'ctx FieldShapeRecord,
    pub wrapping: MutableWrapping,
}

impl<'de> DeserializeSeed<'de> for FieldSeed<'_, '_, '_> {
    type Value = ResponseValue;
    fn deserialize<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let result = if let Some(list_wrapping) = self.wrapping.pop_outermost_list_wrapping() {
            if !self.wrapping.is_list()
                && let Shape::Scalar(ScalarType::Int) = self.field.shape
            {
                let values = Vec::<i32>::deserialize(deserializer)?;
                return Ok(self.state.response.borrow_mut().data.push_int_list(values).into());
            }
            ListSeed {
                state: self.state,
                field: self.field,
                seed: &self,
                is_required: matches!(list_wrapping, ListWrapping::ListNonNull),
                element_is_nullable: self.wrapping.is_nullable(),
            }
            .deserialize(deserializer)
        } else {
            match self.field.shape {
                Shape::Scalar(ty) => ScalarTypeSeed {
                    state: self.state,
                    field: self.field,
                    is_required: self.wrapping.is_required(),
                    ty,
                }
                .deserialize(deserializer),
                Shape::Enum(id) => EnumValueSeed {
                    state: self.state,
                    definition_id: id,
                    field: self.field,
                    is_required: self.wrapping.is_required(),
                }
                .deserialize(deserializer),
                Shape::Concrete(shape_id) => {
                    ConcreteShapeSeed::new(self.state, self.field, self.wrapping.is_required(), shape_id)
                        .deserialize(deserializer)
                }
                Shape::Polymorphic(shape_id) => {
                    PolymorphicShapeSeed::new(self.state, self.field, self.wrapping.is_required(), shape_id)
                        .deserialize(deserializer)
                }
                Shape::DeriveEntity(_) | Shape::DeriveFrom(_) | Shape::DeriveFromScalar => {
                    unreachable!("Should be handled by the ConcreteSeed")
                }
            }
        };

        result.inspect_err(|err| {
            if !self.state.bubbling_up_deser_error.replace(true) && self.state.should_report_error_for(self.field) {
                tracing::error!(
                    "Deserialization failure of subgraph response at path '{}': {err}",
                    self.state.display_path()
                );
                let mut resp = self.state.response.borrow_mut();
                let path = self.state.path();
                resp.propagate_null(&path);
                resp.errors.push(
                    GraphqlError::invalid_subgraph_response()
                        .with_path(path)
                        .with_location(self.field.id.walk(self.state).location()),
                );
            }
        })
    }
}
