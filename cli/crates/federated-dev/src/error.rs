use std::fmt;

use async_graphql::{SimpleObject, Union};
use graphql_composition::Diagnostics;
use tokio::sync::{mpsc, oneshot, watch};

/// The error enum for the crate.
#[derive(Union, Debug, thiserror::Error)]
pub enum Error {
    /// Introspection of a subgraph failed
    #[error("error in subgraph introspection: {_0}")]
    SubgraphIntrospection(SubgraphIntrospectionError),
    /// Internal error happened, not related to user actions
    #[error("internal error: {_0}")]
    Internal(InternalError),
    /// Composing a subgraph was not successful
    #[error("error in subgraph composition: {_0}")]
    SubgraphComposition(SubgraphCompositionError),
}

impl<T> From<mpsc::error::SendError<T>> for Error {
    fn from(value: mpsc::error::SendError<T>) -> Self {
        Self::internal(value.to_string())
    }
}

impl From<oneshot::error::RecvError> for Error {
    fn from(value: oneshot::error::RecvError) -> Self {
        Self::internal(value.to_string())
    }
}

impl<T> From<watch::error::SendError<T>> for Error {
    fn from(value: watch::error::SendError<T>) -> Self {
        Self::internal(value.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::internal(value.to_string())
    }
}

impl Error {
    pub(crate) fn introspection(message: String) -> Self {
        let error = SubgraphIntrospectionError { message };
        Self::SubgraphIntrospection(error)
    }

    pub(crate) fn composition(diagnostics: &Diagnostics) -> Self {
        let mut messages = Vec::new();

        for message in diagnostics.iter_messages() {
            messages.push(message.to_string());
        }

        let error = SubgraphCompositionError { messages };

        Self::SubgraphComposition(error)
    }

    pub(crate) fn internal(message: impl Into<String>) -> Self {
        let error = InternalError {
            message: message.into(),
        };

        Self::Internal(error)
    }
}

#[derive(SimpleObject, Debug)]
pub struct SubgraphIntrospectionError {
    message: String,
}

impl fmt::Display for SubgraphIntrospectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

#[derive(SimpleObject, Debug)]
pub struct InternalError {
    message: String,
}

impl fmt::Display for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

#[derive(SimpleObject, Debug)]
pub struct SubgraphCompositionError {
    messages: Vec<String>,
}

impl fmt::Display for SubgraphCompositionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for message in &self.messages {
            writeln!(f, "- {message}")?;
        }

        Ok(())
    }
}

#[derive(SimpleObject, Debug)]
pub struct SubgraphParseError {
    message: String,
}

impl fmt::Display for SubgraphParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}
