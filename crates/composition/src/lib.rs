#![deny(unsafe_code, missing_docs, rust_2018_idioms)]

//! GraphQL schema composition.

mod compose_supergraph;
mod context;
mod diagnostics;
mod ingest_subgraph;
mod result;
mod strings;
mod subgraphs;
mod supergraph;

pub use self::{diagnostics::Diagnostics, result::CompositionResult, subgraphs::Subgraphs};

use self::{context::Context, strings::StringId, supergraph::Supergraph};

/// Compose subgraphs into a supergraph
pub fn compose(subgraphs: &Subgraphs) -> CompositionResult {
    let mut context = Context {
        subgraphs,
        supergraph: Supergraph::default(),
        diagnostics: Diagnostics::default(),
    };

    compose_supergraph::build_supergraph(&mut context);

    let supergraph_sdl = context.supergraph.render(&context.subgraphs.strings);

    CompositionResult {
        supergraph_sdl,
        diagnostics: context.diagnostics,
    }
}
