use super::input_value_definition::display_input_value_definition;
use crate::{federated_graph::*, Directive, FederatedGraph};
use std::fmt;

pub(crate) fn display_directive_definitions(
    // filter the definitions themselves
    definitions_filter: fn(&DirectiveDefinition<'_>) -> bool,
    // filter the directives on the arguments of the definitions
    directives_filter: fn(&Directive, &FederatedGraph) -> bool,
    graph: &FederatedGraph,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    // (start, end) ranges for each directive definition's arguments
    let mut argument_ranges = Vec::with_capacity(graph.directive_definitions.len());
    let mut range_start = 0;

    for chunk in graph
        .directive_definition_arguments
        .chunk_by(|a, b| a.directive_definition_id == b.directive_definition_id)
    {
        let definition_idx: usize = chunk[0].directive_definition_id.into();

        // Fill the slots for directive definitions that have no arguments.
        if definition_idx > argument_ranges.len() {
            argument_ranges.resize(definition_idx, 0..0);
        }

        let range_end = range_start + chunk.len();

        argument_ranges.push(range_start..range_end);
        range_start = range_end;
    }

    for (idx, directive_definition) in graph.iter_directive_definitions().enumerate() {
        if !definitions_filter(&directive_definition) {
            continue;
        }

        let arguments = &graph.directive_definition_arguments[argument_ranges[idx].clone()];

        display_directive_definition(directive_definition, arguments, directives_filter, graph, f)?;
        f.write_str("\n")?;
    }

    Ok(())
}

fn display_directive_definition(
    directive_definition: DirectiveDefinition<'_>,
    arguments: &[DirectiveDefinitionArgument],
    directives_filter: fn(&Directive, graph: &FederatedGraph) -> bool,
    graph: &FederatedGraph,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    let namespace = directive_definition.namespace.map(|namespace| &graph[namespace]);

    f.write_str("directive @")?;

    if let Some(namespace) = namespace {
        f.write_str(namespace)?;
        f.write_str("__")?;
    }

    f.write_str(&graph[directive_definition.name])?;

    if !arguments.is_empty() {
        f.write_str("(")?;

        let mut arguments = arguments.iter().peekable();

        while let Some(argument) = arguments.next() {
            display_input_value_definition(&argument.input_value_definition, graph, directives_filter, f)?;

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
