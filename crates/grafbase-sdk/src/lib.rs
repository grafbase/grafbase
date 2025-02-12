#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
#![deny(missing_docs)]

#[doc(hidden)]
pub mod extension;
pub mod host_io;
#[cfg(feature = "test-utils")]
pub mod test;
pub mod types;

pub use extension::{Authenticator, Extension, Resolver};
pub use grafbase_sdk_derive::{AuthenticationExtension, ResolverExtension};
#[doc(hidden)]
pub use wit::ExtensionType;
pub use wit::{Error, Headers, SharedContext};

struct Component;

#[cfg(target_arch = "wasm32")]
#[link_section = "sdk:minimum-gateway-version"]
#[doc(hidden)]
pub static MINIMUM_GATEWAY_VERSION: [u8; 6] =
    *include_bytes!(concat!(env!("OUT_DIR"), "/minimum_gateway_version_bytes"));

#[cfg(target_arch = "wasm32")]
#[link_section = "sdk:version"]
#[doc(hidden)]
pub static SDK_VERSION: [u8; 6] = *include_bytes!(concat!(env!("OUT_DIR"), "/sdk_version_bytes"));

#[doc(hidden)]
mod wit {
    #![allow(clippy::too_many_arguments, clippy::missing_safety_doc, missing_docs)]

    wit_bindgen::generate!({
        skip: ["register-extension"],
        path: "./wit/world.wit",
    });
}

wit::export!(Component with_types_in wit);
