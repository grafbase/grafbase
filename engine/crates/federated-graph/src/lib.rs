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
pub enum FederatedGraph {
    V1(FederatedGraphV1),
    V2(FederatedGraphV2),
    V3(FederatedGraphV3),
    Sdl(String),
}

impl FederatedGraph {
    pub fn from_sdl(sdl: &str) -> Result<FederatedGraph, DomainError> {
        Ok(FederatedGraph::Sdl(sdl.to_owned()))
    }

    pub fn into_federated_sdl(self) -> String {
        match self {
            FederatedGraph::Sdl(sdl) => sdl,
            other => render_federated_sdl(&other.into_latest()).unwrap(),
        }
    }

    pub fn into_latest(self) -> FederatedGraphV4 {
        match self {
            FederatedGraph::V1(v1) => FederatedGraph::V2(FederatedGraphV2::from(v1)).into_latest(),
            FederatedGraph::V2(v2) => FederatedGraph::V3(FederatedGraphV3::from(v2)).into_latest(),
            FederatedGraph::V3(v3) => v3.into(),
            FederatedGraph::Sdl(sdl) => from_sdl(&sdl).unwrap(),
        }
    }
}
