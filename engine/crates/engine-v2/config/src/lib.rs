use federated_graph::FederatedGraphV1;

// The specific version modules should be kept private, users of this crate
// should only access types via `latest`
mod v2;

/// The latest version of the configuration.
///
/// Users of the crate should always use this verison, and we can keep the details
/// of older versions isolated in this crate.
pub mod latest {
    // If you introduce a new version you should update this export to the latest
    pub use super::v2::*;
}

/// Configuration for engine-v2
///
/// This made up of a FederatedGraph and any additional configuration required by
/// engine-v2.
///
/// It's serialised and stored as JSON so we need to maintain backwards compatability
/// when making changes (or introduce a new version).
#[derive(serde::Serialize, serde::Deserialize)]
pub enum VersionedConfig {
    /// The initial version of our configuration only contained the FederatedGraph.
    V1(FederatedGraphV1),
    /// V2 introduced some other configuration concerns
    V2(v2::Config),
}

impl VersionedConfig {
    /// Converts a config of any version into whatever the latest version is.
    pub fn into_latest(self) -> latest::Config {
        match self {
            VersionedConfig::V1(graph) => v2::Config {
                graph,
                strings: Default::default(),
                headers: Default::default(),
                default_headers: Default::default(),
                subgraph_configs: Default::default(),
                cache: Default::default(),
                auth: None,
                operation_limits: Default::default(),
            },
            VersionedConfig::V2(latest) => latest,
        }
    }
}
