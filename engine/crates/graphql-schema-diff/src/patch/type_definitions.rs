use cynic_parser::type_system::TypeDefinition;

use crate::ChangeKind;

use super::paths::Paths;

pub(super) fn patch_type_definition<T: AsRef<str>>(ty: TypeDefinition<'_>, schema: &mut String, paths: &Paths<'_, T>) {
    for change in paths.iter_exact([ty.name(), "", ""]) {
        match change.kind() {
            ChangeKind::RemoveObjectType
            | ChangeKind::RemoveUnion
            | ChangeKind::RemoveEnum
            | ChangeKind::RemoveScalar
            | ChangeKind::RemoveInterface
            | ChangeKind::RemoveInputObject => return,
            ChangeKind::RemoveInterfaceImplementation => todo!(),
            kind => {
                debug_assert!(false, "Unhandled change at `{path}`: {kind:?}", path = change.path())
            }
        }
    }

    if let Some(description) = ty.description() {
        let span = description.span();
        schema.push_str(&paths.source()[span.start..span.end]);
        schema.push('\n');
    }

    let span = ty.span();
    schema.push_str(&paths.source()[span.start..span.end]);
    schema.push_str("\n\n");
}
