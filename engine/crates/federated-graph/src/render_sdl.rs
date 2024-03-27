mod render_api_sdl;
mod render_federated_sdl;

pub use self::{render_api_sdl::render_api_sdl, render_federated_sdl::render_federated_sdl};

#[deprecated(since = "0.4.0", note = "Use render_federated_sdl() instead.")]
pub fn render_sdl(graph: crate::FederatedGraph) -> Result<String, std::fmt::Error> {
    render_federated_sdl(&graph.into_latest())
}
