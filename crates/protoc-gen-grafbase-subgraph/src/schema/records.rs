use std::{
    fmt::{self, Display as _},
    str::FromStr,
};

use crate::display_utils::{self, display_fn, grpc_path_to_graphql_name};

use super::{Parent, ids::*};

#[derive(Debug, PartialEq, PartialOrd)]
pub(crate) struct ProtoPackage {
    pub(crate) name: String,
}

#[derive(Debug, PartialEq)]
pub(crate) struct ProtoMessage {
    pub(crate) parent: Parent,
    pub(crate) name: String,
    /// Is this an auto-generated message type corresponding to a map? See https://docs.rs/prost-types/0.13.5/prost_types/struct.MessageOptions.html#structfield.map_entry
    pub(crate) is_map_entry: bool,
    pub(crate) description: Option<String>,
}

impl ProtoMessage {
    pub(crate) fn graphql_output_name(&self) -> impl fmt::Display {
        grpc_path_to_graphql_name(&self.name)
    }

    pub(crate) fn graphql_input_name(&self) -> impl fmt::Display {
        display_fn(|f| {
            self.graphql_output_name().fmt(f)?;
            f.write_str("Input")
        })
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct ProtoField {
    pub(crate) message_id: ProtoMessageId,
    pub(crate) name: String,
    pub(crate) r#type: FieldType,
    pub(crate) number: u16,
    pub(crate) repeated: bool,
    pub(crate) description: Option<String>,
}

impl PartialOrd for ProtoField {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.message_id.partial_cmp(&other.message_id)
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct ProtoService {
    pub(crate) parent: Parent,
    pub(crate) name: String,
    pub(crate) description: Option<String>,
}

impl ProtoService {
    pub(crate) fn graphql_name(&self) -> impl fmt::Display {
        grpc_path_to_graphql_name(&self.name)
    }
}

impl PartialOrd for ProtoService {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.parent.partial_cmp(&other.parent)
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct ProtoMethod {
    pub(crate) service_id: ProtoServiceId,
    pub(crate) name: String,
    pub(crate) output_type: FieldType,
    pub(crate) input_type: FieldType,
    pub(crate) description: Option<String>,
    pub(crate) server_streaming: bool,
    pub(crate) client_streaming: bool,
}

impl PartialOrd for ProtoMethod {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.service_id.partial_cmp(&other.service_id)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum FieldType {
    Scalar(ScalarType),
    Message(ProtoMessageId),
    Enum(ProtoEnumId),
}

impl FieldType {
    pub(crate) fn proto_name(&self, schema: &super::GrpcSchema) -> impl fmt::Display {
        display_fn(move |f| match self {
            FieldType::Scalar(scalar_type) => f.write_str(scalar_type.proto_name()),
            FieldType::Message(proto_message_id) => f.write_str(&schema[*proto_message_id].name),
            FieldType::Enum(proto_enum_id) => f.write_str(schema[*proto_enum_id].name.as_str()),
        })
    }
}

/// See scalar value types table in [the reference](https://protobuf.dev/programming-guides/proto3/).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum ScalarType {
    Double,
    Float,
    Int32,
    Int64,
    UInt32,
    UInt64,
    Sint32,
    Sint64,
    Fixed32,
    Fixed64,
    Sfixed32,
    Sfixed64,
    Bool,
    String,
    Bytes,
}

impl ScalarType {
    pub(crate) fn render_graphql_type(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ScalarType::Double | ScalarType::Float => "Float",
            ScalarType::Int32
            | ScalarType::UInt32
            | ScalarType::Sint32
            | ScalarType::Fixed32
            | ScalarType::Sfixed32 => "Int",
            ScalarType::Int64 | ScalarType::Sint64 | ScalarType::Sfixed64 => "I64",
            ScalarType::Fixed64 | ScalarType::UInt64 => "U64",
            ScalarType::Bool => "Boolean",
            ScalarType::String => "String",
            ScalarType::Bytes => "Bytes",
        })
    }

    pub(crate) fn proto_name(&self) -> &'static str {
        match self {
            ScalarType::Double => "double",
            ScalarType::Float => "float",
            ScalarType::Int32 => "int32",
            ScalarType::Int64 => "int64",
            ScalarType::UInt32 => "uint32",
            ScalarType::UInt64 => "uint64",
            ScalarType::Sint32 => "sint32",
            ScalarType::Sint64 => "sint64",
            ScalarType::Fixed32 => "fixed32",
            ScalarType::Fixed64 => "fixed64",
            ScalarType::Sfixed32 => "sfixed32",
            ScalarType::Sfixed64 => "sfixed64",
            ScalarType::Bool => "bool",
            ScalarType::String => "string",
            ScalarType::Bytes => "bytes",
        }
    }
}

impl FromStr for ScalarType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "double" => Ok(ScalarType::Double),
            "float" => Ok(ScalarType::Float),
            "int32" => Ok(ScalarType::Int32),
            "int64" => Ok(ScalarType::Int64),
            "uint32" => Ok(ScalarType::UInt32),
            "uint64" => Ok(ScalarType::UInt64),
            "sint32" => Ok(ScalarType::Sint32),
            "sint64" => Ok(ScalarType::Sint64),
            "fixed32" => Ok(ScalarType::Fixed32),
            "fixed64" => Ok(ScalarType::Fixed64),
            "sfixed32" => Ok(ScalarType::Sfixed32),
            "sfixed64" => Ok(ScalarType::Sfixed64),
            "bool" => Ok(ScalarType::Bool),
            "string" => Ok(ScalarType::String),
            "bytes" => Ok(ScalarType::Bytes),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct ProtoEnum {
    pub(crate) parent: Parent,
    pub(crate) name: String,
    pub(crate) values: Vec<ProtoEnumValue>,
    pub(crate) description: Option<String>,
}

impl ProtoEnum {
    pub(crate) fn graphql_name(&self) -> impl fmt::Display {
        display_utils::grpc_path_to_graphql_name(&self.name)
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct ProtoEnumValue {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) number: i32,
}
