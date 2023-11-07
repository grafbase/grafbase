#![deny(unsafe_code, missing_docs, rust_2018_idioms)]

//! GraphQL schema composition.

mod compose;
mod composition_ir;
mod diagnostics;
mod emit_federated_graph;
mod ingest_subgraph;
mod result;
mod subgraphs;

pub use self::{diagnostics::Diagnostics, result::CompositionResult, subgraphs::Subgraphs};
pub use grafbase_federated_graph::render_sdl;

use self::{
    compose::{compose_subgraphs, ComposeContext},
    emit_federated_graph::emit_federated_graph,
};

/// Compose subgraphs into a federated graph.
pub fn compose(subgraphs: &Subgraphs) -> CompositionResult {
    let mut diagnostics = Diagnostics::default();
    let mut context = ComposeContext::new(subgraphs, &mut diagnostics);

    compose_subgraphs(&mut context);

    if context.diagnostics.any_fatal() {
        return CompositionResult {
            diagnostics,
            federated_graph: Default::default(),
        };
    }

    let federated_graph = emit_federated_graph(context.into_ir(), subgraphs);

    CompositionResult {
        federated_graph,
        diagnostics,
    }
}

trait VecExt<T> {
    fn push_return_idx(&mut self, elem: T) -> usize;
}

impl<T> VecExt<T> for Vec<T> {
    fn push_return_idx(&mut self, elem: T) -> usize {
        let idx = self.len();
        self.push(elem);
        idx
    }
}
