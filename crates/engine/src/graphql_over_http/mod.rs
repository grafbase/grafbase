//! All the necessary logic to follow the GraphQL over HTTP spec:
//!
//! https://github.com/graphql/graphql-over-http/blob/main/spec/GraphQLOverHTTP.md
//!
mod format;
mod response;

use bytes::Bytes;
use error::ErrorCode;
pub(crate) use format::*;
use futures_util::stream::BoxStream;
use grafbase_telemetry::graphql::GraphqlExecutionTelemetry;
pub(crate) use response::*;

pub enum Body {
    Bytes(Bytes),
    Stream(BoxStream<'static, Result<Bytes, String>>),
}

impl Body {
    pub fn into_stream(self) -> BoxStream<'static, Result<Bytes, String>> {
        match self {
            Body::Bytes(bytes) => Box::pin(futures_util::stream::once(async move { Ok(bytes) })),
            Body::Stream(stream) => stream,
        }
    }

    pub fn into_bytes(self) -> Option<Bytes> {
        match self {
            Body::Bytes(bytes) => Some(bytes),
            Body::Stream(_) => None,
        }
    }
}

impl<T> From<T> for Body
where
    Bytes: From<T>,
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

// Required to be part of the request.extensions
impl Clone for TelemetryExtension {
    fn clone(&self) -> Self {
        unreachable!("TelemetryExtension is not meant to be cloned.")
    }
}
