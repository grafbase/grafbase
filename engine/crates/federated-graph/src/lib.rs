use grafbase_workspace_hack as _;

mod federated_graph;

pub use self::federated_graph::*;

#[cfg(feature = "render_sdl")]
mod render_sdl;

#[cfg(feature = "render_sdl")]
pub use render_sdl::{render_api_sdl, render_federated_sdl};

#[cfg(feature = "from_sdl")]
mod from_sdl;

#[cfg(feature = "from_sdl")]
pub use from_sdl::{from_sdl, DomainError};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum VersionedFederatedGraph {
    V1(FederatedGraphV1),
    V2(FederatedGraphV2),
    V3(FederatedGraphV3),
    Sdl(String),
}

impl VersionedFederatedGraph {
    pub fn from_sdl(sdl: &str) -> Result<VersionedFederatedGraph, DomainError> {
        Ok(VersionedFederatedGraph::Sdl(sdl.to_owned()))
    }

    pub fn into_federated_sdl(self) -> String {
        match self {
            VersionedFederatedGraph::Sdl(sdl) => sdl,
            other => render_federated_sdl(&other.into_latest()).unwrap(),
        }
    }

    pub fn into_latest(self) -> FederatedGraph {
        match self {
            VersionedFederatedGraph::V1(v1) => VersionedFederatedGraph::V2(FederatedGraphV2::from(v1)).into_latest(),
            VersionedFederatedGraph::V2(v2) => VersionedFederatedGraph::V3(FederatedGraphV3::from(v2)).into_latest(),
            VersionedFederatedGraph::V3(v3) => v3.into(),
            VersionedFederatedGraph::Sdl(sdl) => from_sdl(&sdl).unwrap(),
        }
    }
}
