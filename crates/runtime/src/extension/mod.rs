mod authentication;
mod authorization;
mod field_resolver;
mod selection_set_resolver;

pub use authentication::*;
pub use authorization::*;
use bytes::Bytes;
pub use field_resolver::*;
pub use selection_set_resolver::*;

pub trait ExtensionRuntime:
    AuthenticationExtension<Self::Context>
    + AuthorizationExtension<Self::Context>
    + FieldResolverExtension<Self::Context>
    + ResolverExtension<Self::Context>
    + Send
    + Sync
    + 'static
{
    type Context: Send + Sync + 'static;
}

#[derive(Clone, Debug)]
pub enum Data {
    Json(Bytes),
    Cbor(Bytes),
}

impl Data {
    pub fn is_json(&self) -> bool {
        matches!(self, Data::Json(_))
    }

    pub fn is_cbor(&self) -> bool {
        matches!(self, Data::Cbor(_))
    }

    pub fn as_json(&self) -> Option<&Bytes> {
        match self {
            Data::Json(bytes) => Some(bytes),
            _ => None,
        }
    }

    pub fn as_cbor(&self) -> Option<&Bytes> {
        match self {
            Data::Cbor(bytes) => Some(bytes),
            _ => None,
        }
    }
}
