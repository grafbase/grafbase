//! All the necessary logic to follow the GraphQL over HTTP spec:
//!
//! https://github.com/graphql/graphql-over-http/blob/main/spec/GraphQLOverHTTP.md
//!
mod format;
mod response;

pub(crate) use format::*;
use futures_util::stream::BoxStream;
use grafbase_telemetry::graphql::GraphqlExecutionTelemetry;
pub(crate) use response::*;
use runtime::bytes::OwnedOrSharedBytes;

pub use crate::response::error::code::ErrorCode;

pub enum Body {
    Bytes(OwnedOrSharedBytes),
    Stream(BoxStream<'static, Result<OwnedOrSharedBytes, String>>),
}

impl Body {
    pub fn into_stream(self) -> BoxStream<'static, Result<OwnedOrSharedBytes, String>> {
        match self {
            Body::Bytes(bytes) => Box::pin(futures_util::stream::once(async move { Ok(bytes) })),
            Body::Stream(stream) => stream,
        }
    }

    pub fn into_bytes(self) -> Option<OwnedOrSharedBytes> {
        match self {
            Body::Bytes(bytes) => Some(bytes),
            Body::Stream(_) => None,
        }
    }
}

impl<T> From<T> for Body
where
    OwnedOrSharedBytes: From<T>,
{
    fn from(bytes: T) -> Self {
        Body::Bytes(bytes.into())
    }
}

pub enum TelemetryExtension {
    Ready(GraphqlExecutionTelemetry<ErrorCode>),
    Future(futures::channel::oneshot::Receiver<GraphqlExecutionTelemetry<ErrorCode>>),
}

impl Default for TelemetryExtension {
    fn default() -> Self {
        TelemetryExtension::Ready(GraphqlExecutionTelemetry::default())
    }
}

pub enum HooksExtension<C> {
    Single {
        context: C,
        on_operation_response_output: Option<Vec<u8>>,
    },
    Batch {
        context: C,
        on_operation_response_outputs: Vec<Vec<u8>>,
    },
    Stream {
        context: C,
        on_operation_response_outputs: futures::channel::mpsc::Receiver<Vec<u8>>,
    },
}

// Required to be part of the request.extensions
impl Clone for TelemetryExtension {
    fn clone(&self) -> Self {
        unreachable!("TelemetryExtension is not meant to be cloned.")
    }
}

// Required to be part of the request.extensions
impl<C> Clone for HooksExtension<C> {
    fn clone(&self) -> Self {
        unreachable!("HooksExtensions is not meant to be cloned.")
    }
}
