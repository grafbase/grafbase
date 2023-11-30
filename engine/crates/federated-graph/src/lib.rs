mod federated_graph;

pub use self::federated_graph::*;

#[cfg(feature = "render_sdl")]
mod render_sdl;

#[cfg(feature = "render_sdl")]
pub use render_sdl::render_sdl;

#[cfg(feature = "from_sdl")]
mod from_sdl;

#[cfg(feature = "from_sdl")]
pub use from_sdl::from_sdl;

#[derive(serde::Serialize, serde::Deserialize)]
pub enum FederatedGraph {
    V1(FederatedGraphV1),
}
