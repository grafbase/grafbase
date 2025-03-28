//! A structured representation of the types of a GraphQL schema generated from protobuf definitions. This data structure serves two purposes:
//!
//! - Link GraphQL types and the corresponding protobuf message types and RPC definitions.
//! - Serve as a source of truth that is rendered both to a GraphQL schema and to the generated extension's source code.

mod translated_schema;

pub(crate) use self::translated_schema::*;

use prost_types::{DescriptorProto, compiler::CodeGeneratorRequest, field_descriptor_proto};
use std::{collections::HashMap, str::FromStr as _};

/// Instantiates a new translated schema from protobuf definitions.
pub(super) fn translate_schema(code_generator_request: CodeGeneratorRequest) -> TranslatedSchema {
    let mut schema = TranslatedSchema::default();

    let mut messages_by_name: HashMap<String, ProtoMessageId> =
        HashMap::with_capacity(code_generator_request.proto_file.len() * 4);
    let mut enums_by_name: HashMap<String, ProtoEnumId> = HashMap::new();

    let mut local_messages_by_name: HashMap<String, ProtoMessageId> = HashMap::new();
    let mut local_enums_by_name: HashMap<String, ProtoEnumId> = HashMap::new();

    for proto_file in code_generator_request.proto_file {
        local_messages_by_name.clear();
        local_enums_by_name.clear();

        let package_name = proto_file.package;

        let package_id = schema.push_packages(ProtoPackage {
            package_name: package_name.clone(),
        });

        for r#enum in proto_file.enum_type {
            let enum_name = r#enum.name().to_owned();

            let enum_id = schema.push_enums(ProtoEnum {
                package_id,
                proto: r#enum,
            });

            let qualified_name = package_name
                .as_deref()
                .map(|package| format!(".{package}.{enum_name}"))
                .unwrap_or_else(|| enum_name.clone());

            enums_by_name.insert(qualified_name, enum_id);
            local_enums_by_name.insert(enum_name, enum_id);
        }

        let messages_len = schema.messages.len();

        // First resolve the names.
        for (index, message) in proto_file.message_type.iter().enumerate() {
            let message_id = (messages_len + index).into();

            let qualified_name = package_name
                .as_deref()
                .map(|package| format!(".{package}.{}", message.name().to_owned()))
                .unwrap_or_else(|| message.name().to_owned());

            local_messages_by_name.insert(message.name().to_owned(), message_id);

            messages_by_name.insert(qualified_name, message_id);
        }

        for message in proto_file.message_type {
            translate_message(
                &mut schema,
                message,
                package_id,
                &messages_by_name,
                &enums_by_name,
                &local_messages_by_name,
                &local_enums_by_name,
            );
        }

        for service in proto_file.service {
            let service_id = schema.push_services(ProtoService {
                package_id,
                name: service.name.unwrap_or_default(),
            });

            for method in service.method {
                let output_type = translate_type(
                    method.output_type(),
                    &messages_by_name,
                    &enums_by_name,
                    &local_messages_by_name,
                    &local_enums_by_name,
                );

                let input_type = translate_type(
                    method.input_type(),
                    &messages_by_name,
                    &enums_by_name,
                    &local_messages_by_name,
                    &local_enums_by_name,
                );

                schema.push_methods(ProtoMethod {
                    service_id,
                    name: method.name.unwrap_or_default(),
                    output_type,
                    input_type,
                });
            }
        }
    }

    schema
}

fn translate_message(
    schema: &mut TranslatedSchema,
    message: DescriptorProto,
    package_id: ProtoPackageId,
    messages_by_name: &HashMap<String, ProtoMessageId>,
    enums_by_name: &HashMap<String, ProtoEnumId>,
    local_messages_by_name: &HashMap<String, ProtoMessageId>,
    local_enums_by_name: &HashMap<String, ProtoEnumId>,
) {
    let message_id = schema.push_messages(ProtoMessage {
        package_id,
        name: message.name.unwrap_or_default(),
    });

    for _ in message.nested_type {
        todo!("nested message types support")
    }

    for _ in message.enum_type {
        todo!("nested enum types support")
    }

    for field in message.field {
        if field.default_value.is_some() {
            todo!("field default values")
        }

        let number = field
            .number()
            .try_into()
            .expect("Broken invariant: field number must be nonnegative");

        let r#type = match field.r#type() {
            field_descriptor_proto::Type::Double => FieldType::Scalar(ScalarType::Double),
            field_descriptor_proto::Type::Float => FieldType::Scalar(ScalarType::Float),
            field_descriptor_proto::Type::Int64 => FieldType::Scalar(ScalarType::Int64),
            field_descriptor_proto::Type::Uint64 => FieldType::Scalar(ScalarType::UInt64),
            field_descriptor_proto::Type::Int32 => FieldType::Scalar(ScalarType::Int32),
            field_descriptor_proto::Type::Fixed64 => FieldType::Scalar(ScalarType::Fixed64),
            field_descriptor_proto::Type::Fixed32 => FieldType::Scalar(ScalarType::Fixed32),
            field_descriptor_proto::Type::Bool => FieldType::Scalar(ScalarType::Bool),
            field_descriptor_proto::Type::String => FieldType::Scalar(ScalarType::String),
            field_descriptor_proto::Type::Bytes => FieldType::Scalar(ScalarType::Bytes),
            field_descriptor_proto::Type::Uint32 => FieldType::Scalar(ScalarType::UInt32),
            field_descriptor_proto::Type::Sfixed32 => FieldType::Scalar(ScalarType::Sfixed32),
            field_descriptor_proto::Type::Sfixed64 => FieldType::Scalar(ScalarType::Sfixed64),
            field_descriptor_proto::Type::Sint32 => FieldType::Scalar(ScalarType::Sint32),
            field_descriptor_proto::Type::Sint64 => FieldType::Scalar(ScalarType::Sint64),

            // ...
            field_descriptor_proto::Type::Group => continue,
            field_descriptor_proto::Type::Enum | field_descriptor_proto::Type::Message => translate_type(
                field.type_name(),
                &messages_by_name,
                &enums_by_name,
                &local_messages_by_name,
                &local_enums_by_name,
            ),
        };

        if field.default_value.is_some() {
            todo!("field.default_value")
        }

        schema.push_fields(ProtoField {
            message_id,
            name: field.name.unwrap_or_default(),
            r#type,
            number,
        });
    }
}

fn translate_type(
    field_type: &str,
    messages_by_name: &HashMap<String, ProtoMessageId>,
    enums_by_name: &HashMap<String, ProtoEnumId>,
    local_messages_by_name: &HashMap<String, ProtoMessageId>,
    local_enums_by_name: &HashMap<String, ProtoEnumId>,
) -> FieldType {
    if let Ok(scalar_type) = ScalarType::from_str(field_type) {
        return FieldType::Scalar(scalar_type);
    }

    if field_type.contains("<") {
        todo!("parse map types");
    }

    if let Some(message_id) = local_messages_by_name
        .get(field_type)
        .or_else(|| messages_by_name.get(field_type))
    {
        return FieldType::Message(*message_id);
    }

    if let Some(enum_id) = local_enums_by_name
        .get(field_type)
        .or_else(|| enums_by_name.get(field_type))
    {
        return FieldType::Enum(*enum_id);
    }

    panic!("Encountered unexpected unknown field type: {field_type}");
}
