mod options;

use self::options::*;
use crate::schema::{self, *};
use protobuf::descriptor::{
    DescriptorProto, EnumDescriptorProto, FileDescriptorSet, ServiceDescriptorProto, SourceCodeInfo,
    field_descriptor_proto::Label, field_descriptor_proto::Type as FieldType,
};
use protobuf::plugin::CodeGeneratorRequest;
use std::{collections::HashMap, str::FromStr as _};

/// Instantiates a new translated schema from protobuf definitions.
pub(super) fn translate_schema(code_generator_request: CodeGeneratorRequest) -> GrpcSchema {
    let mut schema = GrpcSchema::default();
    let mut messages_by_fully_qualified_name = HashMap::new();
    let mut enums_by_fully_qualified_name = HashMap::new();
    let mut location: Vec<i32> = Vec::with_capacity(4);

    // Create descriptor set for accessing extensions
    let mut file_descriptor_set = FileDescriptorSet::new();
    file_descriptor_set.file = code_generator_request.proto_file.clone();

    for proto_file in code_generator_request.proto_file {
        let source_code_info = proto_file.source_code_info.clone().unwrap_or_default();

        let parent = if proto_file.has_package() && !proto_file.package().is_empty() {
            let package_id = schema.push_packages(ProtoPackage {
                name: proto_file.package().to_string(),
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
    let mut proto_service = ProtoService {
        parent,
        name: if service.name().contains(".") {
            service.name().to_string()
        } else {
            match parent {
                Parent::Message(_) | Parent::Root => service.name().to_string(),
                Parent::Package(proto_package_id) => format!("{}.{}", schema[proto_package_id].name, service.name()),
            }
        },
        description: location_to_description(location, source_code_info),
        default_to_query_fields: false,
        default_to_mutation_fields: false,
    };

    extract_service_graphql_options_from_options(service, &mut proto_service);

    let service_id = schema.push_services(proto_service);

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

        let mut proto_method = ProtoMethod {
            service_id,
            name: method.name().to_string(),
            output_type,
            input_type,
            description,
            server_streaming: method.server_streaming(),
            client_streaming: method.client_streaming(),
            is_query: None,
            is_mutation: None,
            directives: None,
        };

        extract_method_graphql_options_from_options(method, &mut proto_method);

        schema.push_methods(proto_method);
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

        let mut proto_enum_value = ProtoEnumValue {
            name: value.name().to_string(),
            number: value.number(),
            description,
            enum_value_directives: None,
        };

        extract_enum_value_graphql_directives_from_options(value, &mut proto_enum_value);

        values.push(proto_enum_value);
    }

    let mut proto_enum = ProtoEnum {
        parent,
        name: name.clone(),
        description: location_to_description(location, source_code_info),
        values,
        enum_directives: None,
    };

    extract_enum_graphql_directives_from_options(r#enum, &mut proto_enum);

    let enum_id = schema.push_enums(proto_enum);

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

    let mut translated_message = ProtoMessage {
        parent,
        name: name.clone(),
        is_map_entry: message.options.is_some() && message.options.as_ref().is_some_and(|opts| opts.map_entry()),
        description: location_to_description(location, source_code_info),
        input_object_directives: None,
        object_directives: None,
    };

    extract_message_graphql_directives_from_options(message, &mut translated_message);

    let message_id = schema.push_messages(translated_message);

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

        let r#type = match field.type_.unwrap_or_default().enum_value_or_default() {
            FieldType::TYPE_DOUBLE => crate::schema::FieldType::Scalar(ScalarType::Double),
            FieldType::TYPE_FLOAT => crate::schema::FieldType::Scalar(ScalarType::Float),
            FieldType::TYPE_INT64 => crate::schema::FieldType::Scalar(ScalarType::Int64),
            FieldType::TYPE_UINT64 => crate::schema::FieldType::Scalar(ScalarType::UInt64),
            FieldType::TYPE_INT32 => crate::schema::FieldType::Scalar(ScalarType::Int32),
            FieldType::TYPE_FIXED64 => crate::schema::FieldType::Scalar(ScalarType::Fixed64),
            FieldType::TYPE_FIXED32 => crate::schema::FieldType::Scalar(ScalarType::Fixed32),
            FieldType::TYPE_BOOL => crate::schema::FieldType::Scalar(ScalarType::Bool),
            FieldType::TYPE_STRING => crate::schema::FieldType::Scalar(ScalarType::String),
            FieldType::TYPE_BYTES => crate::schema::FieldType::Scalar(ScalarType::Bytes),
            FieldType::TYPE_UINT32 => crate::schema::FieldType::Scalar(ScalarType::UInt32),
            FieldType::TYPE_SFIXED32 => crate::schema::FieldType::Scalar(ScalarType::Sfixed32),
            FieldType::TYPE_SFIXED64 => crate::schema::FieldType::Scalar(ScalarType::Sfixed64),
            FieldType::TYPE_SINT32 => crate::schema::FieldType::Scalar(ScalarType::Sint32),
            FieldType::TYPE_SINT64 => crate::schema::FieldType::Scalar(ScalarType::Sint64),

            // ...
            FieldType::TYPE_GROUP => continue,
            FieldType::TYPE_ENUM | FieldType::TYPE_MESSAGE => translate_type(
                field.type_name(),
                messages_by_fully_qualified_name,
                enums_by_fully_qualified_name,
            ),
        };

        let repeated = field.label.unwrap_or_default().enum_value_or_default() == Label::LABEL_REPEATED;

        let mut proto_field = ProtoField {
            message_id,
            name: field.name().to_owned(),
            r#type,
            number,
            repeated,
            description,
            input_field_directives: None,
            output_field_directives: None,
        };

        extract_field_graphql_directives_from_options(field, &mut proto_field);

        schema.push_fields(proto_field);
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
) -> crate::schema::FieldType {
    if let Ok(scalar_type) = ScalarType::from_str(field_type) {
        return crate::schema::FieldType::Scalar(scalar_type);
    }

    if let Some(message_id) = messages_by_name.get(field_type) {
        return crate::schema::FieldType::Message(*message_id);
    }

    if let Some(enum_id) = enums_by_name.get(field_type) {
        return crate::schema::FieldType::Enum(*enum_id);
    }

    unreachable!("Encountered unexpected unknown field type: {field_type}");
}

fn extract_message_graphql_directives_from_options(message: &DescriptorProto, translated_message: &mut ProtoMessage) {
    let [graphql_output_object_directives, graphql_input_object_directives] =
        [OBJECT_DIRECTIVES, INPUT_OBJECT_DIRECTIVES].map(|field_number| {
            message
                .options
                .special_fields
                .unknown_fields()
                .get(field_number)
                .and_then(|unknown_value_ref| match unknown_value_ref {
                    protobuf::UnknownValueRef::LengthDelimited(items) => {
                        Some(str::from_utf8(items).unwrap().to_owned())
                    }
                    _ => None,
                })
        });

    translated_message.object_directives = graphql_output_object_directives;
    translated_message.input_object_directives = graphql_input_object_directives;
}

fn extract_field_graphql_directives_from_options(
    field: &protobuf::descriptor::FieldDescriptorProto,
    proto_field: &mut ProtoField,
) {
    let [graphql_output_field_directives, graphql_input_field_directives] =
        [OUTPUT_FIELD_DIRECTIVES, INPUT_FIELD_DIRECTIVES].map(|field_number| {
            field
                .options
                .special_fields
                .unknown_fields()
                .get(field_number)
                .and_then(|unknown_value_ref| match unknown_value_ref {
                    protobuf::UnknownValueRef::LengthDelimited(items) => {
                        Some(str::from_utf8(items).unwrap().to_owned())
                    }
                    _ => None,
                })
        });

    proto_field.output_field_directives = graphql_output_field_directives;
    proto_field.input_field_directives = graphql_input_field_directives;
}

fn extract_enum_graphql_directives_from_options(
    enum_desc: &protobuf::descriptor::EnumDescriptorProto,
    proto_enum: &mut ProtoEnum,
) {
    let graphql_enum_directives = enum_desc
        .options
        .special_fields
        .unknown_fields()
        .get(ENUM_DIRECTIVES)
        .and_then(|unknown_value_ref| match unknown_value_ref {
            protobuf::UnknownValueRef::LengthDelimited(items) => Some(str::from_utf8(items).unwrap().to_owned()),
            _ => None,
        });

    proto_enum.enum_directives = graphql_enum_directives;
}

fn extract_enum_value_graphql_directives_from_options(
    enum_value: &protobuf::descriptor::EnumValueDescriptorProto,
    proto_enum_value: &mut ProtoEnumValue,
) {
    let graphql_enum_value_directives = enum_value
        .options
        .special_fields
        .unknown_fields()
        .get(ENUM_VALUE_DIRECTIVES)
        .and_then(|unknown_value_ref| match unknown_value_ref {
            protobuf::UnknownValueRef::LengthDelimited(items) => Some(str::from_utf8(items).unwrap().to_owned()),
            _ => None,
        });

    proto_enum_value.enum_value_directives = graphql_enum_value_directives;
}

fn extract_service_graphql_options_from_options(service: &ServiceDescriptorProto, proto_service: &mut ProtoService) {
    let graphql_default_to_query_fields = service
        .options
        .special_fields
        .unknown_fields()
        .get(DEFAULT_TO_QUERY_FIELDS)
        .and_then(|unknown_value_ref| match unknown_value_ref {
            protobuf::UnknownValueRef::Varint(value) => Some(value != 0),
            _ => None,
        })
        .unwrap_or(false);

    let graphql_default_to_mutation_fields = service
        .options
        .special_fields
        .unknown_fields()
        .get(DEFAULT_TO_MUTATION_FIELDS)
        .and_then(|unknown_value_ref| match unknown_value_ref {
            protobuf::UnknownValueRef::Varint(value) => Some(value != 0),
            _ => None,
        })
        .unwrap_or(false);

    proto_service.default_to_query_fields = graphql_default_to_query_fields;
    proto_service.default_to_mutation_fields = graphql_default_to_mutation_fields;
}

fn extract_method_graphql_options_from_options(
    method: &protobuf::descriptor::MethodDescriptorProto,
    proto_method: &mut ProtoMethod,
) {
    let is_query = method
        .options
        .special_fields
        .unknown_fields()
        .get(IS_QUERY)
        .and_then(|unknown_value_ref| match unknown_value_ref {
            protobuf::UnknownValueRef::Varint(value) => Some(value != 0),
            _ => None,
        });

    let is_mutation = method
        .options
        .special_fields
        .unknown_fields()
        .get(IS_MUTATION)
        .and_then(|unknown_value_ref| match unknown_value_ref {
            protobuf::UnknownValueRef::Varint(value) => Some(value != 0),
            _ => None,
        });

    let directives = method
        .options
        .special_fields
        .unknown_fields()
        .get(DIRECTIVES)
        .and_then(|unknown_value_ref| match unknown_value_ref {
            protobuf::UnknownValueRef::LengthDelimited(items) => Some(str::from_utf8(items).unwrap().to_owned()),
            _ => None,
        });

    proto_method.directives = directives;
    proto_method.is_query = is_query;
    proto_method.is_mutation = is_mutation;
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
