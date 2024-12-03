use std::sync::Once;

use grafbase_hooks::Error;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

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
