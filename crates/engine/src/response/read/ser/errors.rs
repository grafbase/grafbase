use std::borrow::Cow;

use serde::ser::{SerializeMap, SerializeSeq};

use crate::{
    response::{ErrorPathSegment, GraphqlError, ResponseKeys},
    ErrorCode,
};

pub(super) struct SerializableErrors<'a> {
    pub keys: &'a ResponseKeys,
    pub errors: &'a [GraphqlError],
}

impl serde::Serialize for SerializableErrors<'_> {
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

impl serde::Serialize for SerializableError<'_> {
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
    path: &'a [ErrorPathSegment],
}

impl serde::Serialize for SerializableResponsePath<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.path.len()))?;
        for segment in self.path {
            match segment {
                ErrorPathSegment::Field(key) => {
                    seq.serialize_element(&self.keys[*key])?;
                }
                ErrorPathSegment::Index(index) => seq.serialize_element(&index)?,
                ErrorPathSegment::UnknownField(name) => {
                    seq.serialize_element(name)?;
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

impl serde::Serialize for SerializableExtension<'_> {
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
