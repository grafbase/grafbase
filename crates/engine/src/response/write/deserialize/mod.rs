mod r#enum;
mod error;
mod field;
mod key;
mod list;
mod object;
mod root;
mod scalar;
mod state;

use bytes::Bytes;
use object::{ConcreteShapeFieldsSeed, ObjectFields};
use runtime::extension::Data;
use serde::de::DeserializeSeed;

use crate::response::{GraphqlError, ResponseObjectRef};

use self::r#enum::*;
pub(crate) use key::*;
use list::ListSeed;
use scalar::*;
pub(crate) use state::*;

pub(crate) enum Deserializable<'a> {
    JsonValue(serde_json::Value),
    Json(&'a Bytes),
    JsonWithRawValues(&'a Bytes),
    Cbor(&'a Bytes),
}

impl<'de> From<&'de Data> for Deserializable<'de> {
    fn from(data: &'de Data) -> Self {
        match data {
            Data::Json(bytes) => Deserializable::Json(bytes),
            Data::Cbor(bytes) => Deserializable::Cbor(bytes),
        }
    }
}

impl From<serde_json::Value> for Deserializable<'_> {
    fn from(value: serde_json::Value) -> Self {
        Deserializable::JsonValue(value)
    }
}

impl<'parent> SeedState<'_, 'parent> {
    pub fn deserialize_data_with<'de, Seed: DeserializeSeed<'de>>(
        &self,
        data: impl Into<Deserializable<'de>>,
        seed: Seed,
    ) -> Result<<Seed as DeserializeSeed<'de>>::Value, Option<GraphqlError>> {
        match data.into() {
            Deserializable::Json(bytes) => {
                self.response.borrow_mut().data.push_borrowable_bytes(bytes.clone());
                seed.deserialize(&mut sonic_rs::Deserializer::from_slice(bytes))
                    .map_err(|err| {
                        if !self.bubbling_up_deser_error.get() {
                            tracing::error!("Deserialization failure: {err}");
                            Some(GraphqlError::invalid_subgraph_response())
                        } else {
                            None
                        }
                    })
            }
            Deserializable::JsonWithRawValues(bytes) => {
                self.response.borrow_mut().data.push_borrowable_bytes(bytes.clone());
                seed.deserialize(&mut serde_json::Deserializer::from_slice(bytes))
                    .map_err(|err| {
                        if !self.bubbling_up_deser_error.get() {
                            tracing::error!("Deserialization failure: {err}");
                            Some(GraphqlError::invalid_subgraph_response())
                        } else {
                            None
                        }
                    })
            }
            Deserializable::Cbor(bytes) => {
                self.response.borrow_mut().data.push_borrowable_bytes(bytes.clone());
                seed.deserialize(&mut minicbor_serde::Deserializer::new(bytes))
                    .map_err(|err| {
                        if !self.bubbling_up_deser_error.get() {
                            tracing::error!("Deserialization failure: {err}");
                            Some(GraphqlError::invalid_subgraph_response())
                        } else {
                            None
                        }
                    })
            }
            Deserializable::JsonValue(value) => seed.deserialize(value).map_err(|err| {
                if !self.bubbling_up_deser_error.get() {
                    tracing::error!("Deserialization failure: {err}");
                    Some(GraphqlError::invalid_subgraph_response())
                } else {
                    None
                }
            }),
        }
    }

    pub fn insert_empty_update(&self, parent_object: &'parent ResponseObjectRef) {
        self.response
            .borrow_mut()
            .insert_empty_update(parent_object, self.root_shape.id);
    }

    pub fn insert_empty_updates(
        &self,
        parent_objects: impl IntoIterator<
            IntoIter: ExactSizeIterator<Item = &'parent ResponseObjectRef>,
            Item = &'parent ResponseObjectRef,
        >,
    ) {
        self.response
            .borrow_mut()
            .insert_empty_updates(parent_objects, self.root_shape.id);
    }

    pub fn insert_propagated_empty_update(&self, parent_object: &'parent ResponseObjectRef) {
        self.response
            .borrow_mut()
            .insert_propagated_empty_update(parent_object, self.root_shape.id);
    }

    pub fn insert_error_update(
        &self,
        parent_object: &'parent ResponseObjectRef,
        errors: impl IntoIterator<Item = GraphqlError>,
    ) {
        self.response
            .borrow_mut()
            .insert_error_update(parent_object, self.root_shape.id, errors);
    }

    pub fn insert_error_updates(
        &self,
        parent_objects: impl IntoIterator<
            IntoIter: ExactSizeIterator<Item = &'parent ResponseObjectRef>,
            Item = &'parent ResponseObjectRef,
        >,
        errors: impl IntoIterator<Item = GraphqlError>,
    ) {
        self.response
            .borrow_mut()
            .insert_error_updates(parent_objects, self.root_shape.id, errors);
    }

    pub fn insert_errors(
        &self,
        parent_object: &'parent ResponseObjectRef,
        errors: impl IntoIterator<Item = GraphqlError>,
    ) {
        self.response
            .borrow_mut()
            .insert_errors(parent_object, self.root_shape.id, errors);
    }
}
