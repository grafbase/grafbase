#![doc(hidden)]
#![allow(clippy::too_many_arguments, unused)]
#![expect(missing_docs)]

// Manually defining resolver_types to add some derives on the struct
// We can add custom derives in the generate macro, but it's applied on everything.
mod resolver_types;

wit_bindgen::generate!({
    skip: ["register-extension"],
    path: "./wit/since_0_17_0/",
    world: "sdk",
    with: {
        "grafbase:sdk/resolver-types": resolver_types,
    }
});

pub use exports::grafbase::sdk::authentication::{Guest as AuthenticationGuest, PublicMetadataEndpoint};
pub use exports::grafbase::sdk::authorization::Guest as AuthorizationGuest;
pub use exports::grafbase::sdk::hooks::Guest as HooksGuest;
pub use exports::grafbase::sdk::resolver::Guest as ResolverGuest;

pub use grafbase::sdk::access_log::*;
pub use grafbase::sdk::authorization_types::{
    AuthorizationDecisions, AuthorizationDecisionsDenySome, QueryElement, QueryElements, ResponseElement,
    ResponseElements,
};
pub use grafbase::sdk::cache::*;
pub use grafbase::sdk::error::{Error, ErrorResponse};
pub use grafbase::sdk::event_queue::{
    CacheStatus, Event, EventQueue, ExecutedHttpRequest, ExecutedOperation, ExecutedSubgraphRequest, ExtensionEvent,
    FieldError, GraphqlResponseStatus, RequestError, SubgraphRequestExecutionKind, SubgraphResponse,
};
pub use grafbase::sdk::grpc::*;
pub use grafbase::sdk::headers::HeaderError;
pub use grafbase::sdk::http_client::*;
pub use grafbase::sdk::kafka_client::*;
pub use grafbase::sdk::nats_client::*;
pub use grafbase::sdk::postgres::*;
pub use grafbase::sdk::schema::*;
pub use grafbase::sdk::shared_context::SharedContext;
pub use grafbase::sdk::token::*;
pub use resolver_types::{ArgumentsId, Data, Field, FieldId, Response, SelectionSet, SubscriptionItem};
