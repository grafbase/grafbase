#![allow(unused)]
pub mod authorization_types;
pub mod context;
pub mod hooks_types;
pub mod token;

wasmtime::component::bindgen!({
    path: "../grafbase-sdk/wit/since_0_21_0/",
    world: "sdk",
    async: true,
    with: {
        "grafbase:sdk/cache": crate::extension::api::since_0_19_0::wit::cache,
        "grafbase:sdk/error": crate::extension::api::since_0_19_0::wit::error,
        "grafbase:sdk/grpc": crate::extension::api::since_0_14_0::wit::grpc,
        "grafbase:sdk/kafka-client": crate::extension::api::since_0_16_0::wit::kafka_client,
        "grafbase:sdk/nats-client": crate::extension::api::since_0_10_0::wit::nats_client,
        "grafbase:sdk/http-client": crate::extension::api::since_0_10_0::wit::http_client,
        "grafbase:sdk/postgres": crate::extension::api::since_0_15_0::wit::postgres,
        "grafbase:sdk/schema": crate::extension::api::since_0_17_0::wit::schema,
        "grafbase:sdk/headers": crate::extension::api::since_0_19_0::wit::headers,
        "grafbase:sdk/resolver-types": crate::extension::api::since_0_17_0::wit::resolver_types,
        "grafbase:sdk/authentication-types": crate::extension::api::since_0_19_0::wit::authentication_types,
        "grafbase:sdk/authorization-types": authorization_types,
        "grafbase:sdk/contracts-types": crate::extension::api::since_0_19_0::wit::contracts_types,
        "grafbase:sdk/event-types": crate::extension::api::since_0_19_0::wit::event_types,
        "grafbase:sdk/http-types": crate::extension::api::since_0_19_0::wit::http_types,
        "grafbase:sdk/event-queue": crate::extension::api::since_0_19_0::wit::event_queue,
        "grafbase:sdk/logger": crate::extension::api::since_0_19_0::wit::logger,
        "grafbase:sdk/context": context,
        "grafbase:sdk/token": token
    },
    trappable_imports: true,
    ownership: Borrowing {
        duplicate_if_necessary: true
    },
});

use grafbase::sdk;

pub use sdk::authorization_types::{
    AuthorizationDecisions, AuthorizationDecisionsDenySome, QueryElement, QueryElements, ResponseElement,
    ResponseElements,
};
pub use sdk::cache::Cache;
pub use sdk::context::*;
pub use sdk::contracts_types::{Contract, GraphqlSubgraphParam, GraphqlSubgraphResult};
pub use sdk::error::{Error, ErrorResponse};
pub use sdk::headers::{HeaderError, Headers};
pub use sdk::hooks_types::{HttpRequestPartsParam, HttpRequestPartsResult, OnRequestOutput};
pub use sdk::http_types::{HttpError, HttpMethod, HttpRequest, HttpResponse};
pub use sdk::nats_client::{NatsAuth, NatsKeyValue, NatsStreamConfig, NatsStreamDeliverPolicy, NatsSubscriber};
pub use sdk::resolver_types::{ArgumentsId, Data, Field, FieldId, Response, SelectionSet, SubscriptionItem};
pub use sdk::schema::{
    Directive, DirectiveSite, EnumDirectiveSite, FieldDefinitionDirective, FieldDefinitionDirectiveSite,
    InterfaceDirectiveSite, ObjectDirectiveSite, ScalarDirectiveSite, UnionDirectiveSite,
};
pub use sdk::token::Token;
