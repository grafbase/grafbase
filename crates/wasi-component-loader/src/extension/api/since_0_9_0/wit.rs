mod compatibility;
pub mod directive;

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

pub use exports::grafbase::sdk::authentication;
pub use exports::grafbase::sdk::authorization;
pub use exports::grafbase::sdk::init;
pub use exports::grafbase::sdk::resolver;
pub use grafbase::sdk::access_log;
pub use grafbase::sdk::cache;
pub use grafbase::sdk::context;
pub use grafbase::sdk::error;
pub use grafbase::sdk::headers;
pub use grafbase::sdk::http_client;
pub use grafbase::sdk::nats_client;
pub use grafbase::sdk::token;
