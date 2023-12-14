use std::path::{Path, PathBuf};
use tokio::sync::broadcast::Receiver;

pub type EventSender = tokio::sync::broadcast::Sender<Event>;
pub type EventReceiver = tokio::sync::broadcast::Sender<Event>;

// TODO: much of this can go away I think

/// server lifecycle related events
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// emitted when a schema or UDF change is detected
    /// and the server should be reloaded
    Reload(PathBuf),
    /// emitted when the bridge is ready to receive requests
    BridgeReady,
    /// emitted when the proxy server has a startup error
    ProxyError,
    /// emitted when an SDL schema produced from the TS config has been written. Contains the file
    /// path where it was written.
    NewSdlFromTsConfig(Box<Path>),
}

impl Event {
    pub fn should_restart_servers(&self) -> bool {
        match self {
            Self::Reload(_) | Self::ProxyError => true,
            Self::BridgeReady | Self::NewSdlFromTsConfig(_) => false,
        }
    }
}

/// returns a future that resolves when given event is sent
#[allow(clippy::module_name_repetitions)]
pub async fn wait_for_event_and_match<F, O>(mut receiver: Receiver<Event>, f: F) -> O
where
    F: Fn(Event) -> Option<O>,
{
    loop {
        if let Ok(value) = receiver.recv().await {
            if let Some(result) = f(value) {
                break result;
            }
        }
    }
}

/// returns a future that resolves when given event is sent
#[allow(clippy::module_name_repetitions)]
pub async fn wait_for_event<F>(receiver: Receiver<Event>, f: F)
where
    F: Fn(&Event) -> bool,
{
    wait_for_event_and_match(receiver, |event| if f(&event) { Some(()) } else { None }).await;
}
