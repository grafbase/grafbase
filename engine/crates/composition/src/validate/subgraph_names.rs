use super::ValidateContext;
use std::collections::HashSet;

pub(crate) fn validate_subgraph_names(ctx: &mut ValidateContext<'_>) {
    let mut seen = HashSet::new();

    for subgraph in ctx.subgraphs.iter_subgraphs() {
        let name = subgraph.name().as_str();
        validate_name(name, ctx);

        if !seen.insert(name.to_ascii_lowercase()) {
            ctx.diagnostics.push_fatal(format!(
                r#"Found two subgraphs named "{name}". Subgraph names are case insensitive. "#
            ));
        }
    }
}

fn validate_name(name: &str, ctx: &mut ValidateContext<'_>) {
    let mut chars = name.chars();

    let Some(first) = chars.next() else {
        ctx.diagnostics
            .push_fatal("The empty string is not a valid subgraph name".to_owned());
        return;
    };

    let first_char_is_ok = first.is_alphabetic();
    let other_chars_are_ok = chars.all(|char| char == '-' || char.is_alphanumeric());

    if first_char_is_ok && other_chars_are_ok {
        return;
    }

    ctx.diagnostics.push_fatal(format!(
        r#"Invalid subgraph name: "{name}". Only alphanumeric characters and hyphens (`-`) are allowed, and the first character must be alphabetic."#
    ));
}
