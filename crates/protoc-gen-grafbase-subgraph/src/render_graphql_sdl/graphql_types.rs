use crate::schema::ScalarType;

use super::*;

pub(super) fn render_graphql_types(
    schema: &GrpcSchema,
    types_to_render: &services::TypesToRender,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    let services::TypesToRender {
        messages_to_render_as_input,
        messages_to_render_as_output,
        enums_to_render,
    } = types_to_render;

    f.write_str("\"64 bit signed integer\" scalar I64\n")?;
    f.write_str("\"64 bit unsigned integer\" scalar U64\n")?;

    for message_id in messages_to_render_as_input {
        render_message(schema, *message_id, true, f)?;
    }

    for message_id in messages_to_render_as_output {
        render_message(schema, *message_id, false, f)?;
    }

    for enum_id in enums_to_render {
        render_enum_definition(schema, *enum_id, f)?;
    }

    Ok(())
}

fn render_message(
    schema: &GrpcSchema,
    message_id: ProtoMessageId,
    input: bool,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    let message = schema.view(message_id);

    match message.name.as_str() {
        ".google.protobuf.Duration"
        | ".google.protobuf.Timestamp"
        | ".google.protobuf.FieldMask"
        | ".google.protobuf.BoolValue"
        | ".google.protobuf.BytesValue"
        | ".google.protobuf.DoubleValue"
        | ".google.protobuf.FloatValue"
        | ".google.protobuf.Int32Value"
        | ".google.protobuf.Int64Value"
        | ".google.protobuf.StringValue"
        | ".google.protobuf.UInt32Value"
        | ".google.protobuf.UInt64Value" => return Ok(()),

        ".google.protobuf.Empty" => f.write_str("\n\"An empty object \" scalar EmptyObject\n")?,

        _ => (),
    }

    f.write_str("\n")?;

    if let Some(description) = message.description.as_deref() {
        render_description(f, description)?;
    }

    if input {
        writeln!(f, "input {} {{", message.graphql_input_name())?;
    } else {
        writeln!(f, "type {} {{", message.graphql_output_name())?;
    }

    for field in message_id.fields(schema) {
        if let Some(description) = field.description.as_deref() {
            render_description(f, description)?;
        }

        f.write_str(INDENT)?;
        f.write_str(&field.name)?;
        f.write_str(": ")?;

        if input {
            render_input_field_type(schema, &field.r#type, field.repeated, f)?;
        } else {
            render_output_field_type(schema, &field.r#type, field.repeated, f)?;
        }

        f.write_str("\n")?;
    }

    f.write_str("}\n")
}

pub(super) fn render_output_field_type(
    schema: &GrpcSchema,
    ty: &FieldType,
    repeated: bool,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    if repeated {
        f.write_str("[")?;
    }

    match ty {
        FieldType::Scalar(scalar_type) => scalar_type.render_graphql_type(f)?,
        FieldType::Message(proto_message_id) => render_message_type_name(schema, *proto_message_id, false, f)?,
        FieldType::Enum(proto_enum_id) => schema.view(*proto_enum_id).graphql_name().fmt(f)?,
    }

    if repeated {
        f.write_str("!]")?;
    }

    Ok(())
}

pub(super) fn render_input_field_type(
    schema: &GrpcSchema,
    ty: &FieldType,
    repeated: bool,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    if repeated {
        f.write_str("[")?;
    }

    match ty {
        FieldType::Scalar(scalar_type) => scalar_type.render_graphql_type(f)?,
        FieldType::Message(proto_message_id) => render_message_type_name(schema, *proto_message_id, true, f)?,
        FieldType::Enum(proto_enum_id) => schema.view(*proto_enum_id).graphql_name().fmt(f)?,
    }

    if repeated {
        f.write_str("!]")?;
    }

    Ok(())
}

fn render_enum_definition(schema: &GrpcSchema, enum_id: ProtoEnumId, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let enum_type = schema.view(enum_id);

    f.write_str("\n")?;

    if let Some(description) = &enum_type.description {
        render_description(f, description)?;
    }

    writeln!(f, "enum {} {{", enum_type.graphql_name())?;

    for value in &enum_type.values {
        if let Some(description) = value.description.as_deref() {
            render_description(f, description)?;
        }

        f.write_str(INDENT)?;
        f.write_str(value.name.as_str())?;
        f.write_str(",\n")?;
    }

    f.write_str("}\n")
}

pub(super) fn render_description(f: &mut fmt::Formatter<'_>, description: &str) -> fmt::Result {
    writeln!(f, "\"\"\"\n{description}\n\"\"\"", description = description.trim())
}

fn render_message_type_name(
    schema: &GrpcSchema,
    proto_message_id: ProtoMessageId,
    is_input: bool,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    let message = schema.view(proto_message_id);

    // See the docs for the mapping of well-known types: https://protobuf.dev/programming-guides/json/
    match message.name.as_str() {
        ".google.protobuf.Duration" => f.write_str("String"),
        ".google.protobuf.Timestamp" => f.write_str("String"),
        ".google.protobuf.FieldMask" => f.write_str("String"),
        ".google.protobuf.Empty" => f.write_str("EmptyObject"),
        ".google.protobuf.BoolValue" => ScalarType::Bool.render_graphql_type(f),
        ".google.protobuf.BytesValue" => ScalarType::Bytes.render_graphql_type(f),
        ".google.protobuf.DoubleValue" => ScalarType::Double.render_graphql_type(f),
        ".google.protobuf.FloatValue" => ScalarType::Float.render_graphql_type(f),
        ".google.protobuf.Int32Value" => ScalarType::Int32.render_graphql_type(f),
        ".google.protobuf.Int64Value" => ScalarType::Int64.render_graphql_type(f),
        ".google.protobuf.StringValue" => ScalarType::String.render_graphql_type(f),
        ".google.protobuf.UInt32Value" => ScalarType::UInt32.render_graphql_type(f),
        ".google.protobuf.UInt64Value" => ScalarType::UInt64.render_graphql_type(f),

        _ => {
            if is_input {
                message.graphql_input_name().fmt(f)
            } else {
                message.graphql_output_name().fmt(f)
            }
        }
    }
}
