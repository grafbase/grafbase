use serde::ser::{SerializeMap, SerializeSeq};

use crate::response::{
    GraphqlError, InitialResponse, ResponseData, ResponseKeys, ResponseObject, ResponsePath, ResponseValue,
    ServerErrorResponse,
};

impl serde::Serialize for crate::Response {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            crate::Response::Initial(InitialResponse { data, errors }) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("data", &SerializableResponseData { data })?;
                if !errors.is_empty() {
                    map.serialize_entry(
                        "errors",
                        &SerializableErrors {
                            keys: &data.keys,
                            errors,
                        },
                    )?;
                }
                map.end()
            }
            crate::Response::Error(ServerErrorResponse { errors }) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("data", &serde_json::Value::Null)?;
                // Shouldn't happen, but better safe than sorry.
                if !errors.is_empty() {
                    map.serialize_entry("errors", errors)?;
                }
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
        let mut size_hint = 1;
        if !self.error.locations.is_empty() {
            size_hint += 1;
        }
        if self.error.path.is_some() {
            size_hint += 1;
        }
        let mut map = serializer.serialize_map(Some(size_hint))?;
        map.serialize_entry("message", &self.error.message)?;
        if !self.error.locations.is_empty() {
            map.serialize_entry("locations", &self.error.locations)?;
        }
        if let Some(ref path) = self.error.path {
            map.serialize_entry("path", &SerializableResponsePath { keys: self.keys, path })?;
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
        for segment in self.path.iter() {
            match segment.try_into_bound_response_key() {
                Ok(key) => seq.serialize_element(&self.keys[key])?,
                Err(index) => seq.serialize_element(&index)?,
            }
        }
        seq.end()
    }
}

struct SerializableResponseData<'a> {
    data: &'a ResponseData,
}

impl<'a> serde::Serialize for SerializableResponseData<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.data
            .root
            .as_ref()
            .map(|root_id| SerializableResponseObject {
                data: self.data,
                object: &self.data[*root_id],
            })
            .serialize(serializer)
    }
}

struct SerializableResponseObject<'a> {
    data: &'a ResponseData,
    object: &'a ResponseObject,
}

impl<'a> serde::Serialize for SerializableResponseObject<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.object.fields.len()))?;
        // Thanks to the BoundResponsKey starting with the position and the fields being a BTreeMap
        // we're ensuring the fields are serialized in the order they appear in the query.
        for (&key, value) in &self.object.fields {
            if key.is_internal() {
                // internal fields are all at the end.
                break;
            }
            map.serialize_key(&self.data.keys[key])?;
            match value {
                ResponseValue::Null => map.serialize_value(&serde_json::Value::Null)?,
                ResponseValue::Boolean { value, .. } => map.serialize_value(value)?,
                ResponseValue::Int { value, .. } => map.serialize_value(value)?,
                ResponseValue::Float { value, .. } => map.serialize_value(value)?,
                ResponseValue::String { value, .. } => map.serialize_value(&value)?,
                ResponseValue::StringId { id, .. } => map.serialize_value(&self.data.schema[*id])?,
                ResponseValue::BigInt { value, .. } => map.serialize_value(value)?,
                ResponseValue::List { id, .. } => map.serialize_value(&SerializableResponseList {
                    data: self.data,
                    value: &self.data[*id],
                })?,
                ResponseValue::Object { id, .. } => map.serialize_value(&SerializableResponseObject {
                    data: self.data,
                    object: &self.data[*id],
                })?,
                ResponseValue::Json { value, .. } => map.serialize_value(value)?,
            }
        }
        map.end()
    }
}

struct SerializableResponseList<'a> {
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
                ResponseValue::Null => seq.serialize_element(&serde_json::Value::Null)?,
                ResponseValue::Boolean { value, .. } => seq.serialize_element(value)?,
                ResponseValue::Int { value, .. } => seq.serialize_element(value)?,
                ResponseValue::Float { value, .. } => seq.serialize_element(value)?,
                ResponseValue::String { value, .. } => seq.serialize_element(&value)?,
                ResponseValue::StringId { id, .. } => seq.serialize_element(&self.data.schema[*id])?,
                ResponseValue::BigInt { value, .. } => seq.serialize_element(value)?,
                ResponseValue::List { id, .. } => seq.serialize_element(&SerializableResponseList {
                    data: self.data,
                    value: &self.data[*id],
                })?,
                ResponseValue::Object { id, .. } => seq.serialize_element(&SerializableResponseObject {
                    data: self.data,
                    object: &self.data[*id],
                })?,
                ResponseValue::Json { value, .. } => seq.serialize_element(value)?,
            }
        }
        seq.end()
    }
}
