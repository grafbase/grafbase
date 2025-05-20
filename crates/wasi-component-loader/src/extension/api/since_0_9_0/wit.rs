pub mod access_log;
pub mod authorization;
pub mod cache;
pub mod context;
pub mod directive;
pub mod error;
pub mod headers;
pub mod http_client;
pub mod nats_client;
pub mod resolver;
pub mod token;

wasmtime::component::bindgen!({
    path: "../grafbase-sdk/wit/since_0_9_0/",
    world: "sdk",
    async: true,
    with: {
        "grafbase:sdk/headers/headers": crate::resources::Headers,
        "grafbase:sdk/context/authorization-context": crate::resources::AuthorizationContext,
        "grafbase:sdk/context/shared-context": crate::resources::SharedContext,
        "grafbase:sdk/access-log/access-log": crate::resources::AccessLogSender,
        "grafbase:sdk/nats-client/nats-client": crate::resources::NatsClient,
        "grafbase:sdk/nats-client/nats-subscriber": crate::resources::NatsSubscriber,
        "grafbase:sdk/nats-client/nats-key-value": crate::resources::NatsKeyValue,
        "grafbase:sdk/directive": directive,
    },
    trappable_imports: true,
    ownership: Borrowing {
        duplicate_if_necessary: true
    },
});
