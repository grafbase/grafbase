#![deny(unsafe_code, missing_docs, rust_2018_idioms)]
#![allow(clippy::option_option)]
#![doc = include_str!("../README.md")]

mod compose;
mod composition_ir;
mod diagnostics;
mod emit_federated_graph;
mod ingest_subgraph;
mod result;
mod subgraphs;
mod validate;

pub use self::{diagnostics::Diagnostics, result::CompositionResult, subgraphs::Subgraphs};
pub use graphql_federated_graph::{render_api_sdl, render_federated_sdl, render_sdl, FederatedGraph};

use self::{
    compose::{compose_subgraphs, ComposeContext},
    emit_federated_graph::emit_federated_graph,
    ingest_subgraph::ast_value_to_subgraph_value,
};

/// Compose subgraphs into a federated graph.
pub fn compose(subgraphs: &Subgraphs) -> CompositionResult {
    let mut diagnostics = Diagnostics::default();

    if subgraphs.iter_subgraphs().len() == 0 {
        let error = "No graphs found for composition build. You must have at least one active graph.";
        diagnostics.push_fatal(error.to_owned());

        return CompositionResult {
            federated_graph: None,
            diagnostics,
        };
    }

    let mut context = ComposeContext::new(subgraphs, &mut diagnostics);

    validate::validate(&mut context);

    if context.diagnostics.any_fatal() {
        return CompositionResult {
            federated_graph: None,
            diagnostics,
        };
    }

    compose_subgraphs(&mut context);

    if context.diagnostics.any_fatal() {
        CompositionResult {
            federated_graph: None,
            diagnostics,
        }
    } else {
        let federated_graph = emit_federated_graph(context.into_ir(), subgraphs);

        CompositionResult {
            federated_graph: Some(federated_graph),
            diagnostics,
        }
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
