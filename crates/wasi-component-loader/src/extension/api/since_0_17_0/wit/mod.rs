pub mod authorization_types;
pub mod error;
pub mod event_queue;
pub mod headers;
pub mod http_client;
pub mod resolver_types;
pub mod schema;
pub mod shared_context;

wasmtime::component::bindgen!({
    path: "../grafbase-sdk/wit/since_0_17_0/",
    world: "sdk",
    async: true,
    with: {
        "grafbase:sdk/access-log": crate::extension::api::since_0_9_0::wit::access_log,
        "grafbase:sdk/cache": crate::extension::api::since_0_9_0::wit::cache,
        "grafbase:sdk/grpc": crate::extension::api::since_0_14_0::wit::grpc,
        "grafbase:sdk/kafka-client": crate::extension::api::since_0_16_0::wit::kafka_client,
        "grafbase:sdk/nats-client": crate::extension::api::since_0_9_0::wit::nats_client,
        "grafbase:sdk/postgres": crate::extension::api::since_0_15_0::wit::postgres,
        "grafbase:sdk/token": crate::extension::api::since_0_10_0::wit::token,
        "grafbase:sdk/schema": schema,
        "grafbase:sdk/headers/headers": crate::resources::Headers,
        "grafbase:sdk/resolver-types": resolver_types,
        "grafbase:sdk/authorization-types": authorization_types,
        "grafbase:sdk/event-queue/event-queue": crate::resources::EventQueueProxy,
        "grafbase:sdk/shared-context": shared_context
    },
    trappable_imports: true,
    ownership: Borrowing {
        duplicate_if_necessary: true
    },
});
