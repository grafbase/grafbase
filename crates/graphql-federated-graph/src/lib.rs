use grafbase_workspace_hack as _;

pub mod directives;
mod federated_graph;

pub use self::federated_graph::*;
pub use directives::*;

#[cfg(feature = "render_sdl")]
mod render_sdl;

#[cfg(feature = "render_sdl")]
pub use render_sdl::{render_api_sdl, render_federated_sdl};

#[cfg(feature = "from_sdl")]
mod from_sdl;

#[cfg(feature = "from_sdl")]
pub use from_sdl::{from_sdl, DomainError};
