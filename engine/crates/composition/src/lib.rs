#![deny(unsafe_code, missing_docs, rust_2018_idioms)]
#![allow(clippy::option_option)]

//! GraphQL schema composition.

mod compose;
mod composition_ir;
mod diagnostics;
mod emit_federated_graph;
mod ingest_subgraph;
mod result;
mod subgraphs;

pub use self::{diagnostics::Diagnostics, result::CompositionResult, subgraphs::Subgraphs};
pub use graphql_federated_graph::{render_sdl, FederatedGraph};

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

#[cfg(test)]
mod inaccessible {
    use crate::Subgraphs;

    #[test]
    fn inaccessible() -> Result<(), String> {
        let sdl1 = r"
      type New {
        name: String! @inaccessible
        message: String!
        old: Old! @inaccessible
      }
      type Old @inaccessible {
        name: String!
      }
      type Query {
        getNew(name: String!): New  
      }
      ";
        let sdl2 = r"
      type New {
        other: String!
        name: String! @inaccessible
      }
        type Old {
            name: String! 
        }
      ";

        let mut subgraphs = Subgraphs::default();

        let parsed1 =
            async_graphql_parser::parse_schema(sdl1).map_err(|err| format!("Error parsing parsed1: {err}"))?;
        let parsed2 =
            async_graphql_parser::parse_schema(sdl2).map_err(|err| format!("Error parsing parsed2: {err}"))?;

        subgraphs.ingest(&parsed1, "parsed1", "http://example.com/parsed1");
        subgraphs.ingest(&parsed2, "parsed2", "http://example.com/parsed2");

        let actual = match crate::compose(&subgraphs).into_result() {
            Ok(sdl) => graphql_federated_graph::render_sdl(&sdl).unwrap(),
            Err(diagnostics) => format!(
                "{}\n",
                diagnostics
                    .iter_messages()
                    .map(|msg| format!("# {msg}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
        };

        println!("{actual}");

        Err(String::from("@"))
    }
}
