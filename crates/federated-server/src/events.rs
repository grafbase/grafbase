use std::sync::Arc;

use gateway_config::Config;

use crate::graph::Graph;

/// Represents all possible events that can trigger an engine reload.
/// This unified event type simplifies the data flow by consolidating
/// all update sources into a single stream.
#[derive(Clone)]
pub(crate) enum UpdateEvent {
    /// A graph definition update event
    Graph(Graph),
    /// A configuration update event
    Config(Arc<Config>),
}

impl std::fmt::Display for UpdateEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Graph(_) => write!(f, "Graph update"),
            Self::Config(_) => write!(f, "Config update"),
        }
    }
}
