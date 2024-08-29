use cynic_parser::type_system::{
    EnumValueDefinition, FieldDefinition, InputValueDefinition, TypeDefinition, UnionMember,
};

use crate::ChangeKind;

use super::{directives::patch_directives, paths::Paths, INDENTATION};

pub(super) fn patch_type_definition<T: AsRef<str>>(ty: TypeDefinition<'_>, schema: &mut String, paths: &Paths<'_, T>) {
    for change in paths.iter_exact([ty.name(), "", ""]) {
        match change.kind() {
            ChangeKind::RemoveObjectType
            | ChangeKind::RemoveUnion
            | ChangeKind::RemoveEnum
            | ChangeKind::RemoveScalar
            | ChangeKind::RemoveInterface
            | ChangeKind::RemoveInputObject => return,
            kind => {
                debug_assert!(false, "Unhandled change at `{path}`: {kind:?}", path = change.path())
            }
        }
    }

    let mut added_interface_implementations = Vec::new();
    let mut removed_interface_implementations = Vec::new();

    for change in paths.iter_second_level(ty.name()) {
        match dbg!(change.kind()) {
            ChangeKind::AddInterfaceImplementation => {
                added_interface_implementations.push(
                    change
                        .second_level()
                        .expect("No interface name for AddInterfaceImplementation"),
                );
            }
            ChangeKind::RemoveInterfaceImplementation => {
                removed_interface_implementations.push(
                    change
                        .second_level()
                        .expect("No interface name for RemoveInterfaceImplementation"),
                );
            }
            _ => (),
        }
    }

    removed_interface_implementations.sort();

    if let Some(description) = ty.description() {
        let span = description.span();
        schema.push_str(&paths.source()[span.start..span.end]);
        schema.push('\n');
    }

    let prefix = match ty {
        TypeDefinition::Scalar(_) => "scalar",
        TypeDefinition::Object(_) => "type",
        TypeDefinition::Interface(_) => "interface",
        TypeDefinition::Union(_) => "union",
        TypeDefinition::Enum(_) => "enum",
        TypeDefinition::InputObject(_) => "input",
    };

    schema.push_str(prefix);
    schema.push(' ');
    schema.push_str(ty.name());

    let mut implements: Vec<&str> = match ty {
        TypeDefinition::Object(obj) => obj.implements_interfaces().collect(),
        TypeDefinition::Interface(interface) => interface.implements_interfaces().collect(),
        _ => Vec::new(),
    };

    dbg!(&implements);

    implements.extend(added_interface_implementations.into_iter());
    implements.retain(|name| removed_interface_implementations.binary_search(name).is_err());
    implements.sort();
    implements.dedup();
    dbg!(&implements);

    if !implements.is_empty() {
        schema.push_str(" implements ");
        schema.push_str(&implements.join(" & "));
    }

    patch_directives(ty.directives(), schema, paths);

    match ty {
        TypeDefinition::Scalar(_) => (),
        TypeDefinition::Object(object) => patch_fields(object.fields(), ty.name(), schema, paths),
        TypeDefinition::Interface(interface) => patch_fields(interface.fields(), ty.name(), schema, paths),
        TypeDefinition::Union(union) => patch_union(union.members(), ty.name(), schema, paths),
        TypeDefinition::Enum(r#enum) => patch_enum_values(r#enum.values(), ty.name(), schema, paths),
        TypeDefinition::InputObject(input_object) => {
            patch_input_object(input_object.fields(), ty.name(), schema, paths)
        }
    }

    schema.push_str("\n\n");
}

fn patch_input_object<'a, T: AsRef<str>>(
    fields: impl Iterator<Item = InputValueDefinition<'a>>,
    parent: &str,
    schema: &mut String,
    paths: &Paths<'a, T>,
) {
    schema.push_str(" {\n");

    let mut removed_fields = Vec::new();
    let mut changed_field_types = Vec::new();

    for change in paths.iter_second_level(parent) {
        match change.kind() {
            ChangeKind::ChangeFieldType => {
                let field_name = change.second_level().expect("ChangeFieldType without field name");
                changed_field_types.push((field_name, change.resolved_str()));
            }
            ChangeKind::RemoveField => {
                let field_name = change.second_level().expect("RemoveField without field name");
                removed_fields.push(field_name);
            }
            ChangeKind::AddField => {
                schema.push_str(INDENTATION);
                schema.push_str(change.resolved_str().trim());
                schema.push('\n');
            }
            kind => {
                debug_assert!(false, "Unhandled change at `{path}`: {kind:?}", path = change.path())
            }
        }
    }

    removed_fields.sort();

    for field in fields {
        if removed_fields.binary_search(&field.name()).is_ok() {
            continue;
        }

        schema.push_str(INDENTATION);
        schema.push_str(field.name());

        schema.push_str(": ");

        if let Ok(idx) = changed_field_types.binary_search_by(|(field_name, _)| field_name.cmp(&field.name())) {
            schema.push_str(changed_field_types[idx].1);
        } else {
            schema.push_str(&field.ty().to_string());
        }

        schema.push('\n');
    }

    schema.push('}');
}

fn patch_fields<'a, T>(
    fields: impl Iterator<Item = FieldDefinition<'a>>,
    parent: &str,
    schema: &mut String,
    paths: &Paths<'_, T>,
) where
    T: AsRef<str>,
{
    schema.push_str(" {\n");

    let mut changed_field_types = Vec::new();
    let mut removed_fields = Vec::new();

    for change in paths.iter_second_level(parent) {
        match change.kind() {
            ChangeKind::ChangeFieldType => {
                let field_name = change.second_level().expect("ChangeFieldType without field name");
                changed_field_types.push((field_name, change.resolved_str()));
            }
            ChangeKind::RemoveField => {
                let field_name = change.second_level().expect("RemoveField without field name");
                removed_fields.push(field_name);
            }
            ChangeKind::AddField => {
                schema.push_str(INDENTATION);
                schema.push_str(change.resolved_str().trim());
                schema.push('\n');
            }
            ChangeKind::AddInterfaceImplementation | ChangeKind::RemoveInterfaceImplementation => (), // already handled
            kind => {
                debug_assert!(false, "Unhandled change at `{path}`: {kind:?}", path = change.path())
            }
        }
    }

    removed_fields.sort();

    for field in fields {
        if removed_fields.binary_search(&field.name()).is_ok() {
            continue;
        }

        if let Some(description) = field.description() {
            let span = description.span();
            schema.push_str(INDENTATION);
            schema.push_str(&paths.source()[span.start..span.end]);
            schema.push('\n');
        }

        schema.push_str(INDENTATION);
        schema.push_str(field.name());

        let mut arguments = field.arguments().peekable();

        if arguments.peek().is_some() {
            schema.push('(');

            while let Some(argument) = arguments.next() {
                if let Some(description) = argument.description() {
                    let span = description.span();
                    schema.push_str(&paths.source()[span.start..span.end]);
                    schema.push(' ');
                }

                schema.push_str(argument.name());
                schema.push_str(": ");
                schema.push_str(&argument.ty().to_string());

                if argument.default_value().is_some() {
                    schema.push(' ');
                    let span = argument.default_value_span();
                    schema.push_str(&paths.source()[span.start..span.end]);
                }

                if arguments.peek().is_some() {
                    schema.push_str(", ");
                }
            }

            schema.push(')');
        }

        schema.push_str(": ");

        if let Ok(idx) = changed_field_types.binary_search_by(|(field_name, _)| field_name.cmp(&field.name())) {
            schema.push_str(changed_field_types[idx].1);
        } else {
            schema.push_str(&field.ty().to_string());
        }

        patch_directives(field.directives(), schema, paths);

        schema.push('\n');
    }

    schema.push('}');
}

fn patch_enum_values<'a, T>(
    values: impl Iterator<Item = EnumValueDefinition<'a>>,
    enum_name: &str,
    schema: &mut String,
    paths: &Paths<'a, T>,
) where
    T: AsRef<str>,
{
    let mut removed_enum_values = Vec::new();

    schema.push_str(" {\n");

    for change in paths.iter_second_level(enum_name) {
        match change.kind() {
            ChangeKind::AddEnumValue => {
                schema.push_str(INDENTATION);
                schema.push_str(change.resolved_str().trim());
                schema.push('\n');
            }
            ChangeKind::RemoveEnumValue => {
                let value = change.second_level().expect("RemoveEnumValue without value");
                removed_enum_values.push(value);
            }
            kind => {
                debug_assert!(false, "Unhandled change at `{path}`: {kind:?}", path = change.path())
            }
        }
    }

    removed_enum_values.sort();

    for value in values {
        if removed_enum_values.binary_search(&value.value()).is_ok() {
            continue;
        }

        schema.push_str(INDENTATION);
        schema.push_str(value.value());

        patch_directives(value.directives(), schema, paths);

        schema.push('\n');
    }

    schema.push('}');
}

fn patch_union<'a, T>(
    members: impl Iterator<Item = UnionMember<'a>>,
    union_name: &str,
    schema: &mut String,
    paths: &Paths<'a, T>,
) where
    T: AsRef<str>,
{
    let mut removed_members = Vec::new();
    let mut added_members = Vec::new();

    for change in paths.iter_second_level(union_name) {
        match change.kind() {
            ChangeKind::AddUnionMember => {
                added_members.push(change.second_level().expect("AddUnionMember without member name"))
            }
            ChangeKind::RemoveUnionMember => {
                removed_members.push(change.second_level().expect("RemoveUnionMember without member name"))
            }
            _ => (),
        }
    }

    removed_members.sort();

    let mut members = members
        .map(|member| member.name())
        .filter(|name| removed_members.binary_search(name).is_err())
        .chain(added_members.into_iter())
        .peekable();

    if members.peek().is_some() {
        schema.push_str(" = ");
    }

    while let Some(member) = members.next() {
        schema.push_str(member);

        if members.peek().is_some() {
            schema.push_str(" | ");
        }
    }
}
