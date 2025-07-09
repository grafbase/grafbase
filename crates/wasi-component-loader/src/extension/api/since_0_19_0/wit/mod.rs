pub mod logger;

wasmtime::component::bindgen!({
    path: "../grafbase-sdk/wit/since_0_19_0/",
    world: "sdk",
    async: true,
    with: {
        "grafbase:sdk/cache": crate::extension::api::since_0_9_0::wit::cache,
        "grafbase:sdk/grpc": crate::extension::api::since_0_14_0::wit::grpc,
        "grafbase:sdk/kafka-client": crate::extension::api::since_0_16_0::wit::kafka_client,
        "grafbase:sdk/http-client": crate::extension::api::since_0_17_0::wit::http_client,
        "grafbase:sdk/nats-client": crate::extension::api::since_0_9_0::wit::nats_client,
        "grafbase:sdk/postgres": crate::extension::api::since_0_15_0::wit::postgres,
        "grafbase:sdk/token": crate::extension::api::since_0_10_0::wit::token,
        "grafbase:sdk/error": crate::extension::api::since_0_17_0::wit::error,
        "grafbase:sdk/schema": crate::extension::api::since_0_17_0::wit::schema,
        "grafbase:sdk/headers": crate::extension::api::since_0_17_0::wit::headers,
        "grafbase:sdk/resolver-types": crate::extension::api::since_0_17_0::wit::resolver_types,
        "grafbase:sdk/authorization-types": crate::extension::api::since_0_17_0::wit::authorization_types,
        "grafbase:sdk/event-queue": crate::extension::api::since_0_18_0::wit::event_queue,
        "grafbase:sdk/logger/file-logger": crate::resources::FileLogger,
        "grafbase:sdk/authorization-types": crate::extension::api::since_0_17_0::wit::authorization_types,
        "grafbase:sdk/shared-context": crate::extension::api::since_0_17_0::wit::shared_context
    },
    trappable_imports: true,
    ownership: Borrowing {
        duplicate_if_necessary: true
    },
});
