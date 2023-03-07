use std::{borrow::Cow, collections::HashMap};

use dynaql::registry::resolvers::http::QueryParameterEncodingStyle;
use inflector::Inflector;
use openapiv3::StatusCode;
use petgraph::{
    graph::NodeIndex,
    visit::{EdgeRef, Reversed},
    Graph,
};

use crate::parsing::operations::OperationDetails;

mod enums;
mod input_object;
mod input_value;
mod operations;
mod output_type;
mod parameters;

pub use self::{
    enums::Enum,
    input_object::{InputField, InputObject},
    input_value::{InputValue, InputValueKind},
    operations::Operation,
    output_type::OutputType,
    parameters::{PathParameter, QueryParameter, RequestBody},
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

    /// An enum type
    Enum { values: Vec<String> },
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

    /// An edge between an operation and the type/schema of one of its query parameters
    HasQueryParameter {
        name: String,
        wrapping: WrappingType,
        encoding_style: QueryParameterEncodingStyle,
    },

    /// An edge bewteen an operation and it's request type
    HasRequestType {
        content_type: RequestBodyContentType,
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
            Self::Enum { values } => f.debug_struct("Enum").field("values", values).finish(),
            Self::Union => write!(f, "Union"),
        }
    }
}

// The GraphQL spec calls the "NonNull"/"List" types "wrapping types" so I'm borrowing
// that terminology here
#[derive(Clone, Debug, PartialEq)]
pub enum WrappingType {
    NonNull(Box<WrappingType>),
    List(Box<WrappingType>),
    Named,
}

impl WrappingType {
    pub(super) fn wrap_list(self) -> WrappingType {
        WrappingType::List(Box::new(self))
    }

    pub(super) fn wrap_required(self) -> WrappingType {
        if matches!(self, WrappingType::NonNull(_)) {
            // Don't double wrap things in required
            self
        } else {
            WrappingType::NonNull(Box::new(self))
        }
    }

    pub(super) fn wrap_with(self, other: WrappingType) -> WrappingType {
        match other {
            WrappingType::NonNull(other_inner) => {
                if matches!(self, WrappingType::NonNull(_)) {
                    // Don't double wrap in required
                    self.wrap_with(*other_inner)
                } else {
                    WrappingType::NonNull(Box::new(self.wrap_with(*other_inner)))
                }
            }
            WrappingType::List(other_inner) => WrappingType::List(Box::new(self.wrap_with(*other_inner))),
            WrappingType::Named => self,
        }
    }

    pub fn contains_list(&self) -> bool {
        match self {
            WrappingType::NonNull(inner) => inner.contains_list(),
            WrappingType::List(_) => true,
            WrappingType::Named => false,
        }
    }
}

impl OpenApiGraph {
    fn type_name(&self, node: NodeIndex) -> Option<String> {
        match &self.graph[node] {
            schema @ Node::Schema { .. } => Some(schema.name()?),
            Node::Operation(_) => None,
            Node::Object | Node::Enum { .. } => {
                // OpenAPI objects are generally anonymous so we walk back up the graph to the
                // nearest named thing, and construct a name based on the fields in-betweeen.
                // Not ideal, but the best we can do.
                let reversed_graph = Reversed(&self.graph);

                let (_, mut path) = petgraph::algo::astar(
                    &reversed_graph,
                    node,
                    |current_node| self.graph[current_node].name().is_some(),
                    |_| 0,
                    |_| 0,
                )?;

                let named_node = path.pop()?;

                // Reverse our path so we can look things up in the original graph.
                path.reverse();

                let mut name_components = Vec::new();
                let mut path_iter = path.into_iter().peekable();
                while let Some(src_node) = path_iter.next() {
                    let Some(&dest_node) = path_iter.peek() else { break; };

                    // I am sort of assuming there's only one edge here.
                    // Should be the case at the moment but might need to update this to a loop if that changes
                    let edge = self.graph.edges_connecting(src_node, dest_node).next().unwrap();
                    if let Edge::HasField { name, .. } = edge.weight() {
                        name_components.push(name.as_str());
                    }
                }

                let root_name = self.graph[named_node].name().unwrap();
                name_components.push(root_name.as_str());

                name_components.reverse();
                Some(name_components.join("_").to_pascal_case())
            }
            Node::Scalar(kind) => Some(kind.type_name()),
            Node::Union => {
                // Unions are named based on the names of their constituent types.
                // Although it's perfectly possible for any of the members to be un-named
                // so this will probably require a bit more work at some point.
                let mut name = self
                    .graph
                    .edges(node)
                    .filter_map(|edge| match edge.weight() {
                        Edge::HasUnionMember => OutputType::from_index(edge.target(), self)?.name(self),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("Or");
                name.push_str("Union");
                Some(name)
            }
        }
    }
}

impl Node {
    // Used to determine whether this specific node type has a name.
    // To generate the full name of a particular node you should use the OpenApiGraph::type_name
    // function.
    fn name(&self) -> Option<String> {
        match self {
            Node::Schema(schema) => Some(
                // There's a title property that we _could_ use for a name, but the spec doesn't
                // enforce that it's unique and (certainly in stripes case) it is not.
                // Might do some stuff to work around htat, but for now it's either "x-resourceId"
                // which stripe use or the name of the schema in components.
                schema
                    .openapi
                    .schema_data
                    .extensions
                    .get("x-resourceId")
                    .and_then(|v| v.as_str())
                    .unwrap_or(schema.openapi_name.as_str())
                    .to_pascal_case(),
            ),
            Node::Operation(op) => op.operation_id.clone(),
            _ => None,
        }
    }
}

pub struct FieldName<'a>(Cow<'a, str>);

impl<'a> std::fmt::Display for FieldName<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.0.to_camel_case();

        write!(f, "{name}")
    }
}

#[derive(Clone, Debug)]
pub enum RequestBodyContentType {
    Json,
    FormEncoded(HashMap<String, QueryParameterEncodingStyle>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::redundant_clone)]
    fn test_wrap_with() {
        let named_type = WrappingType::Named;
        let list = named_type.clone().wrap_list();
        let required = named_type.clone().wrap_required();

        let required_list = list.clone().wrap_required();

        assert_eq!(&named_type.clone().wrap_with(list.clone()), &list);
        assert_eq!(&named_type.clone().wrap_with(required.clone()), &required);

        assert_eq!(&named_type.clone().wrap_with(required_list.clone()), &required_list);
        assert_eq!(
            &list.clone().wrap_with(required_list.clone()),
            &list.wrap_list().wrap_required()
        );

        // Check that we can't double wrap things in required
        assert_eq!(required.clone().wrap_with(required.clone()), required);
    }
}
