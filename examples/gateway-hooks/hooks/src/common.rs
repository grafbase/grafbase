use std::sync::{LazyLock, Once};

use crate::bindings::component::grafbase::types::Error;
use reqwest::Client;
use tokio::runtime::Runtime;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub(crate) static REQWEST: LazyLock<Client> = LazyLock::new(Client::new);

/// We initialize this once for the whole component lifetime.
/// It is a single-threaded Tokio runtime, which can execute async rust code.
pub(crate) static RUNTIME: LazyLock<Runtime> =
    LazyLock::new(|| tokio::runtime::Builder::new_current_thread().build().unwrap());

/// Initializes the log subscriber, which must be called in the beginning of every hook to get output.
/// When the hook is called once, the Once construct prevents re-initializing the logger, which is already
/// in the component memory.
pub(crate) fn init_logging() {
    static LOG: Once = Once::new();

    LOG.call_once(|| {
        let log_layer = tracing_subscriber::fmt::layer().with_ansi(true).with_target(true);

        tracing_subscriber::registry()
            .with(log_layer)
            .with(EnvFilter::new("debug"))
            .init();
    });
}

pub(crate) fn maybe_read_input<T: serde::de::DeserializeOwned + Default>(data: &str) -> T {
    if data.is_empty() {
        // avoid logging any errors if nothing was present
        Default::default()
    } else {
        read_input(data).unwrap_or_default()
    }
}

pub(crate) fn read_input<T: serde::de::DeserializeOwned>(data: &str) -> Result<T, Error> {
    serde_json::from_str(data).map_err(|err| {
        tracing::error!("Failed to deserialize input: {err}");
        contract_error()
    })
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub(crate) struct Metadata {
    #[serde(default, rename = "allowRole")]
    pub allow_role: Option<String>,
}

pub(crate) fn error(message: impl Into<String>) -> Error {
    Error {
        message: message.into(),
        extensions: Vec::new(),
    }
}

pub(crate) fn contract_error() -> Error {
    error("Contract error")
}
