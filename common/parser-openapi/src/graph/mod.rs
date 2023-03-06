use openapiv3::StatusCode;
use petgraph::{graph::NodeIndex, Graph};

use crate::parsing::operations::OperationDetails;

mod input_value;
mod query_types;

pub use self::{
    input_value::{InputValue, InputValueKind},
    query_types::{OutputType, PathParameter, QueryOperation},
};

/// A graph representation of an OpenApi schema.
///
/// Can be queried to determine what resources are linked to what models etc.
pub struct OpenApiGraph {
    graph: Graph<Node, Edge>,
    operation_indices: Vec<NodeIndex>,
    pub metadata: crate::ApiMetadata,
}

impl OpenApiGraph {
    pub fn new(parsed: crate::parsing::Context, metadata: crate::ApiMetadata) -> Self {
        OpenApiGraph {
            graph: parsed.graph,
            operation_indices: parsed.operation_indices,
            metadata,
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

    /// An edge between an operation and the type/schema of one of its path parameters
    HasPathParameter { name: String, wrapping: WrappingType },

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
}

#[derive(Debug)]
pub enum ScalarKind {
    String,
    Integer,
    Float,
    Boolean,
    #[allow(dead_code)]
    Id,
    JsonObject,
}

impl ScalarKind {
    fn type_name(&self) -> String {
        use dynaql::registry::scalars::{JSONScalar, SDLDefinitionScalar};

        match self {
            ScalarKind::String => "String".to_string(),
            ScalarKind::Integer => "Int".to_string(),
            ScalarKind::Float => "Float".to_string(),
            ScalarKind::Boolean => "Boolean".to_string(),
            ScalarKind::Id => "ID".to_string(),
            ScalarKind::JsonObject => JSONScalar::name().expect("JSONScalar to have a name").to_owned(),
        }
    }
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

// The GraphQL spec calls the "NonNull"/"List" types "wrapping types" so I'm borrowing
// that terminology here
#[derive(Clone, Debug)]
pub enum WrappingType {
    NonNull(Box<WrappingType>),
    List(Box<WrappingType>),
    Named,
}

impl WrappingType {
    pub fn contains_list(&self) -> bool {
        match self {
            WrappingType::NonNull(inner) => inner.contains_list(),
            WrappingType::List(_) => true,
            WrappingType::Named => false,
        }
    }
}
