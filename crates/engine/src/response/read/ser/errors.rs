use std::borrow::Cow;

use error::ErrorPath;
use id_newtypes::BitSet;
use operation::{Location, ResponseKeys};
use serde::ser::{SerializeMap, SerializeSeq};

use crate::{
    ErrorCode,
    prepare::QueryModifications,
    response::{ErrorParts, ErrorPathSegment, GraphqlError, QueryErrorWithLocationAndPath},
};

pub(super) struct SerializableErrorParts<'a> {
    pub keys: &'a ResponseKeys,
    pub query_modifications: &'a QueryModifications,
    pub errors: &'a ErrorParts,
}

impl serde::Serialize for SerializableErrorParts<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.errors.len()))?;
        for part in self.errors.parts() {
            for error in part.errors() {
                seq.serialize_element(&SerializableError { keys: self.keys, error })?;
            }
            let mut bitset = BitSet::with_capacity(self.query_modifications.errors.len());
            for QueryErrorWithLocationAndPath {
                error_id,
                location,
                path,
            } in part.shared_query_errors()
            {
                if !bitset.put(*error_id) {
                    let error = &self.query_modifications[*error_id];
                    seq.serialize_element(&SerializableQueryError {
                        keys: self.keys,
                        error,
                        location: *location,
                        path,
                    })?;
                }
            }
        }
        seq.end()
    }
}

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

struct SerializableQueryError<'a> {
    keys: &'a ResponseKeys,
    error: &'a GraphqlError,
    location: Location,
    path: &'a ErrorPath,
}

impl serde::Serialize for SerializableQueryError<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(4))?;
        map.serialize_entry("message", &self.error.message)?;
        map.serialize_entry("locations", &[self.location])?;
        map.serialize_entry(
            "path",
            &SerializableResponsePath {
                keys: self.keys,
                path: self.path,
            },
        )?;
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

struct SerializableError<'a> {
    keys: &'a ResponseKeys,
    error: &'a GraphqlError,
}

impl serde::Serialize for SerializableError<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
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
