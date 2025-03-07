wasmtime::component::bindgen!({
    path: "../grafbase-sdk/wit/since_0_8_0/",
    world: "sdk",
    async: true,
    with: {
        "grafbase:sdk/headers/headers": crate::resources::Headers,
        "grafbase:sdk/context/shared-context": crate::resources::SharedContext,
        "grafbase:sdk/access-log/access-log": crate::resources::AccessLogSender,
        "grafbase:sdk/nats-client/nats-client": crate::resources::NatsClient,
        "grafbase:sdk/nats-client/nats-subscriber": crate::resources::NatsSubscriber,
        "grafbase:sdk/nats-client/nats-key-value": crate::resources::NatsKeyValue,
        "grafbase:sdk/directive": crate::wit::directive,
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

pub struct Extension;

impl Extension {
    pub(crate) fn add_to_linker(
        &self,
        linker: &mut wasmtime::component::Linker<crate::WasiState>,
    ) -> wasmtime::Result<()> {
        grafbase::sdk::access_log::add_to_linker(linker, |state| state)?;
        grafbase::sdk::cache::add_to_linker(linker, |state| state)?;
        grafbase::sdk::context::add_to_linker(linker, |state| state)?;
        grafbase::sdk::error::add_to_linker(linker, |state| state)?;
        grafbase::sdk::headers::add_to_linker(linker, |state| state)?;
        grafbase::sdk::http_client::add_to_linker(linker, |state| state)?;
        grafbase::sdk::nats_client::add_to_linker(linker, |state| state)?;

        Ok(())
    }
}
