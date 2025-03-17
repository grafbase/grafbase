use std::{
    collections::BTreeSet,
    fmt::{self, Display as _},
};

use heck::*;

use crate::translate_schema::{FieldType, ProtoEnumId, ProtoMessageId, TranslatedSchema};

const INDENT: &str = "  ";

pub(crate) fn render_graphql_sdl(schema: &TranslatedSchema, mut out: impl fmt::Write) -> fmt::Result {
    /// Lets you take a routine that expects a formatter, and use it on a Write sink.
    fn with_formatter<F>(mut out: impl fmt::Write, action: F) -> fmt::Result
    where
        F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
    {
        struct Helper<T>(T);

        impl<T> fmt::Display for Helper<T>
        where
            T: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                (self.0)(f)
            }
        }

        out.write_fmt(format_args!("{}", Helper(action)))
    }

    with_formatter(&mut out, |f| render_graphql_sdl_impl(schema, f))
}

fn render_graphql_sdl_impl(schema: &TranslatedSchema, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut messages_to_render_as_input = BTreeSet::new();
    let mut messages_to_render_as_output = BTreeSet::new();
    let mut enums_to_render = BTreeSet::new();

    for package in schema.iter_packages() {
        let Some(package_name) = package.package_name.as_deref() else {
            continue;
        };

        writeln!(
            f,
            "\n\"Namespace for the {} protobuf package\"\ntype {} {{",
            package_name,
            AsUpperCamelCase(package_name)
        )?;

        for service in package.id.services(schema) {
            writeln!(
                f,
                "{INDENT}{}: {}",
                service.name.to_lower_camel_case(),
                service.graphql_object_name(schema)
            )?;
        }

        f.write_str("}\n")?;
    }

    for service in schema.iter_services() {
        writeln!(
            f,
            "\n\"Namespace for the {} protobuf service\"\ntype {} {{",
            service.name,
            service.graphql_object_name(schema),
        )?;

        for method in service.id.methods(schema) {
            f.write_str(INDENT)?;
            AsLowerCamelCase(&method.name).fmt(f)?;
            f.write_str("(input: ")?;

            collect_message_id_and_enum_ids_recursively(
                schema,
                &method.input_type,
                &mut messages_to_render_as_input,
                &mut enums_to_render,
            );

            collect_message_id_and_enum_ids_recursively(
                schema,
                &method.output_type,
                &mut messages_to_render_as_output,
                &mut enums_to_render,
            );

            render_field_type(schema, &method.input_type, f)?;

            f.write_str("): ")?;

            render_field_type(schema, &method.output_type, f)?;

            writeln!(f, " @grpcMethod(name: \"{}\")", method.name)?;
        }

        f.write_str("}\n")?;
    }

    f.write_str("\ntype Mutation {\n")?;

    for package in schema.iter_packages() {
        match package.package_name.as_deref() {
            Some(package_name) => {
                writeln!(
                    f,
                    "{INDENT}{}: {}",
                    AsLowerCamelCase(package_name),
                    AsUpperCamelCase(package_name)
                )?;
            }
            None => {
                for service in package.id.services(schema) {
                    writeln!(
                        f,
                        "{INDENT}{}: {}",
                        AsLowerCamelCase(&service.name),
                        service.graphql_object_name(schema)
                    )?;
                }
            }
        }
    }

    f.write_str("}\n")?;

    for message in messages_to_render_as_output {
        render_message(schema, message, f)?;
    }

    for message in messages_to_render_as_input {
        render_message(schema, message, f)?;
    }

    Ok(())
}

fn collect_message_id_and_enum_ids_recursively(
    schema: &TranslatedSchema,
    field_type: &FieldType,
    message_ids: &mut BTreeSet<ProtoMessageId>,
    enum_ids: &mut BTreeSet<ProtoEnumId>,
) {
    match field_type {
        FieldType::Scalar(_scalar_type) => (),
        FieldType::Enum(proto_enum_id) => {
            enum_ids.insert(*proto_enum_id);
        }
        FieldType::Message(proto_message_id) => {
            message_ids.insert(*proto_message_id);

            for field in proto_message_id.fields(schema) {
                collect_message_id_and_enum_ids_recursively(schema, &field.r#type, message_ids, enum_ids);
            }
        }
        FieldType::Map(map) => {
            let (a, b) = map.as_ref();
            collect_message_id_and_enum_ids_recursively(schema, a, message_ids, enum_ids);
            collect_message_id_and_enum_ids_recursively(schema, b, message_ids, enum_ids);
        }
    }
}

fn render_message(schema: &TranslatedSchema, message_id: ProtoMessageId, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let message = schema.view(message_id);

    f.write_str("\ntype ")?;
    f.write_str(&message.name)?;
    f.write_str(" {\n")?;

    for field in message_id.fields(schema) {
        f.write_str(INDENT)?;
        f.write_str(&field.name)?;
        f.write_str(": ")?;
        render_field_type(schema, &field.r#type, f)?;

        writeln!(
            f,
            " @grpcField(number: {}, type: \"{}\")",
            field.number,
            field.r#type.proto_name(schema)
        )?;
    }

    f.write_str("}\n")
}

fn render_field_type(schema: &TranslatedSchema, ty: &FieldType, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match ty {
        FieldType::Scalar(scalar_type) => scalar_type.render_graphql_type(f),
        FieldType::Message(proto_message_id) => f.write_str(&schema.view(*proto_message_id).name),
        FieldType::Enum(proto_enum_id) => f.write_str(schema.view(*proto_enum_id).proto.name()),
        FieldType::Map(_) => todo!("render maps"),
    }
}
