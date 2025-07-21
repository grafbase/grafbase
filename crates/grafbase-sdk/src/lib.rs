#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
#![doc = "# Features"]
#![doc = document_features::document_features!()]
#![deny(missing_docs, unused_crate_dependencies)]

mod cbor;
mod component;
#[doc(hidden)]
pub mod extension;
pub mod host_io;
#[cfg(feature = "jq-selection")]
pub mod jq_selection;
#[cfg(feature = "test-utils")]
pub mod test;
pub mod types;
mod wit;

pub use component::SdkError;
pub use extension::{
    AuthenticationExtension, AuthorizationExtension, ContractsExtension, HooksExtension, IntoQueryAuthorization,
    IntoSubscription, ResolverExtension, Subscription,
};
pub use grafbase_sdk_derive::{
    AuthenticationExtension, AuthorizationExtension, ContractsExtension, HooksExtension, ResolverExtension,
};

use component::Component;

#[cfg(target_arch = "wasm32")]
#[unsafe(link_section = "sdk:minimum-gateway-version")]
#[doc(hidden)]
pub static MINIMUM_GATEWAY_VERSION: [u8; 6] =
    *include_bytes!(concat!(env!("OUT_DIR"), "/minimum_gateway_version_bytes"));

#[cfg(target_arch = "wasm32")]
#[unsafe(link_section = "sdk:version")]
#[doc(hidden)]
pub static SDK_VERSION: [u8; 6] = *include_bytes!(concat!(env!("OUT_DIR"), "/sdk_version_bytes"));

wit::export!(Component with_types_in wit);

mod sealed {
    pub trait Sealed {}
}
