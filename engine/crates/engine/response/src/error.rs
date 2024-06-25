use std::{
    any::Any,
    collections::BTreeMap,
    fmt::{self, Debug, Display, Formatter},
    sync::Arc,
};

use engine_parser::Pos;
use engine_value::ConstValue;
use query_path::QueryPathSegment;
use serde::{Deserialize, Serialize};

// use crate::{parser, LegacyInputType, Pos, QueryPathSegment, Value};

#[derive(Debug, serde::Serialize, serde::Deserialize, strum_macros::Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    PersistedQueryNotFound,
    InternalServerError,
}

/// Extensions to the error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct ErrorExtensionValues(pub BTreeMap<String, ConstValue>);

impl ErrorExtensionValues {
    /// Set an extension value.
    pub fn set(&mut self, name: impl AsRef<str>, value: impl Into<ConstValue>) {
        self.0.insert(name.as_ref().to_string(), value.into());
    }

    /// Unset an extension value.
    pub fn unset(&mut self, name: impl AsRef<str>) {
        self.0.remove(name.as_ref());
    }
}

/// An error in a GraphQL server.
#[derive(Clone, Serialize, Deserialize)]
pub struct ServerError {
    /// An explanatory message of the error.
    pub message: String,
    /// The source of the error.
    #[serde(skip)]
    pub source: Option<Arc<dyn Any + Send + Sync>>,
    /// Where the error occurred.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub locations: Vec<Pos>,
    /// If the error occurred in a resolver, the path to the error.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub path: Vec<QueryPathSegment>,
    /// Extensions to the error.
    #[serde(skip_serializing_if = "error_extensions_is_empty", default)]
    pub extensions: Option<ErrorExtensionValues>,
}

fn error_extensions_is_empty(values: &Option<ErrorExtensionValues>) -> bool {
    values.as_ref().map_or(true, |values| values.0.is_empty())
}

impl Debug for ServerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServerError")
            .field("message", &self.message)
            .field("locations", &self.locations)
            .field("path", &self.path)
            .field("extensions", &self.extensions)
            .finish_non_exhaustive()
    }
}

impl PartialEq for ServerError {
    fn eq(&self, other: &Self) -> bool {
        self.message.eq(&other.message)
            && self.locations.eq(&other.locations)
            && self.path.eq(&other.path)
            && self.extensions.eq(&other.extensions)
    }
}

impl ServerError {
    /// Create a new server error with the message.
    pub fn new(message: impl Into<String>, pos: Option<Pos>) -> Self {
        Self {
            message: message.into(),
            source: None,
            locations: pos.map(|pos| vec![pos]).unwrap_or_default(),
            path: Vec::new(),
            extensions: None,
        }
    }

    /// Create a new server error with the message.
    pub fn new_with_locations(message: impl Into<String>, locations: Vec<Pos>) -> Self {
        Self {
            message: message.into(),
            source: None,
            locations,
            path: Vec::new(),
            extensions: None,
        }
    }

    /// Get the source of the error.
    ///
    /// # Examples
    ///
    /// ```rust, ignore
    /// use engine::*;
    /// use std::io::ErrorKind;
    ///
    /// struct Query;
    ///
    /// #[Object]
    /// impl Query {
    ///     async fn value(&self) -> Result<i32> {
    ///         Err(Error::new_with_source(std::io::Error::new(ErrorKind::Other, "my error")))
    ///     }
    /// }
    ///
    /// let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    ///
    /// # tokio::runtime::Runtime::new().unwrap().block_on(async move {
    /// let err = schema.execute("{ value }").await.into_result().unwrap_err().remove(0);
    /// assert!(err.source::<std::io::Error>().is_some());
    /// # });
    /// ```
    pub fn source<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.source.as_ref().and_then(|err| err.downcast_ref())
    }

    #[doc(hidden)]
    #[must_use]
    pub fn with_path(self, path: Vec<QueryPathSegment>) -> Self {
        Self { path, ..self }
    }
}

impl Display for ServerError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl From<ServerError> for Vec<ServerError> {
    fn from(single: ServerError) -> Self {
        vec![single]
    }
}

impl From<engine_parser::Error> for ServerError {
    fn from(e: engine_parser::Error) -> Self {
        Self {
            message: e.to_string(),
            source: None,
            locations: e.positions().collect(),
            path: Vec::new(),
            extensions: None,
        }
    }
}

/// Alias for `Result<T, ServerError>`.
pub type ServerResult<T> = std::result::Result<T, ServerError>;
