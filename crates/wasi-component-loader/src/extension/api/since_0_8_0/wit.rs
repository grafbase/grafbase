pub mod directive;

wasmtime::component::bindgen!({
    path: "../grafbase-sdk/wit/since_0_8_0/",
    world: "sdk",
    async: true,
    with: {
        "grafbase:sdk/types/headers": crate::resources::Headers,
        "grafbase:sdk/types/shared-context": crate::resources::SharedContext,
        "grafbase:sdk/types/access-log": crate::resources::AccessLogSender,
        "grafbase:sdk/types/nats-client": crate::resources::NatsClient,
        "grafbase:sdk/types/nats-subscriber": crate::resources::NatsSubscriber,
        "grafbase:sdk/types/nats-key-value": crate::resources::NatsKeyValue,
        "grafbase:sdk/directive": directive,
    },
    trappable_imports: true,
    ownership: Borrowing {
        duplicate_if_necessary: true
    },
});
