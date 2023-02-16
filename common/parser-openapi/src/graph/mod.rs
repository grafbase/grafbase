use openapiv3::StatusCode;
use petgraph::{graph::NodeIndex, Graph};

use crate::parsing::operations::OperationDetails;

mod query_types;

/// A graph representation of an OpenApi schema.
///
/// Can be queried to determine what resources are linked to what models etc.
#[derive(Default)]
pub struct OpenApiGraph {
    graph: Graph<Node, Edge>,
    operation_indices: Vec<NodeIndex>,
}

impl OpenApiGraph {
    pub fn new(parsed: crate::parsing::Context) -> Self {
        OpenApiGraph {
            graph: parsed.graph,
            operation_indices: parsed.operation_indices,
        }
    }
}

pub struct SchemaDetails {
    openapi_name: String,
    openapi: openapiv3::Schema,
}

impl SchemaDetails {
    pub fn new(openapi_name: String, openapi: openapiv3::Schema) -> Self {
        SchemaDetails { openapi_name, openapi }
    }
}

pub enum Node {
    /// A schema in the OpenApi spec.
    Schema(SchemaDetails),

    /// An individual HTTP operation in the OpenApi spec.
    Operation(OperationDetails),

    /// A GraphQL Object that may be needed in the output.
    Object,
    /// A scalar
    Scalar(ScalarKind),

    /// A union type that may be needed in the output.
    Union,
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum Edge {
    /// Links an object with the types of it's fields.
    HasField { name: String, wrapping: WrappingType },

    /// The edge between a schema and its underlying type
    HasType { wrapping: WrappingType },

    /// An edge bewteen an operation and it's request type
    #[allow(dead_code)]
    HasRequestType {
        content_type: String,
        wrapping: WrappingType,
    },

    /// An edge bewteen an operation and it's response type
    HasResponseType {
        status_code: StatusCode,
        #[allow(dead_code)]
        content_type: String,
        wrapping: WrappingType,
    },

    /// An edge between a union and it's constituent members
    HasUnionMember,
}

impl Node {
    fn as_operation(&self) -> Option<&OperationDetails> {
        match self {
            Node::Operation(op) => Some(op),
            _ => None,
        }
    }

    fn as_object(&self) -> Option<()> {
        match self {
            Node::Object => Some(()),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum ScalarKind {
    String,
    Integer,
    Float,
    Boolean,
    #[allow(dead_code)]
    Id,
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Schema(schema) => f
                .debug_struct("Schema")
                .field("name", &schema.openapi_name)
                .finish_non_exhaustive(),
            Self::Operation(details) => f.debug_tuple("Operation").field(details).finish(),
            Self::Object => write!(f, "Object"),
            Self::Scalar(kind) => f.debug_tuple("Scalar").field(kind).finish(),
            Self::Union => write!(f, "Union"),
        }
    }
}

#[derive(Debug)]
pub enum WrappingType {
    Required(Box<WrappingType>),
    List(Box<WrappingType>),
    Named,
}
