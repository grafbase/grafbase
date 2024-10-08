use std::borrow::Cow;

use serde::ser::{SerializeMap, SerializeSeq};

use crate::response::{
    value::ResponseObjectField, ErrorCode, ExecutedResponse, GraphqlError, RefusedRequestResponse,
    RequestErrorResponse, Response, ResponseData, ResponseKeys, ResponseListId, ResponseObject, ResponseObjectId,
    ResponsePath, ResponseValue, UnpackedResponseEdge,
};

impl<OnOperationResponseHookOutput> serde::Serialize for Response<OnOperationResponseHookOutput> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Response::Executed(ExecutedResponse {
                operation,
                data,
                errors,
                ..
            }) => {
                let mut map = serializer.serialize_map(None)?;
                let keys = &operation.response_keys;
                if let Some(data) = data {
                    map.serialize_entry("data", &SerializableResponseData { keys, data })?;
                } else {
                    map.serialize_entry("data", &())?;
                }
                if !errors.is_empty() {
                    map.serialize_entry("errors", &SerializableErrors { keys, errors })?;
                }
                map.end()
            }
            Response::RequestError(RequestErrorResponse { errors, .. }) => {
                let mut map = serializer.serialize_map(None)?;
                // Shouldn't happen, but better safe than sorry.
                if !errors.is_empty() {
                    let empty_keys = ResponseKeys::default();
                    map.serialize_entry(
                        "errors",
                        &SerializableErrors {
                            keys: &empty_keys,
                            errors,
                        },
                    )?;
                }
                map.end()
            }
            Response::RefusedRequest(RefusedRequestResponse { error, .. }) => {
                let mut map = serializer.serialize_map(None)?;
                // Shouldn't happen, but better safe than sorry.
                let empty_keys = ResponseKeys::default();
                map.serialize_entry(
                    "errors",
                    &SerializableErrors {
                        keys: &empty_keys,
                        errors: std::array::from_ref(error),
                    },
                )?;
                map.end()
            }
        }
    }
}

struct SerializableErrors<'a> {
    keys: &'a ResponseKeys,
    errors: &'a [GraphqlError],
}

impl<'a> serde::Serialize for SerializableErrors<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.errors.len()))?;
        for error in self.errors {
            seq.serialize_element(&SerializableError { keys: self.keys, error })?;
        }
        seq.end()
    }
}

struct SerializableError<'a> {
    keys: &'a ResponseKeys,
    error: &'a GraphqlError,
}

impl<'a> serde::Serialize for SerializableError<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let size_hint = [
            true,
            !self.error.locations.is_empty(),
            self.error.path.is_some(),
            !self.error.extensions.is_empty(),
        ]
        .into_iter()
        .filter(|b| *b)
        .count();
        let mut map = serializer.serialize_map(Some(size_hint))?;
        map.serialize_entry("message", &self.error.message)?;
        if !self.error.locations.is_empty() {
            map.serialize_entry("locations", &self.error.locations)?;
        }
        if let Some(ref path) = self.error.path {
            map.serialize_entry("path", &SerializableResponsePath { keys: self.keys, path })?;
        }
        map.serialize_entry(
            "extensions",
            &SerializableExtension {
                code: self.error.code,
                extensions: &self.error.extensions,
            },
        )?;
        map.end()
    }
}

struct SerializableExtension<'a> {
    code: ErrorCode,
    extensions: &'a [(Cow<'static, str>, serde_json::Value)],
}

impl<'a> serde::Serialize for SerializableExtension<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let has_code = self.extensions.iter().any(|(key, _)| key == "code");
        let mut map = serializer.serialize_map(Some(self.extensions.len() + (!has_code as usize)))?;
        for (key, value) in self.extensions {
            map.serialize_entry(key, value)?;
        }
        if !has_code {
            map.serialize_entry("code", &self.code)?;
        }
        map.end()
    }
}

struct SerializableResponsePath<'a> {
    keys: &'a ResponseKeys,
    path: &'a ResponsePath,
}

impl<'a> serde::Serialize for SerializableResponsePath<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.path.len()))?;
        for edge in self.path.iter() {
            match edge.unpack() {
                UnpackedResponseEdge::Index(index) => seq.serialize_element(&index)?,
                // for requrest errors, keys will be empty. There shouldn't be any path within
                // those errors to begin with, but just in case better to output somthing than
                // crashing.
                UnpackedResponseEdge::BoundResponseKey(key) => {
                    seq.serialize_element(&self.keys.try_resolve(key.as_response_key()).unwrap_or("<unknown>"))?
                }
                UnpackedResponseEdge::ExtraFieldResponseKey(key) => {
                    seq.serialize_element(&self.keys.try_resolve(key).unwrap_or("<unknown>"))?
                }
            }
        }
        seq.end()
    }
}

struct SerializableResponseData<'a> {
    keys: &'a ResponseKeys,
    data: &'a ResponseData,
}

impl<'a> serde::Serialize for SerializableResponseData<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        SerializableResponseObject {
            keys: self.keys,
            data: self.data,
            object: &self.data[self.data.root],
        }
        .serialize(serializer)
    }
}

struct SerializableResponseObject<'a> {
    keys: &'a ResponseKeys,
    data: &'a ResponseData,
    object: &'a ResponseObject,
}

impl<'a> serde::Serialize for SerializableResponseObject<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.object.len()))?;
        // Thanks to the BoundResponseKey starting with the position and the fields being a BTreeMap
        // we're ensuring the fields are serialized in the order they appear in the query.
        for ResponseObjectField { edge, value, .. } in self.object.fields() {
            let UnpackedResponseEdge::BoundResponseKey(key) = edge.unpack() else {
                // Bound response keys are always first, anything after are extra fields which
                // don't need to be serialized.
                break;
            };
            map.serialize_key(&self.keys[key])?;
            match value {
                ResponseValue::Null => map.serialize_value(&())?,
                ResponseValue::Boolean { value, .. } => map.serialize_value(value)?,
                ResponseValue::Int { value, .. } => map.serialize_value(value)?,
                ResponseValue::Float { value, .. } => map.serialize_value(value)?,
                ResponseValue::String { value, .. } => map.serialize_value(&value)?,
                ResponseValue::StringId { id, .. } => map.serialize_value(&self.data.schema[*id])?,
                ResponseValue::BigInt { value, .. } => map.serialize_value(value)?,
                &ResponseValue::List {
                    part_id,
                    offset,
                    length,
                    ..
                } => map.serialize_value(&SerializableResponseList {
                    keys: self.keys,
                    data: self.data,
                    value: &self.data[ResponseListId {
                        part_id,
                        offset,
                        length,
                    }],
                })?,
                &ResponseValue::Object { part_id, index, .. } => map.serialize_value(&SerializableResponseObject {
                    keys: self.keys,
                    data: self.data,
                    object: &self.data[ResponseObjectId { part_id, index }],
                })?,
                ResponseValue::Json { value, .. } => map.serialize_value(value)?,
            }
        }
        map.end()
    }
}

struct SerializableResponseList<'a> {
    keys: &'a ResponseKeys,
    data: &'a ResponseData,
    value: &'a [ResponseValue],
}

impl<'a> serde::Serialize for SerializableResponseList<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.value.len()))?;
        for node in self.value {
            match node {
                ResponseValue::Null => seq.serialize_element(&())?,
                ResponseValue::Boolean { value, .. } => seq.serialize_element(value)?,
                ResponseValue::Int { value, .. } => seq.serialize_element(value)?,
                ResponseValue::Float { value, .. } => seq.serialize_element(value)?,
                ResponseValue::String { value, .. } => seq.serialize_element(&value)?,
                ResponseValue::StringId { id, .. } => seq.serialize_element(&self.data.schema[*id])?,
                ResponseValue::BigInt { value, .. } => seq.serialize_element(value)?,
                &ResponseValue::List {
                    part_id,
                    offset,
                    length,
                    ..
                } => seq.serialize_element(&SerializableResponseList {
                    keys: self.keys,
                    data: self.data,
                    value: &self.data[ResponseListId {
                        part_id,
                        offset,
                        length,
                    }],
                })?,
                &ResponseValue::Object { part_id, index, .. } => {
                    seq.serialize_element(&SerializableResponseObject {
                        keys: self.keys,
                        data: self.data,
                        object: &self.data[ResponseObjectId { part_id, index }],
                    })?
                }
                ResponseValue::Json { value, .. } => seq.serialize_element(value)?,
            }
        }
        seq.end()
    }
}
