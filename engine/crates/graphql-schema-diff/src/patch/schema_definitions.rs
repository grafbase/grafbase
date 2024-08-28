use cynic_parser::type_system::SchemaDefinition;

use crate::ChangeKind;

use super::paths::Paths;

pub(super) fn patch_schema_definition<T: AsRef<str>>(
    definition: SchemaDefinition<'_>,
    schema: &mut String,
    paths: &Paths<'_, T>,
) {
    if paths
        .iter_exact([""; 3])
        .any(|change| matches!(change.kind(), ChangeKind::RemoveSchemaDefinition))
    {
        return;
    }

    let span = definition.span();
    schema.push_str(&paths.source()[span.start..span.end]);
    schema.push_str("\n\n");
}
