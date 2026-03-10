use cynic_parser::type_system::{Directive, DirectiveDefinition};

use crate::ChangeKind;

use super::paths::Paths;

pub(super) fn patch_directive_definition<T: AsRef<str>>(
    directive_definition: DirectiveDefinition<'_>,
    schema: &mut String,
    paths: &Paths<'_, T>,
) {
    if paths
        .iter_exact([directive_definition.name(), "", ""])
        .any(|change| matches!(change.kind(), ChangeKind::RemoveDirectiveDefinition))
    {
        return;
    }

    let span = directive_definition.span();

    schema.push_str(&paths.source()[span.start..span.end]);
    schema.push_str("\n\n");
}

/// Patch directive usages on a type or field.
///
/// `field_name` is `None` for type-level directives, or `Some(name)` for field-level ones.
pub(in crate::patch) fn patch_directives<'a, T>(
    directives: impl Iterator<Item = Directive<'a>>,
    schema: &mut String,
    paths: &Paths<'_, T>,
    type_name: &str,
    field_name: Option<&str>,
) where
    T: AsRef<str>,
{
    let changes: Vec<_> = paths.directive_usage_changes_at(type_name, field_name).collect();

    // Collect removed directives as (name, per-name-index) pairs.
    let removed: Vec<(&str, usize)> = changes
        .iter()
        .filter(|c| matches!(c.kind(), ChangeKind::RemoveDirective))
        .filter_map(|c| parse_directive_name_and_index(c.path()))
        .collect();

    let mut per_name_idx: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for directive in directives {
        let name = directive.name();
        let idx = *per_name_idx.entry(name).or_default();
        per_name_idx.insert(name, idx + 1);

        if !removed.iter().any(|&(n, i)| n == name && i == idx) {
            render_directive(directive, schema, paths);
        }
    }

    // Append added directives.
    for change in changes.iter().filter(|c| matches!(c.kind(), ChangeKind::AddDirective)) {
        schema.push(' ');
        schema.push_str(change.resolved_str());
    }
}

/// Parse the directive name and per-name index from a change path like `Type.field.@name[idx]` or `Type.@name[idx]`.
fn parse_directive_name_and_index(path: &str) -> Option<(&str, usize)> {
    let segment = path.rsplit('.').next()?;
    let without_at = segment.strip_prefix('@')?;
    let bracket = without_at.rfind('[')?;
    let name = &without_at[..bracket];
    let idx: usize = without_at[bracket + 1..].strip_suffix(']')?.parse().ok()?;
    Some((name, idx))
}

fn render_directive<T: AsRef<str>>(directive: Directive<'_>, schema: &mut String, paths: &Paths<'_, T>) {
    schema.push_str(" @");
    schema.push_str(directive.name());

    let mut arguments = directive.arguments().peekable();

    if arguments.peek().is_none() {
        return;
    }

    schema.push('(');

    while let Some(argument) = arguments.next() {
        let span = argument.span();
        schema.push_str(&paths.source()[span.start..span.end]);

        if arguments.peek().is_some() {
            schema.push_str(", ");
        }
    }

    schema.push(')');
}
