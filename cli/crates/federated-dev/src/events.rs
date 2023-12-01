use std::sync::OnceLock;
use tokio::sync::broadcast;

fn channels() -> &'static (
    broadcast::Sender<FederatedDevEvent>,
    broadcast::Receiver<FederatedDevEvent>,
) {
    static EVENTS: OnceLock<(
        broadcast::Sender<FederatedDevEvent>,
        broadcast::Receiver<FederatedDevEvent>,
    )> = OnceLock::new();

    EVENTS.get_or_init(|| {
        let (sender, receiver) = broadcast::channel(16);
        (sender, receiver)
    })
}

pub(crate) fn emit_event(event: FederatedDevEvent) {
    channels().0.send(event).ok();
}

/// Subscribe to all events from the federated dev server. The receiver will start listening when
/// the function is called, events emitted previously will not be received.
pub fn subscribe() -> tokio::sync::broadcast::Receiver<FederatedDevEvent> {
    channels().1.resubscribe()
}

/// An event returned
#[derive(Debug, Clone)]
pub enum FederatedDevEvent {
    /// Composition worked for new schema.
    ComposeAfterAdditionSuccess {
        /// Which subgraph was added
        subgraph_name: String,
    },
    /// Composition failed for new schema.
    ComposeAfterAdditionFailure {
        /// Which subgraph was added
        subgraph_name: String,
    },
    /// Composition worked after removal
    ComposeAfterRemovalSuccess {
        /// Which subgraph was removed
        subgraph_name: String,
    },
    /// Composition failed after removal
    ComposeAfterRemovalFailure {
        /// Which subgraph was removed
        subgraph_name: String,
        /// The composition errors
        rendered_error: String,
    },
}
