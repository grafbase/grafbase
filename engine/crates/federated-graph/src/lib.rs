mod federated_graph;

pub use self::federated_graph::*;

#[cfg(feature = "render_sdl")]
mod render_sdl;

#[cfg(feature = "render_sdl")]
pub use render_sdl::{render_api_sdl, render_federated_sdl, render_sdl};

#[cfg(feature = "from_sdl")]
mod from_sdl;

#[cfg(feature = "from_sdl")]
pub use from_sdl::{from_sdl, DomainError};

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum FederatedGraph {
    V1(FederatedGraphV1),
    V2(FederatedGraphV2),
    V3(FederatedGraphV3),
}

impl std::fmt::Debug for FederatedGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FederatedGraph").finish_non_exhaustive()
    }
}

impl FederatedGraph {
    pub fn into_sdl(self) -> Result<String, std::fmt::Error> {
        render_sdl(self)
    }

    #[deprecated(note = "Use into_sdl() instead")]
    pub fn to_sdl(&self) -> Result<String, std::fmt::Error> {
        self.clone().into_sdl()
    }

    pub fn from_sdl(sdl: &str) -> Result<FederatedGraph, DomainError> {
        from_sdl(sdl)
    }

    pub fn into_latest(self) -> FederatedGraphV3 {
        match self {
            FederatedGraph::V1(v1) => FederatedGraph::V2(FederatedGraphV2::from(v1)).into_latest(),
            FederatedGraph::V2(v2) => v2.into(),
            FederatedGraph::V3(v3) => v3,
        }
    }
}
