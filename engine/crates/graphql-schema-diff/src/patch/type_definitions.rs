use cynic_parser::type_system::{
    Directive, EnumValueDefinition, FieldDefinition, InputValueDefinition, TypeDefinition,
};

use crate::ChangeKind;

use super::{paths::Paths, INDENTATION};

pub(super) fn patch_type_definition<T: AsRef<str>>(ty: TypeDefinition<'_>, schema: &mut String, paths: &Paths<'_, T>) {
    for change in paths.iter_exact([ty.name(), "", ""]) {
        match change.kind() {
            ChangeKind::RemoveObjectType
            | ChangeKind::RemoveUnion
            | ChangeKind::RemoveEnum
            | ChangeKind::RemoveScalar
            | ChangeKind::RemoveInterface
            | ChangeKind::RemoveInputObject => return,
            ChangeKind::AddInterfaceImplementation | ChangeKind::RemoveInterfaceImplementation => todo!(),
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

    let implements: Option<Vec<&str>> = match ty {
        TypeDefinition::Object(obj) => Some(obj.implements_interfaces().collect()),
        TypeDefinition::Interface(interface) => Some(interface.implements_interfaces().collect()),
        _ => None,
    };

    if let Some(implements) = implements.filter(|implements| !implements.is_empty()) {
        schema.push_str(" implements ");
        schema.push_str(&implements.join(" & "));
    }

    for directive in ty.directives() {
        render_directive(directive, schema, paths);
    }

    match ty {
        TypeDefinition::Scalar(_) => (),
        TypeDefinition::Object(object) => patch_fields(object.fields(), ty.name(), schema, paths),
        TypeDefinition::Interface(interface) => patch_fields(interface.fields(), ty.name(), schema, paths),
        TypeDefinition::Union(_) => todo!(),
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

    schema.push_str("}");
}

fn patch_directives<'a, T>(directives: impl Iterator<Item = Directive<'a>>, schema: &mut String, paths: &Paths<'_, T>)
where
    T: AsRef<str>,
{
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

    while let Some(argument) = arguments.next() {
        let span = argument.span();
        schema.push_str(&paths.source()[span.start..span.end])
    }

    schema.push(')');
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
                    schema.push_str(" ");
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

        for directive in field.directives() {
            render_directive(directive, schema, paths);
        }

        schema.push('\n');
    }

    schema.push_str("}");
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

    schema.push_str("}");
}
