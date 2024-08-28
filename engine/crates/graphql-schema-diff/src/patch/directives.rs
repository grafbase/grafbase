use cynic_parser::type_system::DirectiveDefinition;

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
