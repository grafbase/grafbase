#![doc(hidden)]
#![allow(clippy::too_many_arguments, unused)]
#![expect(missing_docs)]

// Manually defining resolver_types to add some derives on the struct
// We can add custom derives in the generate macro, but it's applied on everything.
mod resolver_types;

wit_bindgen::generate!({
    skip: ["register-extension"],
    path: "./wit/since_0_23_0/",
    world: "sdk",
    with: {
        "grafbase:sdk/resolver-types": resolver_types,
    },
});

pub(crate) use exports::grafbase::sdk::authentication::{Guest as AuthenticationGuest, PublicMetadataEndpoint};
pub(crate) use exports::grafbase::sdk::authorization::Guest as AuthorizationGuest;
pub(crate) use exports::grafbase::sdk::contracts::Guest as ContractsGuest;
pub(crate) use exports::grafbase::sdk::hooks::Guest as HooksGuest;
pub(crate) use exports::grafbase::sdk::resolver::Guest as ResolverGuest;

pub(crate) use grafbase::sdk::authorization_types::{
    AuthorizationDecisions, AuthorizationDecisionsDenySome, AuthorizationOutput, QueryElement, QueryElements,
    ResponseElement, ResponseElements,
};
pub(crate) use grafbase::sdk::cache::*;
pub(crate) use grafbase::sdk::context::*;
pub(crate) use grafbase::sdk::contracts_types::{Contract, GraphqlSubgraph};
pub(crate) use grafbase::sdk::error::{Error, ErrorResponse};
pub(crate) use grafbase::sdk::event_queue::EventQueue;
pub(crate) use grafbase::sdk::event_types::{
    CacheStatus, Event, ExecutedHttpRequest, ExecutedOperation, ExecutedSubgraphRequest, ExtensionEvent, FieldError,
    GraphqlResponseStatus, OperationType, RequestError, SubgraphRequestExecutionKind, SubgraphResponse,
};
pub(crate) use grafbase::sdk::grpc::*;
pub(crate) use grafbase::sdk::headers::HeaderError;
pub(crate) use grafbase::sdk::hooks_types::{HttpRequestParts, OnRequestOutput, OnResponseOutput};
pub(crate) use grafbase::sdk::http_client::HttpClient;
pub(crate) use grafbase::sdk::http_types::*;
pub(crate) use grafbase::sdk::kafka_client::*;
pub(crate) use grafbase::sdk::logger::*;
pub use grafbase::sdk::nats_client::*;
pub use grafbase::sdk::postgres::*;
pub(crate) use grafbase::sdk::schema::*;
pub(crate) use grafbase::sdk::token::Token;
pub(crate) use resolver_types::{ArgumentsId, Data, Field, FieldId, Response, SelectionSet, SubscriptionItem};
