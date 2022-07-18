use tokio::sync::broadcast::Receiver;

/// server lifecycle related events
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// emitted when a schema change is detected
    /// and the server should be reloaded
    Reload,
    /// emitted when the bridge is ready to receive requests
    BridgeReady,
}

/// returns a future that resolves when given event is sent
#[allow(clippy::module_name_repetitions)]
pub async fn wait_for_event(mut receiver: Receiver<Event>, event: Event) {
    loop {
        if let Ok(value) = receiver.recv().await {
            if value == event {
                break;
            }
        }
    }
}
