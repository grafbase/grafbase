mod authentication;
mod authorization;
mod field_resolver;
mod selection_set_resolver;

pub use authentication::*;
pub use authorization::*;
pub use field_resolver::*;
pub use selection_set_resolver::*;

pub trait ExtensionRuntime:
    AuthenticationExtension<Self::Context>
    + AuthorizationExtension<Self::Context>
    + FieldResolverExtension<Self::Context>
    + SelectionSetResolverExtension<Self::Context>
    + Send
    + Sync
    + 'static
{
    type Context: Send + Sync + 'static;
}

#[derive(Clone, Debug)]
pub enum Data {
    JsonBytes(Vec<u8>),
    CborBytes(Vec<u8>),
}
