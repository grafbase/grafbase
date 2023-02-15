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
    operation_index: Vec<NodeIndex>,
}

impl OpenApiGraph {
    pub fn new(parsed: crate::parsing::Context) -> Self {
        OpenApiGraph {
            graph: parsed.graph,
            operation_index: parsed.operation_index,
        }
    }
}

pub struct SchemaDetails {
    name: String,
    openapi: openapiv3::Schema,
}

impl SchemaDetails {
    pub fn new(name: String, openapi: openapiv3::Schema) -> Self {
        SchemaDetails { name, openapi }
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
    HasField { name: String, wrapper: WrapperType },

    /// The edge between a schema and its underlying type
    HasType { wrapper: WrapperType },

    /// An edge bewteen an operation and it's request type
    #[allow(dead_code)]
    HasRequestType { content_type: String, wrapper: WrapperType },

    /// An edge bewteen an operation and it's response type
    HasResponseType {
        status_code: StatusCode,
        #[allow(dead_code)]
        content_type: String,
        wrapper: WrapperType,
    },

    /// An edge between a union and it's constituent members
    HasUnionMember,
}

impl Node {
    fn operation(&self) -> Option<&OperationDetails> {
        match self {
            Node::Operation(op) => Some(op),
            _ => None,
        }
    }

    fn object(&self) -> Option<()> {
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
                .field("name", &schema.name)
                .finish_non_exhaustive(),
            Self::Operation(arg0) => f.debug_tuple("Operation").field(arg0).finish(),
            Self::Object => write!(f, "Object"),
            Self::Scalar(arg0) => f.debug_tuple("Scalar").field(arg0).finish(),
            Self::Union => write!(f, "Union"),
        }
    }
}

#[derive(Debug)]
pub enum WrapperType {
    Required(Box<WrapperType>),
    List(Box<WrapperType>),
    Named,
}
