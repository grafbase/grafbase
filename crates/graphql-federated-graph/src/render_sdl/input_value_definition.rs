use super::{
    directive::write_directive,
    display_utils::{INDENT, ValueDisplay, render_field_type, write_description},
};
use crate::{Directive, FederatedGraph, InputValueDefinition};
use std::fmt;

pub(crate) fn display_input_value_definition(
    input_value_definition: &InputValueDefinition,
    graph: &FederatedGraph,
    directives_filter: fn(&Directive, graph: &FederatedGraph) -> bool,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    write_description(f, input_value_definition.description, INDENT, graph)?;
    let input_value_definition_name = &graph[input_value_definition.name];
    f.write_str(input_value_definition_name)?;
    f.write_str(": ")?;
    f.write_str(&render_field_type(&input_value_definition.r#type, graph))?;

    if let Some(default) = &input_value_definition.default {
        write!(f, " = {}", ValueDisplay(default, graph))?;
    }

    let mut filtered_directives = input_value_definition
        .directives
        .iter()
        .filter(|directive| directives_filter(directive, graph))
        .peekable();

    if filtered_directives.peek().is_some() {
        for directive in filtered_directives {
            f.write_str(" ")?;
            write_directive(f, directive, graph)?;
        }
    }

    Ok(())
}
