//! All the necessary logic to follow the GraphQL over HTTP spec:
//!
//! https://github.com/graphql/graphql-over-http/blob/main/spec/GraphQLOverHTTP.md
//!
mod format;
mod response;

pub(crate) use format::*;
use futures_util::stream::BoxStream;
pub(crate) use response::*;
use runtime::bytes::OwnedOrSharedBytes;

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
