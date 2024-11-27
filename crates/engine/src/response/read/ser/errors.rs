use std::borrow::Cow;

use serde::ser::{SerializeMap, SerializeSeq};

use crate::{
    response::{GraphqlError, ResponseKeys, ResponsePath, UnpackedResponseEdge},
    ErrorCode,
};

pub(super) struct SerializableErrors<'a> {
    pub keys: &'a ResponseKeys,
    pub errors: &'a [GraphqlError],
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

pub(super) struct SerializableExtension<'a> {
    pub code: ErrorCode,
    pub extensions: &'a [(Cow<'static, str>, serde_json::Value)],
}

impl<'a> serde::Serialize for SerializableExtension<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut has_code = false;
        let mut map = serializer.serialize_map(None)?;
        for (key, value) in self.extensions {
            has_code |= key == "code";
            map.serialize_entry(key, value)?;
        }
        if !has_code {
            map.serialize_entry("code", &self.code)?;
        }
        map.end()
    }
}
