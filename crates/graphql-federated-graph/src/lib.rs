//! A structured representation of a federated GraphQL schema. Can be instantiated by [composition](https://crates.io/crates/graphql-composition) or [from SDL](`from_sdl()`).

pub mod directives;

mod federated_graph;
mod from_sdl;
mod render_sdl;

pub use self::federated_graph::*;
pub use directives::*;

#[cfg(feature = "render_sdl")]
pub use render_sdl::{display_graphql_string_literal, render_api_sdl, render_federated_sdl};

#[cfg(feature = "from_sdl")]
pub use from_sdl::DomainError;
