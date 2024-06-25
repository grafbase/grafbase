use std::{
    any::Any,
    convert::Infallible,
    fmt::{self, Debug, Display, Formatter},
    marker::PhantomData,
    sync::Arc,
};

use engine_response::error::ErrorExtensionValues;
use engine_value::ConstValue;
use thiserror::Error;

pub use engine_response::error::ServerError;

use crate::{LegacyInputType, Pos};

#[derive(Debug, serde::Serialize, serde::Deserialize, strum_macros::Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    PersistedQueryNotFound,
    InternalServerError,
}

/// Alias for `Result<T, ServerError>`.
pub type ServerResult<T> = std::result::Result<T, ServerError>;

/// An error parsing an input value.
///
/// This type is generic over T as it uses T's type name when converting to a regular error.
#[derive(Debug)]
pub struct InputValueError<T> {
    message: String,
    phantom: PhantomData<T>,
}

impl<T> InputValueError<T> {
    fn new(message: String) -> Self {
        Self {
            message,
            phantom: PhantomData,
        }
    }

    pub fn message(self) -> String {
        self.message
    }

    /// A custom error message.
    ///
    /// TODO: Temp function to provide an error message, should change as soon as we work on
    /// scalars to have a proper parsing and not just checking.
    #[must_use]
    pub fn ty_custom(ty: impl Display, msg: impl Display) -> Self {
        Self::new(format!(r#"Failed to parse "{ty}": {msg}"#))
    }
}

impl<T: LegacyInputType> InputValueError<T> {
    /// The expected input type did not match the actual input type.
    #[must_use]
    pub fn expected_type(actual: ConstValue) -> Self {
        Self::new(format!(
            r#"Expected input type "{}", found {}."#,
            T::type_name(),
            actual
        ))
    }

    /// A custom error message.
    ///
    /// Any type that implements `Display` is automatically converted to this if you use the `?`
    /// operator.
    #[must_use]
    pub fn custom(msg: impl Display) -> Self {
        Self::new(format!(r#"Failed to parse "{}": {msg}"#, T::type_name()))
    }

    /// Propagate the error message to a different type.
    pub fn propagate<U: LegacyInputType>(self) -> InputValueError<U> {
        if T::type_name() != U::type_name() {
            InputValueError::new(format!(
                r#"{} (occurred while parsing "{}")"#,
                self.message,
                U::type_name()
            ))
        } else {
            InputValueError::new(self.message)
        }
    }

    /// Convert the error into a server error.
    pub fn into_server_error(self, pos: Pos) -> ServerError {
        ServerError::new(self.message, Some(pos))
    }
}

impl<T: LegacyInputType, E: Display> From<E> for InputValueError<T> {
    fn from(error: E) -> Self {
        Self::custom(error)
    }
}

/// An error parsing a value of type `T`.
pub type InputValueResult<T> = Result<T, InputValueError<T>>;

/// An error with a message and optional extensions.
#[derive(Clone, serde::Serialize)]
pub struct Error {
    /// The error message.
    pub message: String,
    /// The source of the error.
    #[serde(skip)]
    pub source: Option<Arc<dyn Any + Send + Sync>>,
    /// Extensions to the error.
    #[serde(skip_serializing_if = "error_extensions_is_empty")]
    pub extensions: Option<ErrorExtensionValues>,
}

fn error_extensions_is_empty(values: &Option<ErrorExtensionValues>) -> bool {
    values.as_ref().map_or(true, |values| values.0.is_empty())
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Error")
            .field("message", &self.message)
            .field("extensions", &self.extensions)
            .finish_non_exhaustive()
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        self.message.eq(&other.message) && self.extensions.eq(&other.extensions)
    }
}

impl Error {
    /// Create an error from the given error message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
            extensions: None,
        }
    }

    /// Create an error with a type that implements `Display`, and it will also set the
    /// `source` of the error to this value.
    pub fn new_with_source(source: impl Display + Send + Sync + 'static) -> Self {
        Self {
            message: source.to_string(),
            source: Some(Arc::new(source)),
            extensions: None,
        }
    }

    /// Create an error with an error and a custom message error;
    pub fn new_with_source_and_message(message: impl Display, source: impl Send + Sync + 'static) -> Self {
        Self {
            message: message.to_string(),
            source: Some(Arc::new(source)),
            extensions: None,
        }
    }

    /// Convert the error to a server error.
    #[must_use]
    pub fn into_server_error(self, pos: Pos) -> ServerError {
        ServerError {
            message: self.message,
            source: self.source,
            locations: vec![pos],
            path: Vec::new(),
            extensions: self.extensions,
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::new(value.to_string())
    }
}

impl From<ServerError> for Error {
    fn from(value: ServerError) -> Self {
        Self {
            message: value.message,
            source: value.source,
            extensions: value.extensions,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::new(value.to_string())
    }
}

impl From<http::method::InvalidMethod> for Error {
    fn from(value: http::method::InvalidMethod) -> Self {
        Self::new(value.to_string())
    }
}

impl From<Infallible> for Error {
    fn from(value: Infallible) -> Self {
        Self::new(value.to_string())
    }
}

impl From<chrono::ParseError> for Error {
    fn from(value: chrono::ParseError) -> Self {
        Self::new(value.to_string())
    }
}

impl From<std::net::AddrParseError> for Error {
    fn from(value: std::net::AddrParseError) -> Self {
        Self::new(value.to_string())
    }
}

impl From<uuid::Error> for Error {
    fn from(value: uuid::Error) -> Self {
        Self::new(value.to_string())
    }
}

impl From<url::ParseError> for Error {
    fn from(value: url::ParseError) -> Self {
        Self::new(value.to_string())
    }
}

/// An alias for `Result<T, Error>`.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// An error parsing the request.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ParseRequestError {
    /// An IO error occurred.
    #[error("{0}")]
    Io(#[from] std::io::Error),

    /// The request's syntax was invalid.
    #[error("Invalid request: {0}")]
    InvalidRequest(Box<dyn std::error::Error + Send + Sync>),

    /// The request's files map was invalid.
    #[error("Invalid files map: {0}")]
    InvalidFilesMap(Box<dyn std::error::Error + Send + Sync>),

    /// The request's multipart data was invalid.
    #[error("Invalid multipart data")]
    InvalidMultipart(multer::Error),

    /// Missing "operators" part for multipart request.
    #[error("Missing \"operators\" part")]
    MissingOperatorsPart,

    /// Missing "map" part for multipart request.
    #[error("Missing \"map\" part")]
    MissingMapPart,

    /// It's not an upload operation
    #[error("It's not an upload operation")]
    NotUpload,

    /// Files were missing the request.
    #[error("Missing files")]
    MissingFiles,

    /// The request's payload is too large, and this server rejected it.
    #[error("Payload too large")]
    PayloadTooLarge,

    /// The request is a batch request, but the server does not support batch requests.
    #[error("Batch requests are not supported")]
    UnsupportedBatch,
}

impl From<multer::Error> for ParseRequestError {
    fn from(err: multer::Error) -> Self {
        match err {
            multer::Error::FieldSizeExceeded { .. } | multer::Error::StreamSizeExceeded { .. } => {
                ParseRequestError::PayloadTooLarge
            }
            _ => ParseRequestError::InvalidMultipart(err),
        }
    }
}

impl From<mime::FromStrError> for ParseRequestError {
    fn from(e: mime::FromStrError) -> Self {
        Self::InvalidRequest(Box::new(e))
    }
}

/// An error which can be extended into a `Error`.
pub trait ErrorExtensions: Sized {
    /// Convert the error to a `Error`.
    fn extend(&self) -> Error;

    /// Add extensions to the error, using a callback to make the extensions.
    fn extend_with<C>(self, cb: C) -> Error
    where
        C: FnOnce(&Self, &mut ErrorExtensionValues),
    {
        let mut new_extensions = Default::default();
        cb(&self, &mut new_extensions);

        let Error {
            message,
            source,
            extensions,
        } = self.extend();

        let mut extensions = extensions.unwrap_or_default();
        extensions.0.extend(new_extensions.0);

        Error {
            message,
            source,
            extensions: Some(extensions),
        }
    }
}

impl ErrorExtensions for Error {
    fn extend(&self) -> Error {
        self.clone()
    }
}

// implementing for &E instead of E gives the user the possibility to implement for E which does
// not conflict with this implementation acting as a fallback.
impl<E: Display> ErrorExtensions for &E {
    fn extend(&self) -> Error {
        Error {
            message: self.to_string(),
            source: None,
            extensions: None,
        }
    }
}

/// Extend a `Result`'s error value with [`ErrorExtensions`](trait.ErrorExtensions.html).
pub trait ResultExt<T, E>: Sized {
    /// Extend the error value of the result with the callback.
    fn extend_err<C>(self, cb: C) -> Result<T>
    where
        C: FnOnce(&E, &mut ErrorExtensionValues);

    /// Extend the result to a `Result`.
    fn extend(self) -> Result<T>;
}

// This is implemented on E and not &E which means it cannot be used on foreign types.
// (see example).
impl<T, E> ResultExt<T, E> for std::result::Result<T, E>
where
    E: ErrorExtensions + Send + Sync + 'static,
{
    fn extend_err<C>(self, cb: C) -> Result<T>
    where
        C: FnOnce(&E, &mut ErrorExtensionValues),
    {
        match self {
            Err(err) => Err(err.extend_with(|e, ee| cb(e, ee))),
            Ok(value) => Ok(value),
        }
    }

    fn extend(self) -> Result<T> {
        match self {
            Err(err) => Err(err.extend()),
            Ok(value) => Ok(value),
        }
    }
}
