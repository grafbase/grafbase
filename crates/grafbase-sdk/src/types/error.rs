use std::{borrow::Cow, collections::HashMap};

use crate::{SdkError, cbor, wit};

/// Graphql Error with a message and extensions
#[derive(Clone)]
pub struct Error(pub(crate) wit::Error);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.message)
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Error")
            .field("message", &self.0.message)
            .field(
                "extensions",
                &self
                    .0
                    .extensions
                    .iter()
                    .map(|(key, value)| (key, cbor::from_slice::<serde_json::Value>(value).unwrap_or_default()))
                    .collect::<HashMap<_, _>>(),
            )
            .finish()
    }
}

impl From<Error> for wit::Error {
    fn from(err: Error) -> Self {
        err.0
    }
}

impl Error {
    /// Create a new error with a message.
    #[inline]
    pub fn new(message: impl Into<String>) -> Self {
        Self(wit::Error {
            message: message.into(),
            extensions: Vec::new(),
        })
    }

    /// Add an extension key value pair to the error.
    #[inline]
    pub fn extension(mut self, key: impl Into<String>, value: impl serde::Serialize) -> Result<Self, SdkError> {
        let value = crate::cbor::to_vec(&value)?;
        self.0.extensions.push((key.into(), value));
        Ok(self)
    }
}

impl wit::Error {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            extensions: Vec::new(),
        }
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Error(wit::Error {
            message: err,
            extensions: Vec::new(),
        })
    }
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Error(wit::Error {
            message: err.to_string(),
            extensions: Vec::new(),
        })
    }
}

impl From<Cow<'_, str>> for Error {
    fn from(err: Cow<'_, str>) -> Self {
        Error(wit::Error {
            message: err.into_owned(),
            extensions: Vec::new(),
        })
    }
}

impl From<SdkError> for Error {
    fn from(err: SdkError) -> Self {
        Error(wit::Error {
            message: err.to_string(),
            extensions: Vec::new(),
        })
    }
}

impl From<wit::Error> for Error {
    fn from(err: wit::Error) -> Self {
        Error(err)
    }
}

impl From<wit::HttpError> for Error {
    fn from(err: wit::HttpError) -> Self {
        match err {
            wit::HttpError::Timeout => "HTTP request timed out".into(),
            wit::HttpError::Request(err) => format!("Request error: {err}").into(),
            wit::HttpError::Connect(err) => format!("Connection error: {err}").into(),
        }
    }
}
