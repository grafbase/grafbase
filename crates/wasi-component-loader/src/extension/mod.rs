mod instance;
mod loader;
mod pool;
mod runtime;
mod types_impl;

pub(crate) use instance::*;
pub(crate) use loader::*;
pub use loader::{ExtensionGuestConfig, SchemaDirective};
pub use runtime::*;

pub(crate) mod wit {
    wasmtime::component::bindgen!({
        path: "../grafbase-sdk/wit",
        world: "sdk",
        async: true,
        with: {
            "grafbase:sdk/types/headers": crate::resources::Headers,
            "grafbase:sdk/types/shared-context": crate::resources::SharedContext,
            "grafbase:sdk/types/access-log": crate::resources::AccessLogSender,
            "grafbase:sdk/types/nats-client": crate::resources::NatsClient,
            "grafbase:sdk/types/nats-subscriber": crate::resources::NatsSubscriber,
            "grafbase:sdk/types/nats-key-value": crate::resources::NatsKeyValue,
        },
        trappable_imports: true,
        ownership: Borrowing {
            duplicate_if_necessary: true
        },
    });

    pub use grafbase::sdk::types::*;
}
