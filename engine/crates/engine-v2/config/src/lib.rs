use federated_graph::FederatedGraphV1;

// The specific version modules should be kept private, users of this crate
// should only access types via `latest`
mod v2;
mod v3;
mod v4;

/// The latest version of the configuration.
///
/// Users of the crate should always use this verison, and we can keep the details
/// of older versions isolated in this crate.
pub mod latest {
    // If you introduce a new version you should update this export to the latest
    pub use super::v4::*;
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
    /// V3 is like V2 but with FederatedGraphV2
    V3(v3::Config),
    /// V4 is like V3 but with FederatedGraphV3
    V4(v4::Config),
}

impl VersionedConfig {
    /// Converts a config of any version into whatever the latest version is.
    pub fn into_latest(self) -> latest::Config {
        match self {
            VersionedConfig::V1(graph) => VersionedConfig::V2(v2::Config {
                graph,
                strings: Default::default(),
                headers: Default::default(),
                default_headers: Default::default(),
                subgraph_configs: Default::default(),
                cache: Default::default(),
                auth: None,
                operation_limits: Default::default(),
            })
            .into_latest(),

            VersionedConfig::V2(v2::Config {
                graph,
                strings,
                headers,
                default_headers,
                subgraph_configs,
                cache,
                auth,
                operation_limits,
            }) => VersionedConfig::V3(v3::Config {
                graph: graph.into(),
                strings,
                headers,
                default_headers,
                subgraph_configs,
                cache,
                auth,
                operation_limits,
                disable_introspection: Default::default(),
            })
            .into_latest(),

            VersionedConfig::V3(v3::Config {
                graph,
                strings,
                headers,
                default_headers,
                subgraph_configs,
                cache,
                auth,
                operation_limits,
                disable_introspection,
            }) => VersionedConfig::V4(v4::Config {
                graph: graph.into(),
                strings,
                headers,
                default_headers,
                subgraph_configs,
                cache,
                auth,
                operation_limits,
                disable_introspection,
            })
            .into_latest(),

            VersionedConfig::V4(latest) => latest,
        }
    }
}
