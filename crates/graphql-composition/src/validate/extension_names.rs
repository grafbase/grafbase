use super::*;
use std::collections::HashMap;

pub(crate) fn validate_extension_names(ctx: &mut ValidateContext<'_>) {
    let extensions = ctx.subgraphs.iter_extensions();

    let mut seen = HashMap::with_capacity(extensions.len());

    for extension in extensions {
        let name = ctx.subgraphs.strings.resolve(extension.name);
        let link_url = ctx.subgraphs.strings.resolve(extension.url);
        let normalized_url = match link_url.rsplit_once('/') {
            Some((prefix, last_segment)) => {
                if last_segment
                    .trim_start_matches('v')
                    .chars()
                    .all(|c| c == '.' || c.is_ascii_digit())
                {
                    prefix
                } else {
                    link_url
                }
            }
            None => link_url,
        };

        let Some(existing) = seen.insert(name.to_ascii_lowercase(), normalized_url) else {
            continue;
        };

        if existing != normalized_url {
            ctx.diagnostics.push_fatal(format!(
                r#"Found multiple extensions named "{name}". The urls must match, but got "{existing}" and "{normalized_url}"."#,
            ));
        }
    }
}
