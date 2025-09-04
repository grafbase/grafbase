use std::cell::Cell;

use super::SeedState;
use crate::{
    prepare::FieldShapeRecord,
    response::{
        DataPartId, ResponseListId, ResponseValue, ResponseValueId,
        write::deserialize::{
            field::FieldSeed,
            scalar::{NonNullFloatSeed, NonNullIntSeed},
        },
    },
};

/// A trait to abstract over different list element types.
pub(super) trait ListSeedType {
    type Value: 'static;
    type Seed<'a>
    where
        Self: 'a;

    /// Create a ResponseValueId for the element at the given index.
    /// Used to response path errors and make values inaccessible if relevant.
    fn make_response_value_id(&self, index: u32) -> ResponseValueId;
    /// Convert the list of deserialized values into a ResponseValue, possibly updating the response state.
    fn finalize(&self, values: Vec<Self::Value>) -> ResponseValue;
    fn seed(&self) -> Self::Seed<'_>;
}

pub(crate) struct ResponseValueSeedList<'ctx, 'parent, 'state, 'seed> {
    pub seed: &'seed FieldSeed<'ctx, 'parent, 'state>,
    pub id: ResponseListId,
    pub element_is_nullable: bool,
}

impl<'ctx, 'parent, 'state> ListSeedType for ResponseValueSeedList<'ctx, 'parent, 'state, '_> {
    type Value = ResponseValue;
    type Seed<'a>
        = FieldSeed<'ctx, 'parent, 'state>
    where
        Self: 'a;

    fn make_response_value_id(&self, index: u32) -> ResponseValueId {
        ResponseValueId::Index {
            part_id: self.id.part_id,
            list_id: self.id.list_id,
            index,
            nullable: self.element_is_nullable,
        }
    }

    fn finalize(&self, values: Vec<Self::Value>) -> ResponseValue {
        self.seed.state.response.borrow_mut().data.put_list(self.id, values);
        self.id.into()
    }

    fn seed(&self) -> Self::Seed<'_> {
        self.seed.clone()
    }
}

pub(crate) struct NonNullIntSeedList<'ctx, 'parent, 'state> {
    state: &'state SeedState<'ctx, 'parent>,
    part_id: DataPartId,
    field: &'ctx FieldShapeRecord,
    encountered_unexpected_value: Cell<bool>,
}

impl<'ctx, 'parent, 'state> NonNullIntSeedList<'ctx, 'parent, 'state> {
    pub fn new(state: &'state SeedState<'ctx, 'parent>, field: &'ctx FieldShapeRecord) -> Self {
        let part_id = state.response.borrow().data.id;
        Self {
            state,
            part_id,
            field,
            encountered_unexpected_value: Cell::new(false),
        }
    }
}

impl<'ctx, 'parent, 'state> ListSeedType for NonNullIntSeedList<'ctx, 'parent, 'state> {
    type Value = i32;
    type Seed<'a>
        = NonNullIntSeed<'ctx, 'parent, 'state, 'a>
    where
        Self: 'a;

    fn make_response_value_id(&self, index: u32) -> ResponseValueId {
        ResponseValueId::IntListIndex {
            part_id: self.part_id,
            index,
        }
    }

    fn finalize(&self, values: Vec<Self::Value>) -> ResponseValue {
        if self.encountered_unexpected_value.get() {
            ResponseValue::Unexpected
        } else {
            self.state.response.borrow_mut().data.push_int_list(values).into()
        }
    }

    fn seed(&self) -> Self::Seed<'_> {
        NonNullIntSeed {
            state: self.state,
            field: self.field,
            encountered_unexpected_value: &self.encountered_unexpected_value,
        }
    }
}

pub(crate) struct NonNullFloatSeedList<'ctx, 'parent, 'state> {
    state: &'state SeedState<'ctx, 'parent>,
    part_id: DataPartId,
    field: &'ctx FieldShapeRecord,
    encountered_unexpected_value: Cell<bool>,
}

impl<'ctx, 'parent, 'state> NonNullFloatSeedList<'ctx, 'parent, 'state> {
    pub fn new(state: &'state SeedState<'ctx, 'parent>, field: &'ctx FieldShapeRecord) -> Self {
        let part_id = state.response.borrow().data.id;
        Self {
            state,
            part_id,
            field,
            encountered_unexpected_value: Cell::new(false),
        }
    }
}

impl<'ctx, 'parent, 'state> ListSeedType for NonNullFloatSeedList<'ctx, 'parent, 'state> {
    type Value = f64;
    type Seed<'a>
        = NonNullFloatSeed<'ctx, 'parent, 'state, 'a>
    where
        Self: 'a;

    fn make_response_value_id(&self, index: u32) -> ResponseValueId {
        ResponseValueId::FloatListIndex {
            part_id: self.part_id,
            index,
        }
    }

    fn finalize(&self, values: Vec<Self::Value>) -> ResponseValue {
        if self.encountered_unexpected_value.get() {
            ResponseValue::Unexpected
        } else {
            self.state.response.borrow_mut().data.push_float_list(values).into()
        }
    }

    fn seed(&self) -> Self::Seed<'_> {
        NonNullFloatSeed {
            state: self.state,
            field: self.field,
            encountered_unexpected_value: &self.encountered_unexpected_value,
        }
    }
}
