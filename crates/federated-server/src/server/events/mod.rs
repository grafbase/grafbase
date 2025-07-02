use super::gateway::GraphDefinition;
use gateway_config::Config;

/// Represents all possible events that can trigger an engine reload.
/// This unified event type simplifies the data flow by consolidating
/// all update sources into a single stream.
#[derive(Clone)]
pub enum UpdateEvent {
    /// A graph definition update event
    Graph(GraphDefinition),
    /// A configuration update event
    Config(Box<Config>),
}

#[allow(dead_code)]
impl UpdateEvent {
    /// Creates a new graph update event
    pub fn graph(definition: GraphDefinition) -> Self {
        Self::Graph(definition)
    }

    /// Creates a new config update event
    pub fn config(config: Config) -> Self {
        Self::Config(Box::new(config))
    }

    /// Returns true if this is a graph update event
    pub fn is_graph(&self) -> bool {
        matches!(self, Self::Graph(_))
    }

    /// Returns true if this is a config update event
    pub fn is_config(&self) -> bool {
        matches!(self, Self::Config(_))
    }

    /// Returns the graph definition if this is a graph update event
    pub fn as_graph(&self) -> Option<&GraphDefinition> {
        match self {
            Self::Graph(def) => Some(def),
            _ => None,
        }
    }

    /// Returns the config if this is a config update event
    pub fn as_config(&self) -> Option<&Config> {
        match self {
            Self::Config(config) => Some(config),
            _ => None,
        }
    }
}

impl std::fmt::Display for UpdateEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Graph(_) => write!(f, "Graph update"),
            Self::Config(_) => write!(f, "Config update"),
        }
    }
}
