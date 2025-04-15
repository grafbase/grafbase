use super::*;

pub(super) fn render_schema_directives(schema: &GrpcSchema, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("extend schema\n  @link(url: \"https://grafbase.com/extensions/grpc/0.1.0\", import: [\"@protoServices\", \"@protoEnums\", \"@protoMessages\", \"@grpcMethod\"])\n")?;

    render_proto_services(schema, f)?;
    render_proto_messages(schema, f)?;
    render_proto_enums(schema, f)?;

    f.write_str("\n")
}

fn render_proto_services(schema: &GrpcSchema, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if schema.services.is_empty() {
        return Ok(());
    }

    f.write_str(INDENT)?;
    f.write_str("@protoServices(\n")?;
    f.write_str(INDENT)?;
    f.write_str(INDENT)?;
    f.write_str("services: [\n")?;

    for service in schema.iter_services() {
        writeln!(f, "{INDENT}{INDENT}{INDENT}{{")?;
        writeln!(f, "{INDENT}{INDENT}{INDENT}{INDENT}name: \"{}\"", service.name)?;
        writeln!(f, "{INDENT}{INDENT}{INDENT}{INDENT}methods: [")?;

        for method in service.id.methods(schema) {
            writeln!(f, "{INDENT}{INDENT}{INDENT}{INDENT}{INDENT}{{")?;
            writeln!(
                f,
                "{INDENT}{INDENT}{INDENT}{INDENT}{INDENT}{INDENT}name: \"{}\"",
                method.name
            )?;
            writeln!(
                f,
                "{INDENT}{INDENT}{INDENT}{INDENT}{INDENT}{INDENT}inputType: \"{}\"",
                method.input_type.proto_name(schema)
            )?;
            writeln!(
                f,
                "{INDENT}{INDENT}{INDENT}{INDENT}{INDENT}{INDENT}outputType: \"{}\"",
                method.output_type.proto_name(schema)
            )?;

            if method.server_streaming {
                writeln!(
                    f,
                    "{INDENT}{INDENT}{INDENT}{INDENT}{INDENT}{INDENT}serverStreaming: true",
                )?;
            }
            writeln!(f, "{INDENT}{INDENT}{INDENT}{INDENT}{INDENT}}}")?;
        }

        writeln!(f, "{INDENT}{INDENT}{INDENT}{INDENT}]")?;
        writeln!(f, "{INDENT}{INDENT}{INDENT}}}")?;
    }

    f.write_str(INDENT)?;
    f.write_str(INDENT)?;
    f.write_str("]\n")?;

    f.write_str(INDENT)?;
    f.write_str(")\n")?;

    Ok(())
}

fn render_proto_messages(schema: &GrpcSchema, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if schema.messages.is_empty() {
        return Ok(());
    }

    f.write_str(INDENT)?;
    f.write_str("@protoMessages(\n")?;
    f.write_str(INDENT)?;
    f.write_str(INDENT)?;
    f.write_str("messages: [\n")?;

    for message in schema.iter_messages() {
        writeln!(f, "{INDENT}{INDENT}{INDENT}{{")?;
        writeln!(f, "{INDENT}{INDENT}{INDENT}{INDENT}name: \"{}\"", message.name)?;
        writeln!(f, "{INDENT}{INDENT}{INDENT}{INDENT}fields: [")?;

        for field in message.id.fields(schema) {
            writeln!(f, "{INDENT}{INDENT}{INDENT}{INDENT}{INDENT}{{")?;
            writeln!(
                f,
                "{INDENT}{INDENT}{INDENT}{INDENT}{INDENT}{INDENT}name: \"{}\"",
                field.name
            )?;
            writeln!(
                f,
                "{INDENT}{INDENT}{INDENT}{INDENT}{INDENT}{INDENT}number: \"{}\"",
                field.number,
            )?;
            writeln!(
                f,
                "{INDENT}{INDENT}{INDENT}{INDENT}{INDENT}{INDENT}repeated: \"{}\"",
                field.repeated,
            )?;
            writeln!(
                f,
                "{INDENT}{INDENT}{INDENT}{INDENT}{INDENT}{INDENT}type: \"{}\"",
                field.r#type.proto_name(schema)
            )?;

            writeln!(f, "{INDENT}{INDENT}{INDENT}{INDENT}{INDENT}}}")?;
        }

        writeln!(f, "{INDENT}{INDENT}{INDENT}{INDENT}]")?;
        writeln!(f, "{INDENT}{INDENT}{INDENT}}}")?;
    }

    f.write_str(INDENT)?;
    f.write_str(INDENT)?;
    f.write_str("]\n")?;

    f.write_str(INDENT)?;
    f.write_str(")\n")?;

    Ok(())
}

fn render_proto_enums(schema: &GrpcSchema, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if schema.enums.is_empty() {
        return Ok(());
    }

    f.write_str(INDENT)?;
    f.write_str("@protoEnums(\n")?;
    f.write_str(INDENT)?;
    f.write_str(INDENT)?;
    f.write_str("enums: [\n")?;

    for enum_ in schema.iter_enums() {
        writeln!(f, "{INDENT}{INDENT}{INDENT}{{")?;
        writeln!(f, "{INDENT}{INDENT}{INDENT}{INDENT}name: \"{}\"", enum_.name)?;
        writeln!(f, "{INDENT}{INDENT}{INDENT}{INDENT}values: [")?;

        for value in &enum_.values {
            writeln!(f, "{INDENT}{INDENT}{INDENT}{INDENT}{INDENT}{{")?;
            writeln!(
                f,
                "{INDENT}{INDENT}{INDENT}{INDENT}{INDENT}{INDENT}name: \"{}\"",
                value.name
            )?;
            writeln!(
                f,
                "{INDENT}{INDENT}{INDENT}{INDENT}{INDENT}{INDENT}number: \"{}\"",
                value.number,
            )?;

            writeln!(f, "{INDENT}{INDENT}{INDENT}{INDENT}{INDENT}}}")?;
        }

        writeln!(f, "{INDENT}{INDENT}{INDENT}{INDENT}]")?;
        writeln!(f, "{INDENT}{INDENT}{INDENT}}}")?;
    }

    f.write_str(INDENT)?;
    f.write_str(INDENT)?;
    f.write_str("]\n")?;

    f.write_str(INDENT)?;
    f.write_str(")\n")?;

    Ok(())
}
