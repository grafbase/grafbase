use super::*;
use std::collections::HashSet;

pub(crate) fn validate_extension_names(ctx: &mut ValidateContext<'_>) {
    let extensions = ctx.subgraphs.iter_extensions();

    let mut seen = HashSet::with_capacity(extensions.len());

    for extension in extensions {
        let name = ctx.subgraphs.strings.resolve(extension.name);

        if !seen.insert(name.to_ascii_lowercase()) {
            ctx.diagnostics.push_fatal(format!(
                r#"Found two extensions named "{name}". Extension names are case insensitive."#
            ));
        }
    }
}
