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

pub(in crate::patch) fn patch_directives<'a, T>(
    directives: impl Iterator<Item = Directive<'a>>,
    schema: &mut String,
    paths: &Paths<'_, T>,
) where
    T: AsRef<str>,
{
    // TODO: patching. Depends on thorough diffing implementation, which is missing.

    for directive in directives {
        render_directive(directive, schema, paths);
    }
}

fn render_directive<T: AsRef<str>>(directive: Directive<'_>, schema: &mut String, paths: &Paths<'_, T>) {
    schema.push_str(" @");
    schema.push_str(directive.name());

    let mut arguments = directive.arguments().peekable();

    if arguments.peek().is_none() {
        return;
    }

    schema.push('(');

    for argument in arguments {
        let span = argument.span();
        schema.push_str(&paths.source()[span.start..span.end])
    }

    schema.push(')');
}
