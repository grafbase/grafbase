#![deny(unsafe_code, missing_docs, rust_2018_idioms)]
#![doc = include_str!("../README.md")]

pub mod diagnostics;

mod compose;
mod composition_ir;
mod emit_federated_graph;
mod federated_graph;
mod grafbase_extensions;
mod ingest_subgraph;
mod result;
mod subgraphs;
mod validate;

pub use self::{
    diagnostics::Diagnostics,
    federated_graph::{DomainError, FederatedGraph, render_api_sdl, render_federated_sdl},
    grafbase_extensions::LoadedExtension,
    result::CompositionResult,
    subgraphs::{IngestError, Subgraphs},
};

use self::{
    compose::{ComposeContext, compose_subgraphs},
    emit_federated_graph::emit_federated_graph,
    ingest_subgraph::ast_value_to_subgraph_value,
};

/// Compose subgraphs into a federated graph.
pub fn compose(subgraphs: Subgraphs) -> CompositionResult {
    let subgraphs = subgraphs.finalize();
    let mut diagnostics = Diagnostics::default();

    if subgraphs.iter_subgraphs().len() == 0 {
        let error = "No graphs found for composition build. You must have at least one active graph.";
        diagnostics.push_fatal(error.to_owned());

        return CompositionResult {
            federated_graph: None,
            diagnostics,
        };
    }

    let mut context = ComposeContext::new(&subgraphs, &mut diagnostics);

    validate::validate(&mut context);

    if context.diagnostics.any_fatal() {
        return CompositionResult {
            federated_graph: None,
            diagnostics,
        };
    }

    for (_, directive) in subgraphs.iter_extra_directives_on_schema_definition() {
        let subgraphs::DirectiveProvenance::Linked {
            linked_schema_id,
            is_composed_directive,
        } = directive.provenance
        else {
            continue;
        };

        if let Some(extension_id) = context.get_extension_for_linked_schema(linked_schema_id) {
            context.mark_used_extension(extension_id);
        } else if !is_composed_directive {
            context.diagnostics.push_warning(format!(
                "Directive `{}` is not defined in any extension or composed directive",
                &context[directive.name]
            ));
        }
    }
    compose_subgraphs(&mut context);

    if context.diagnostics.any_fatal() {
        CompositionResult {
            federated_graph: None,
            diagnostics,
        }
    } else {
        let federated_graph = emit_federated_graph(context.into_ir(), &subgraphs);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grafbase_schema_can_be_composed() {
        use std::{fs, path::Path};
        let schema_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../cli/src/api/graphql/api.graphql");
        let schema = fs::read_to_string(schema_path).unwrap();

        let mut subgraphs = Subgraphs::default();
        subgraphs
            .ingest_str(&schema, "grafbase-api", Some("https://api.grafbase.com"))
            .unwrap();
        let result = compose(subgraphs);
        assert!(!result.diagnostics().any_fatal());
    }
}
