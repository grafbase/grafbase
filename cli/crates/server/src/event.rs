use std::path::PathBuf;
use tokio::sync::broadcast::Receiver;

/// server lifecycle related events
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// emitted when a schema change is detected
    /// and the server should be reloaded
    Reload(PathBuf),
    /// emitted when the bridge is ready to receive requests
    BridgeReady,
}

impl Event {
    pub fn should_restart_servers(&self) -> bool {
        match self {
            Self::Reload(_) => true,
            Self::BridgeReady => false,
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
