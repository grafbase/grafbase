use super::*;

/// This is a reserved name.
const JOIN_GRAPH_ENUM_NAME: &str = "join__Graph";

/// Validates reserved names. Returns true in case of error.
pub(super) fn validate_definition_names(
    definitions: &[subgraphs::DefinitionView<'_>],
    ctx: &mut ComposeContext<'_>,
) -> bool {
    let Some(first) = definitions.first() else {
        return false;
    };

    if ctx.subgraphs[first.name].as_ref() != JOIN_GRAPH_ENUM_NAME {
        return false;
    }

    for definition in definitions {
        ctx.diagnostics.push_fatal(format!(
            "[{}] Definition name `{}` is a reserved federation definition name, it cannot be defined in subgraphs.",
            ctx.subgraphs[ctx.subgraphs.at(definition.subgraph_id).name],
            JOIN_GRAPH_ENUM_NAME
        ));
    }

    true
}
