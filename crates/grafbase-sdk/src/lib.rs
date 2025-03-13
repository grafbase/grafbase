#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
#![deny(missing_docs)]

mod cbor;
mod component;
#[doc(hidden)]
pub mod extension;
pub mod host;
pub mod host_io;
#[cfg(feature = "jq-selection")]
pub mod jq_selection;
#[cfg(feature = "test-utils")]
pub mod test;
pub mod types;

pub use component::SdkError;
pub use extension::{
    authorization::IntoQueryAuthorization, resolver::Subscription, AuthenticationExtension, AuthorizationExtension,
    ResolverExtension,
};
pub use grafbase_sdk_derive::{AuthenticationExtension, AuthorizationExtension, ResolverExtension};
pub use host::{AuthorizationContext, Headers};
pub use types::{Error, ErrorResponse, Token};
pub use wit::{NatsAuth, NatsStreamDeliverPolicy, SharedContext};

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
        path: "./wit/since_0_9_0/",
        world: "sdk",
    });

    pub use exports::grafbase::sdk::authentication::{Guest as AuthenticationGuest, Token};
    pub use exports::grafbase::sdk::authorization::{
        AuthorizationDecisions, AuthorizationDecisionsDenySome, Guest as AuthorizationGuest,
    };
    pub use exports::grafbase::sdk::init::Guest as InitGuest;
    pub use exports::grafbase::sdk::resolver::{FieldOutput, Guest as ResolverGuest};

    pub use grafbase::sdk::access_log::*;
    pub use grafbase::sdk::cache::*;
    pub use grafbase::sdk::context::{AuthorizationContext, SharedContext};
    pub use grafbase::sdk::directive::{
        DirectiveSite, EnumDirectiveSite, FieldDefinitionDirective, FieldDefinitionDirectiveSite,
        InterfaceDirectiveSite, ObjectDirectiveSite, QueryElement, QueryElements, ResponseElement, ResponseElements,
        ScalarDirectiveSite, SchemaDirective, UnionDirectiveSite,
    };
    pub use grafbase::sdk::error::*;
    pub use grafbase::sdk::headers::*;
    pub use grafbase::sdk::http_client::*;
    pub use grafbase::sdk::nats_client::*;
}

wit::export!(Component with_types_in wit);

mod sealed {
    pub trait Sealed {}
}
