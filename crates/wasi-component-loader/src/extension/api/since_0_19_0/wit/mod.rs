pub mod authentication_types;
pub mod authorization_types;
pub mod cache;
pub mod contract_types;
pub mod error;
pub mod event_queue;
pub mod event_types;
pub mod headers;
pub mod hooks_types;
pub mod http_client;
pub mod http_types;
pub mod logger;

wasmtime::component::bindgen!({
    path: "../grafbase-sdk/wit/since_0_19_0/",
    world: "sdk",
    async: true,
    with: {
        "grafbase:sdk/cache": cache,
        "grafbase:sdk/error": error,
        "grafbase:sdk/grpc": crate::extension::api::since_0_14_0::wit::grpc,
        "grafbase:sdk/kafka-client": crate::extension::api::since_0_16_0::wit::kafka_client,
        "grafbase:sdk/nats-client": crate::extension::api::since_0_10_0::wit::nats_client,
        "grafbase:sdk/postgres": crate::extension::api::since_0_15_0::wit::postgres,
        "grafbase:sdk/token": crate::extension::api::since_0_10_0::wit::token,
        "grafbase:sdk/schema": crate::extension::api::since_0_17_0::wit::schema,
        "grafbase:sdk/headers/headers": crate::resources::Headers,
        "grafbase:sdk/resolver-types": crate::extension::api::since_0_17_0::wit::resolver_types,
        "grafbase:sdk/authorization-types": authorization_types,
        "grafbase:sdk/authentication-types": authentication_types,
        "grafbase:sdk/event-queue/event-queue": crate::resources::EventQueueProxy,
        "grafbase:sdk/logger/file-logger": crate::resources::FileLogger,
        "grafbase:sdk/shared-context": crate::extension::api::since_0_17_0::wit::shared_context
    },
    trappable_imports: true,
    ownership: Borrowing {
        duplicate_if_necessary: true
    },
});
