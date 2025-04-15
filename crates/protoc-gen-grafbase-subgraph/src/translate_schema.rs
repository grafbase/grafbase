use crate::schema::{self, *};

use prost_types::{
    DescriptorProto, EnumDescriptorProto, ServiceDescriptorProto, SourceCodeInfo,
    compiler::CodeGeneratorRequest,
    field_descriptor_proto::{self, Label},
};
use std::{collections::HashMap, str::FromStr as _};

/// Instantiates a new translated schema from protobuf definitions.
pub(super) fn translate_schema(code_generator_request: CodeGeneratorRequest) -> GrpcSchema {
    let mut schema = GrpcSchema::default();
    let mut messages_by_fully_qualified_name = HashMap::new();
    let mut enums_by_fully_qualified_name = HashMap::new();
    let mut location: Vec<i32> = Vec::with_capacity(4);

    for proto_file in code_generator_request.proto_file {
        let source_code_info = proto_file.source_code_info.unwrap_or_default();

        let parent = if let Some(package_name) = proto_file.package.as_ref().filter(|name| !name.is_empty()) {
            let package_id = schema.push_packages(ProtoPackage {
                name: package_name.clone(),
            });
            Parent::Package(package_id)
        } else {
            Parent::Root
        };

        for (idx, r#enum) in proto_file.enum_type.iter().enumerate() {
            location.clear();
            location.push(5); // EnumDescriptorProto.enum_type is field number 5
            location.push(idx as i32);

            translate_enum(
                &mut schema,
                &mut location,
                &source_code_info,
                parent,
                r#enum,
                &mut enums_by_fully_qualified_name,
            );
        }

        // First resolve the messages _without their fields_, because the fields rely on the names of the messages.
        for (idx, message) in proto_file.message_type.iter().enumerate() {
            location.clear();
            location.push(4); // FileDescriptorProto.message_type is field number 4
            location.push(idx as i32);

            translate_message(
                &mut schema,
                &mut location,
                &source_code_info,
                parent,
                message,
                &mut messages_by_fully_qualified_name,
                &mut enums_by_fully_qualified_name,
            );
        }

        for (idx, message) in proto_file.message_type.iter().enumerate() {
            location.clear();
            location.push(4); // FileDescriptorProto.message_type is field number 4
            location.push(idx as i32);

            translate_fields(
                &mut schema,
                &mut location,
                &source_code_info,
                parent,
                message,
                &messages_by_fully_qualified_name,
                &enums_by_fully_qualified_name,
            );
        }

        for (index, service) in proto_file.service.iter().enumerate() {
            location.clear();
            location.push(6); // FileDescriptorProto.service is field number 6
            location.push(index as i32);

            translate_service(
                &mut schema,
                &mut location,
                &source_code_info,
                parent,
                service,
                &messages_by_fully_qualified_name,
                &enums_by_fully_qualified_name,
            )
        }
    }

    schema
}

fn translate_service(
    schema: &mut GrpcSchema,
    location: &mut Vec<i32>,
    source_code_info: &SourceCodeInfo,
    parent: schema::Parent,
    service: &ServiceDescriptorProto,
    messages_by_fully_qualified_name: &HashMap<String, ProtoMessageId>,
    enums_by_fully_qualified_name: &HashMap<String, ProtoEnumId>,
) {
    let service_id = schema.push_services(ProtoService {
        parent,
        name: if service.name().contains(".") {
            service.name.clone().unwrap_or_default()
        } else {
            match parent {
                Parent::Message(_) | Parent::Root => service.name.clone().unwrap_or_default(),
                Parent::Package(proto_package_id) => format!("{}.{}", schema[proto_package_id].name, service.name()),
            }
        },
        description: location_to_description(location, source_code_info),
    });

    for (idx, method) in service.method.iter().enumerate() {
        location.push(2); // method field on service
        location.push(idx as i32);

        let description = location_to_description(location, source_code_info);

        location.pop();
        location.pop();

        let output_type = translate_type(
            method.output_type(),
            messages_by_fully_qualified_name,
            enums_by_fully_qualified_name,
        );

        let input_type = translate_type(
            method.input_type(),
            messages_by_fully_qualified_name,
            enums_by_fully_qualified_name,
        );

        schema.push_methods(ProtoMethod {
            service_id,
            name: method.name.clone().unwrap_or_default(),
            output_type,
            input_type,
            description,
            server_streaming: method.server_streaming(),
            client_streaming: method.client_streaming(),
        });
    }
}

fn translate_enum(
    schema: &mut GrpcSchema,
    location: &mut Vec<i32>,
    source_code_info: &SourceCodeInfo,
    parent: schema::Parent,
    r#enum: &EnumDescriptorProto,
    enums_by_name: &mut HashMap<String, ProtoEnumId>,
) {
    let name = parent.child_name(schema, r#enum.name());

    let mut values = Vec::with_capacity(r#enum.value.len());

    for (idx, value) in r#enum.value.iter().enumerate() {
        location.push(2); // value field on EnumDescriptorProto
        location.push(idx as i32);

        let description = location_to_description(location, source_code_info);

        location.pop();
        location.pop();

        values.push(ProtoEnumValue {
            name: value.name.clone().unwrap_or_default(),
            number: value.number(),
            description,
        });
    }

    let enum_id = schema.push_enums(ProtoEnum {
        parent,
        name: name.clone(),
        description: location_to_description(location, source_code_info),
        values,
    });

    enums_by_name.insert(name, enum_id);
}

fn translate_message(
    schema: &mut GrpcSchema,
    location: &mut Vec<i32>,
    source_code_info: &SourceCodeInfo,
    parent: schema::Parent,
    message: &DescriptorProto,
    messages_by_name: &mut HashMap<String, ProtoMessageId>,
    enums_by_name: &mut HashMap<String, ProtoEnumId>,
) {
    let name = parent.child_name(schema, message.name());

    let message_id = schema.push_messages(ProtoMessage {
        parent,
        name: name.clone(),
        is_map_entry: message
            .options
            .as_ref()
            .map(|opts| opts.map_entry())
            .unwrap_or_default(),
        description: location_to_description(location, source_code_info),
    });

    messages_by_name.insert(name, message_id);

    for (idx, submessage) in message.nested_type.iter().enumerate() {
        location.push(3); // DescriptorProto.nested_type
        location.push(idx as i32);

        translate_message(
            schema,
            location,
            source_code_info,
            Parent::Message(message_id),
            submessage,
            messages_by_name,
            enums_by_name,
        );

        location.pop();
        location.pop();
    }

    for (idx, r#enum) in message.enum_type.iter().enumerate() {
        location.push(4); // DescriptorProto.enum_type
        location.push(idx as i32);

        translate_enum(
            schema,
            location,
            source_code_info,
            Parent::Message(message_id),
            r#enum,
            enums_by_name,
        );

        location.pop();
        location.pop();
    }
}

fn translate_fields(
    schema: &mut GrpcSchema,
    location: &mut Vec<i32>,
    source_code_info: &SourceCodeInfo,
    parent: Parent,
    message: &DescriptorProto,
    messages_by_fully_qualified_name: &HashMap<String, ProtoMessageId>,
    enums_by_fully_qualified_name: &HashMap<String, ProtoEnumId>,
) {
    let message_id = messages_by_fully_qualified_name[parent.child_name(schema, message.name()).as_str()];

    for (idx, field) in message.field.iter().enumerate() {
        location.push(2); // DescriptorProto.field
        location.push(idx as i32);
        let description = location_to_description(location, source_code_info);
        location.pop();
        location.pop();

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
                messages_by_fully_qualified_name,
                enums_by_fully_qualified_name,
            ),
        };

        let repeated = field.label() == Label::Repeated;

        schema.push_fields(ProtoField {
            message_id,
            name: field.name().to_owned(),
            r#type,
            number,
            repeated,
            description,
        });
    }

    // Now do the same for all submessages
    for submessage in &message.nested_type {
        translate_fields(
            schema,
            location,
            source_code_info,
            Parent::Message(message_id),
            submessage,
            messages_by_fully_qualified_name,
            enums_by_fully_qualified_name,
        );
    }
}

/// In the descriptor input, all message and input type names are fully qualified and prefixed with a dot.
fn translate_type(
    field_type: &str,
    messages_by_name: &HashMap<String, ProtoMessageId>,
    enums_by_name: &HashMap<String, ProtoEnumId>,
) -> FieldType {
    if let Ok(scalar_type) = ScalarType::from_str(field_type) {
        return FieldType::Scalar(scalar_type);
    }

    if let Some(message_id) = messages_by_name.get(field_type) {
        return FieldType::Message(*message_id);
    }

    if let Some(enum_id) = enums_by_name.get(field_type) {
        return FieldType::Enum(*enum_id);
    }

    unreachable!("Encountered unexpected unknown field type: {field_type}");
}

fn location_to_description(path: &[i32], source_code_info: &SourceCodeInfo) -> Option<String> {
    for location in &source_code_info.location {
        if location.path == path {
            let mut description = String::new();

            if location.leading_comments().is_empty() && location.trailing_comments().is_empty() {
                continue;
            }

            description.push_str(location.leading_comments().trim());

            description.push('\n');

            description.push_str(location.trailing_comments().trim());

            return Some(description);
        }
    }

    None
}
