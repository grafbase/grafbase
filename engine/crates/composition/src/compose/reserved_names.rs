use super::{ComposeContext, DefinitionWalker};

/// This is a reserved name.
const JOIN_GRAPH_ENUM_NAME: &str = "join__Graph";

/// Validates reserved names. Returns true in case of error.
pub(super) fn validate_definition_names(definitions: &[DefinitionWalker<'_>], ctx: &mut ComposeContext<'_>) -> bool {
    let Some(first) = definitions.first() else {
        return false;
    };

    if first.name().as_str() != JOIN_GRAPH_ENUM_NAME {
        return false;
    }

    for definition in definitions {
        ctx.diagnostics.push_fatal(format!(
            "[{}] Definition name `{}` is a reserved federation definition name, it cannot be defined in subgraphs.",
            definition.subgraph().name().as_str(),
            JOIN_GRAPH_ENUM_NAME
        ));
    }

    true
}
