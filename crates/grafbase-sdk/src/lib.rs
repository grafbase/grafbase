#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
#![deny(missing_docs)]
#![expect(unsafe_op_in_unsafe_fn)]

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

pub use component::SdkError;
pub use extension::{resolver::Subscription, Authenticator, Extension, Resolver};
pub use grafbase_sdk_derive::{AuthenticationExtension, ResolverExtension};
pub use types::{Error, ErrorResponse};
pub use wit::{Headers, NatsAuth, NatsStreamDeliverPolicy, SharedContext};

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

#[doc(hidden)]
mod wit {
    #![expect(missing_docs)]

    wit_bindgen::generate!({
        skip: ["register-extension"],
        path: "./wit/world.wit",
        world: "sdk",
    });

    pub use exports::grafbase::sdk::extension::Guest;
    pub use grafbase::sdk::types::*;
}

wit::export!(Component with_types_in wit);
