#![allow(unused)]
pub mod cache;
pub mod hooks_types;

wasmtime::component::bindgen!({
    path: "../grafbase-sdk/wit/since_0_23_0/",
    world: "sdk",
    async: true,
    with: {
        "grafbase:sdk/cache": cache,
        "grafbase:sdk/error": crate::extension::api::since_0_19_0::wit::error,
        "grafbase:sdk/grpc": crate::extension::api::since_0_14_0::wit::grpc,
        "grafbase:sdk/kafka-client": crate::extension::api::since_0_16_0::wit::kafka_client,
        "grafbase:sdk/nats-client": crate::extension::api::since_0_10_0::wit::nats_client,
        "grafbase:sdk/http-client": crate::extension::api::since_0_19_0::wit::http_client,
        "grafbase:sdk/postgres": crate::extension::api::since_0_15_0::wit::postgres,
        "grafbase:sdk/schema": crate::extension::api::since_0_17_0::wit::schema,
        "grafbase:sdk/headers": crate::extension::api::since_0_19_0::wit::headers,
        "grafbase:sdk/resolver-types": crate::extension::api::since_0_17_0::wit::resolver_types,
        "grafbase:sdk/authentication-types": crate::extension::api::since_0_19_0::wit::authentication_types,
        "grafbase:sdk/authorization-types": crate::extension::api::since_0_21_0::wit::authorization_types,
        "grafbase:sdk/contracts-types": crate::extension::api::since_0_19_0::wit::contracts_types,
        "grafbase:sdk/event-types": crate::extension::api::since_0_19_0::wit::event_types,
        "grafbase:sdk/http-types": crate::extension::api::since_0_19_0::wit::http_types,
        "grafbase:sdk/event-queue": crate::extension::api::since_0_21_0::wit::event_queue,
        "grafbase:sdk/logger": crate::extension::api::since_0_19_0::wit::logger,
        "grafbase:sdk/context": crate::extension::api::since_0_21_0::wit::context,
        "grafbase:sdk/token": crate::extension::api::since_0_21_0::wit::token
    },
    trappable_imports: true,
    ownership: Borrowing {
        duplicate_if_necessary: true
    },
});

use grafbase::sdk;

pub(crate) use sdk::authorization_types::{
    AuthorizationDecisions, AuthorizationDecisionsDenySome, QueryElement, QueryElements, ResponseElement,
    ResponseElements,
};
pub(crate) use sdk::cache::Cache;
pub(crate) use sdk::context::{AuthenticatedRequestContext, AuthorizedOperationContext, RequestContext};
pub(crate) use sdk::contracts_types::{Contract, GraphqlSubgraphParam, GraphqlSubgraphResult};
pub(crate) use sdk::error::{Error, ErrorResponse};
pub(crate) use sdk::headers::{HeaderError, Headers};
pub(crate) use sdk::hooks_types::{HttpRequestPartsParam, HttpRequestPartsResult, OnRequestOutput, OnResponseOutput};
pub(crate) use sdk::http_types::{HttpError, HttpMethod, HttpRequest, HttpResponse};
pub(crate) use sdk::nats_client::{NatsAuth, NatsKeyValue, NatsStreamConfig, NatsStreamDeliverPolicy, NatsSubscriber};
pub(crate) use sdk::resolver_types::{ArgumentsId, Data, Field, FieldId, Response, SelectionSet, SubscriptionItem};
pub(crate) use sdk::schema::{
    Directive, DirectiveSite, EnumDirectiveSite, FieldDefinitionDirective, FieldDefinitionDirectiveSite,
    InterfaceDirectiveSite, ObjectDirectiveSite, ScalarDirectiveSite, UnionDirectiveSite,
};
pub(crate) use sdk::token::Token;
