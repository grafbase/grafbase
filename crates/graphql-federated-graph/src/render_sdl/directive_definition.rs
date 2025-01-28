use super::input_value_definition::display_input_value_definition;
use crate::{federated_graph::*, Directive, FederatedGraph};
use std::fmt;

pub(crate) fn display_directive_definition(
    directive_definition: &DirectiveDefinition,
    directives_filter: fn(&Directive, graph: &FederatedGraph) -> bool,
    graph: &FederatedGraph,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    f.write_str("directive @")?;
    f.write_str(&graph[directive_definition.name])?;

    if !graph[directive_definition.arguments].is_empty() {
        f.write_str("(")?;

        let mut arguments = graph[directive_definition.arguments].iter().peekable();

        while let Some(argument) = arguments.next() {
            display_input_value_definition(argument, graph, directives_filter, f)?;

            if arguments.peek().is_some() {
                f.write_str(", ")?;
            }
        }

        f.write_str(")")?;
    }

    f.write_str(" on ")?;

    fmt::Display::fmt(&directive_definition.locations, f)?;

    f.write_str("\n")
}
