use super::display_utils::*;
use crate::{federated_graph::*, FederatedGraphV3};
use std::fmt::{self, Write as _};

/// Render a GraphQL SDL string for a federated graph. It does not include any
/// federation-specific directives, it only reflects the final API schema as visible
/// for consumers.
pub fn render_api_sdl(graph: &FederatedGraphV3) -> String {
    Renderer { graph }.to_string()
}

struct Renderer<'a> {
    graph: &'a FederatedGraphV3,
}

impl fmt::Display for Renderer<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Renderer { graph } = self;

        for r#enum in &graph.enums {
            f.write_str("enum ")?;
            f.write_str(&graph[r#enum.name])?;
            f.write_char(' ')?;
            write_block(f, |f| {
                for variant in &graph[r#enum.values] {
                    write_enum_variant(f, variant, graph)?;
                }

                Ok(())
            })?;
        }

        Ok(())
    }
}
