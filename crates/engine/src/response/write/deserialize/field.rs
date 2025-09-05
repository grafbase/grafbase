use schema::{ListWrapping, MutableWrapping, ScalarType, Wrapping};
use serde::de::DeserializeSeed;
use walker::Walk;

use super::{
    EnumValueSeed, ListSeed, ScalarTypeSeed, SeedState,
    object::{ConcreteShapeSeed, PolymorphicShapeSeed},
};
use crate::{
    prepare::{FieldShapeRecord, Shape},
    response::{
        GraphqlError, ResponseValue,
        write::deserialize::list::{NonNullFloatSeedList, NonNullIntSeedList, ResponseValueSeedList},
    },
};

#[derive(Clone)]
pub(super) struct FieldSeed<'ctx, 'parent, 'state> {
    pub state: &'state SeedState<'ctx, 'parent>,
    pub field: &'ctx FieldShapeRecord,
    pub wrapping: MutableWrapping,
}

const REQUIRED: Wrapping = Wrapping::new().non_null();

impl<'de> DeserializeSeed<'de> for FieldSeed<'_, '_, '_> {
    type Value = ResponseValue;
    fn deserialize<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let result = if let Some(list_wrapping) = self.wrapping.pop_outermost_list_wrapping() {
            // We have specialized handling of [Int!] and [Float!] as we can store them efficiently
            // as Vec<i32> and Vec<f64> and are somewhat common enough.
            match self.field.shape {
                Shape::Scalar(ScalarType::Int) if self.wrapping == REQUIRED => ListSeed {
                    state: self.state,
                    field: self.field,
                    list_type: &NonNullIntSeedList::new(self.state, self.field),
                    is_required: matches!(list_wrapping, ListWrapping::ListNonNull),
                }
                .deserialize(deserializer),
                Shape::Scalar(ScalarType::Float) if self.wrapping == REQUIRED => ListSeed {
                    state: self.state,
                    field: self.field,
                    list_type: &NonNullFloatSeedList::new(self.state, self.field),
                    is_required: matches!(list_wrapping, ListWrapping::ListNonNull),
                }
                .deserialize(deserializer),
                _ => {
                    let id = self.state.response.borrow_mut().data.reserve_list_id();
                    ListSeed {
                        state: self.state,
                        field: self.field,
                        list_type: &ResponseValueSeedList {
                            seed: &self,
                            id,
                            element_is_nullable: self.wrapping.is_nullable(),
                        },
                        is_required: matches!(list_wrapping, ListWrapping::ListNonNull),
                    }
                    .deserialize(deserializer)
                }
            }
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
