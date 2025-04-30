use super::*;
use std::collections::HashSet;

pub(crate) fn validate_extension_names(ctx: &mut ValidateContext<'_>) {
    let extensions = ctx.subgraphs.iter_extensions();

    let mut seen = HashSet::with_capacity(extensions.len());

    for extension in extensions {
        let name = ctx.subgraphs.strings.resolve(extension.name);

        if !seen.insert(name.to_ascii_lowercase()) {
            if let Some(previous) = ctx
                .subgraphs
                .iter_extensions()
                .find(|ext| ctx[ext.name].eq_ignore_ascii_case(&ctx[extension.name]))
            {
                if extension_urls_are_compatible(&ctx[previous.url], &ctx[extension.url]) {
                    continue;
                } else {
                    ctx.diagnostics.push_fatal(format!(
                        r#"Found two extensions named "{name}". The urls must match, but got "{}" and "{}"."#,
                        &ctx[previous.url], &ctx[extension.url],
                    ));
                }
            } else {
                ctx.diagnostics.push_fatal(format!(
                    r#"Found two extensions named "{name}". Extension names are case insensitive."#
                ));
            }
        }
    }
}

fn extension_urls_are_compatible(a: &str, b: &str) -> bool {
    // If there is a path, it must be the same
    if a.starts_with("file:") || b.starts_with("file:") {
        return a == b;
    }

    // Otherwise, assume they're the same
    true
}
