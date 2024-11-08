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
    Sdl(String),
}

impl VersionedFederatedGraph {
    pub fn from_sdl(sdl: &str) -> Result<VersionedFederatedGraph, DomainError> {
        Ok(VersionedFederatedGraph::Sdl(sdl.to_owned()))
    }

    pub fn into_federated_sdl(self) -> Result<String, DomainError> {
        Ok(match self {
            VersionedFederatedGraph::Sdl(sdl) => sdl,
        })
    }

    pub fn into_latest(self) -> Result<FederatedGraph, DomainError> {
        Ok(match self {
            VersionedFederatedGraph::Sdl(sdl) => from_sdl(&sdl)?,
        })
    }
}
