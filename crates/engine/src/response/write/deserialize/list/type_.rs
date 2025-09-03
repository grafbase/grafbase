use std::cell::Cell;

use super::SeedState;
use crate::response::{DataPartId, ResponseListId, ResponseValue, ResponseValueId};

/// A trait to abstract over different list element types.
pub(super) trait ListSeedType {
    type DeserializedValue;
    type Value;
    /// Create a ResponseValueId for the element at the given index.
    /// Used to response path errors and make values inaccessible if relevant.
    fn make_response_value_id(&self, index: u32) -> ResponseValueId;
    /// Handle a deserialized value. For ResponseValue it's just pushing to the values, but for f64
    /// and i32 we can't push a 'null' to the list, so we have to keep track that this list
    /// deserializer should return a ResponseValue::Unexpected.
    fn handle_deserialize_value(&self, values: &mut Vec<Self::Value>, value: Self::DeserializedValue);
    /// Convert the list of deserialized values into a ResponseValue, possibly updating the response state.
    fn into_response_value(self, state: &SeedState<'_, '_>, values: Vec<Self::Value>) -> ResponseValue;
}

pub(crate) struct ResponseValueSeedList {
    pub id: ResponseListId,
    pub element_is_nullable: bool,
}

impl ListSeedType for ResponseValueSeedList {
    type DeserializedValue = ResponseValue;
    type Value = ResponseValue;
    fn make_response_value_id(&self, index: u32) -> ResponseValueId {
        ResponseValueId::Index {
            part_id: self.id.part_id,
            list_id: self.id.list_id,
            index,
            nullable: self.element_is_nullable,
        }
    }

    fn handle_deserialize_value(&self, values: &mut Vec<Self::Value>, value: Self::DeserializedValue) {
        values.push(value);
    }

    fn into_response_value(self, state: &SeedState<'_, '_>, values: Vec<Self::Value>) -> ResponseValue {
        state.response.borrow_mut().data.put_list(self.id, values);
        self.id.into()
    }
}

pub(crate) struct NonNullScalarSeedList<T> {
    part_id: DataPartId,
    encountered_unexpected_value: Cell<bool>,
    _marker: std::marker::PhantomData<T>,
}

impl<T> NonNullScalarSeedList<T> {
    pub fn new(part_id: DataPartId) -> Self {
        Self {
            part_id,
            encountered_unexpected_value: Cell::new(false),
            _marker: std::marker::PhantomData,
        }
    }
}

impl ListSeedType for NonNullScalarSeedList<i32> {
    type DeserializedValue = Result<i32, ()>;
    type Value = i32;
    fn make_response_value_id(&self, index: u32) -> ResponseValueId {
        ResponseValueId::IntListIndex {
            part_id: self.part_id,
            index,
        }
    }

    fn handle_deserialize_value(&self, values: &mut Vec<Self::Value>, value: Self::DeserializedValue) {
        match value {
            Ok(v) => values.push(v),
            Err(_) => self.encountered_unexpected_value.set(true),
        }
    }

    fn into_response_value(self, state: &SeedState<'_, '_>, values: Vec<Self::Value>) -> ResponseValue {
        if self.encountered_unexpected_value.get() {
            ResponseValue::Unexpected
        } else {
            state.response.borrow_mut().data.push_int_list(values).into()
        }
    }
}

impl ListSeedType for NonNullScalarSeedList<f64> {
    type DeserializedValue = Result<f64, ()>;
    type Value = f64;
    fn make_response_value_id(&self, index: u32) -> ResponseValueId {
        ResponseValueId::FloatListIndex {
            part_id: self.part_id,
            index,
        }
    }

    fn handle_deserialize_value(&self, values: &mut Vec<Self::Value>, value: Self::DeserializedValue) {
        match value {
            Ok(v) => values.push(v),
            Err(_) => self.encountered_unexpected_value.set(true),
        }
    }

    fn into_response_value(self, state: &SeedState<'_, '_>, values: Vec<Self::Value>) -> ResponseValue {
        if self.encountered_unexpected_value.get() {
            ResponseValue::Unexpected
        } else {
            state.response.borrow_mut().data.push_float_list(values).into()
        }
    }
}
