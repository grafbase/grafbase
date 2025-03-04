pub mod directive;

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
        "grafbase:sdk/directive": crate::wit::directive,
    },
    trappable_imports: true,
    ownership: Borrowing {
        duplicate_if_necessary: true
    },
});

// Having arguments as &[u8] is a massive pain to deal with and bindgen doesn't allow a lot of
// flexibility. Either everything is borrowed or nothing is. So wrote those manually.
pub use directive::{
    EnumDirectiveSite, FieldDefinitionDirective, FieldDefinitionDirectiveSite, InterfaceDirectiveSite,
    ObjectDirectiveSite, ScalarDirectiveSite, SchemaDirective, UnionDirectiveSite,
};
pub use grafbase::sdk::types::*;
