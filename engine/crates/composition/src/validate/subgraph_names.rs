pub(crate) fn validate_subgraph_names(ctx: &mut crate::compose::ComposeContext<'_>) {
    for subgraph in ctx.subgraphs.iter_subgraphs() {
        let name = subgraph.name().as_str();
        let mut chars = name.chars();

        let Some(first) = chars.next() else {
            ctx.diagnostics
                .push_fatal("The empty string is not a valid subgraph name".to_owned());
            continue;
        };

        let first_char_is_ok = first.is_alphabetic();
        let other_chars_are_ok = chars.all(|char| char == '-' || char.is_alphanumeric());

        if first_char_is_ok && other_chars_are_ok {
            continue;
        }

        ctx.diagnostics.push_fatal(format!(
            r#"Invalid subgraph name: "{name}". Only alphanumeric characters and hyphens (`-`) are allowed, and the first character must be alphabetic."#
        ));
    }
}
