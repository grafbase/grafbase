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

pub use component::SdkError;
pub use extension::{
    AuthenticationExtension, AuthorizationExtension, FieldResolverExtension, HooksExtension, IntoQueryAuthorization,
    SelectionSetResolverExtension, Subscription,
};
pub use grafbase_sdk_derive::{
    AuthenticationExtension, AuthorizationExtension, FieldResolverExtension, HooksExtension,
    SelectionSetResolverExtension,
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

#[doc(hidden)]
#[allow(clippy::too_many_arguments)]
mod wit {
    #![expect(missing_docs)]

    wit_bindgen::generate!({
        skip: ["register-extension"],
        path: "./wit/since_0_17_0/",
        world: "sdk",
    });

    pub use exports::grafbase::sdk::authentication::Guest as AuthenticationGuest;
    pub use exports::grafbase::sdk::authorization::Guest as AuthorizationGuest;
    pub use exports::grafbase::sdk::field_resolver::Guest as FieldResolverGuest;
    pub use exports::grafbase::sdk::hooks::Guest as HooksGuest;
    pub use exports::grafbase::sdk::selection_set_resolver::Guest as SelectionSetResolverGuest;

    pub use grafbase::sdk::access_log::*;
    pub use grafbase::sdk::authorization_types::{AuthorizationDecisions, AuthorizationDecisionsDenySome};
    pub use grafbase::sdk::cache::*;
    pub use grafbase::sdk::directive::*;
    pub use grafbase::sdk::error::*;
    pub use grafbase::sdk::event_queue::{
        CacheStatus, Event, ExecutedHttpRequest, ExecutedOperation, ExecutedSubgraphRequest, FieldError,
        GraphqlResponseStatus, RequestError, SubgraphRequestExecutionKind, SubgraphResponse,
    };
    pub use grafbase::sdk::field_resolver_types::FieldOutput;
    pub use grafbase::sdk::grpc::*;
    pub use grafbase::sdk::headers::*;
    pub use grafbase::sdk::http_client::*;
    pub use grafbase::sdk::kafka_client::*;
    pub use grafbase::sdk::nats_client::*;
    pub use grafbase::sdk::postgres::*;
    pub use grafbase::sdk::resolver_types::*;
    pub use grafbase::sdk::schema::*;
    pub use grafbase::sdk::selection_set_resolver_types;
    pub use grafbase::sdk::token::*;
}

wit::export!(Component with_types_in wit);

mod sealed {
    pub trait Sealed {}
}
