use std::{borrow::Cow, fmt::Debug};

use engine::registry::resolvers::http::{ExpectedStatusCode, QueryParameterEncodingStyle, RequestBodyContentType};
use inflector::Inflector;
use once_cell::sync::Lazy;
use petgraph::{
    graph::NodeIndex,
    visit::{EdgeFiltered, EdgeRef, IntoEdges, Reversed},
    Graph,
};
use regex::Regex;
use serde_json::Value;

mod enums;
mod input_object;
mod input_value;
mod operations;
mod output_type;
mod parameters;
mod scalar;
mod transforms;

mod all_of_member;
pub mod construction;
mod debug;
mod resource;

pub use self::{
    debug::DebugNode,
    enums::Enum,
    input_object::{InputField, InputObject},
    input_value::{InputValue, InputValueKind},
    operations::Operation,
    output_type::{OutputField, OutputFieldType, OutputType},
    parameters::{PathParameter, QueryParameter, RequestBody},
    resource::{Resource, ResourceOperation},
    scalar::Scalar,
};
use crate::{parsing::ParseOutput, ApiMetadata, Error};

/// A graph representation of an OpenApi schema.
///
/// Can be queried to determine what resources are linked to what models etc.
pub struct OpenApiGraph {
    graph: Graph<Node, Edge>,
    operation_indices: Vec<NodeIndex>,
    pub metadata: ApiMetadata,
}

impl OpenApiGraph {
    pub fn new(parsed: ParseOutput, metadata: ApiMetadata) -> Result<Self, Error> {
        let mut this = OpenApiGraph {
            graph: parsed.graph,
            operation_indices: vec![],
            metadata,
        };

        transforms::run(&mut this)?;

        // ParseOutput has operation_indices _but_ the transforms might have removed
        // some nodes so we can't trust them anymore.  Re-calculating them here is
        // the safest option
        this.operation_indices = this
            .graph
            .node_indices()
            .filter(|index| matches!(this.graph[*index], Node::Operation(_)))
            .collect();

        Ok(this)
    }

    #[cfg(test)]
    pub fn from_petgraph(graph: Graph<Node, Edge>) -> Self {
        use engine::registry::ConnectorHeaders;
        use parser_sdl::OpenApiQueryNamingStrategy;

        OpenApiGraph {
            graph,
            operation_indices: vec![],
            metadata: ApiMetadata {
                name: String::from("Test"),
                namespace: false,
                url: None,
                headers: ConnectorHeaders::default(),
                query_naming: OpenApiQueryNamingStrategy::default(),
            },
        }
    }

    // Used to get a Debug impl for a the given node index.
    #[allow(dead_code)]
    fn debug(&self, index: NodeIndex) -> NodeDebug<'_> {
        NodeDebug(&self.graph[index])
    }
}

struct NodeDebug<'a>(&'a Node);

impl Debug for NodeDebug<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub struct SchemaDetails {
    openapi_name: String,
    resource_id: Option<String>,
}

impl SchemaDetails {
    pub fn new(openapi_name: String, resource_id: Option<String>) -> Self {
        SchemaDetails {
            openapi_name,
            resource_id,
        }
    }
}

pub enum Node {
    /// A schema in the OpenApi spec.
    Schema(Box<SchemaDetails>),

    /// An individual HTTP operation in the OpenApi spec.
    Operation(Box<OperationDetails>),

    /// A GraphQL Object that may be needed in the output.
    Object,

    /// A scalar
    Scalar(ScalarKind),

    /// A scalar that appears inside a union, so needs additional wrapping
    UnionWrappedScalar(ScalarKind),

    /// A union type that may be needed in the output.
    Union,

    /// An enum type
    Enum,

    // The default value for a type node, linked via a HasDefault edge
    Default(Value),

    // A possible value for a given scalar/enum
    PossibleValue(Value),

    /// An OpenAPI allOf schema.
    ///
    /// These should be optimised away by the `merge_all_of_schemas` transform.
    AllOf,

    /// If a schema is using allOf it's possible for there to be fields
    /// that don't have a defined type in one of the schemas, because one of the
    /// other schemas provides that information.  This represents one of those
    /// fields.  `merge_all_of_schemas` should deal with resolving placeholders
    /// to actual types...
    PlaceholderType,
}

#[derive(Clone, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum Edge {
    /// Links an object with the types of it's fields.
    HasField {
        name: String,
        /// Required is whether the field is required to be present in the object.
        /// This is a seprate concept from nullability in OpenAPI.  We'll need to
        /// merge this with wrapping type when doing our output.
        required: bool,
        /// Whether this field is non-null/a list etc.
        wrapping: WrappingType,
    },

    /// The edge between a schema and its underlying type
    HasType {
        wrapping: WrappingType,
    },

    /// An edge between an operation and the type/schema of one of its path parameters
    HasPathParameter {
        name: String,
        wrapping: WrappingType,
    },

    /// An edge between an operation and the type/schema of one of its query parameters
    HasQueryParameter {
        name: String,
        wrapping: WrappingType,
        encoding_style: QueryParameterEncodingStyle,
    },

    /// An edge bewteen an operation and it's request type
    HasRequestType {
        content_type: Box<RequestBodyContentType>,
        wrapping: WrappingType,
    },

    /// An edge bewteen an operation and it's response type
    HasResponseType {
        status_code: ExpectedStatusCode,
        content_type: String,
        wrapping: WrappingType,
    },

    /// An edge between a union and it's constituent members
    HasUnionMember,

    // This edge goes between an Operation and the Schema that represents
    // its primary resource.  This edge will only be present on Operations
    // where we can actually determine this.
    ForResource {
        arity: Arity,
    },

    /// An edge between any type node and its associated default
    HasDefault,

    /// An edge between a scalar type and its possible values.
    HasPossibleValue,

    /// An edge between an AllOfSchema and it's members.
    /// Members should be objects, or schemas that represent an object.
    /// This edge should be optimised out by the `merge_all_of_schemas` transform,
    /// so we shouldn't need to consider it at output time.
    AllOfMember,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Arity {
    One,
    Many,
}

impl Node {
    fn as_operation(&self) -> Option<&OperationDetails> {
        match self {
            Node::Operation(op) => Some(op),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ScalarKind {
    String,
    Integer,
    Float,
    Boolean,
    Json,
}

impl ScalarKind {
    pub fn type_name(self) -> String {
        use engine::registry::scalars::{JSONScalar, SDLDefinitionScalar};

        match self {
            ScalarKind::String => "String".to_string(),
            ScalarKind::Integer => "Int".to_string(),
            ScalarKind::Float => "Float".to_string(),
            ScalarKind::Boolean => "Boolean".to_string(),
            ScalarKind::Json => JSONScalar::name().expect("JSONScalar to have a name").to_owned(),
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
            Self::UnionWrappedScalar(kind) => f.debug_tuple("ScalarWrapper").field(kind).finish(),
            Self::Enum => f.debug_struct("Enum").finish(),
            Self::Union => write!(f, "Union"),
            Self::Default(value) => f.debug_tuple("Default").field(value).finish(),
            Self::PossibleValue(value) => f.debug_tuple("PossibleValue").field(value).finish(),
            Self::AllOf => f.debug_tuple("AllOf").finish(),
            Self::PlaceholderType => f.debug_tuple("PlaceholderType").finish(),
        }
    }
}

#[derive(Debug)]
pub struct OperationDetails {
    pub path: String,
    pub http_method: HttpMethod,
    pub operation_id: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::EnumString, strum::Display)]
#[strum(serialize_all = "UPPERCASE", ascii_case_insensitive)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
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

    pub(super) fn set_required(self, required: bool) -> WrappingType {
        if required {
            self.wrap_required()
        } else {
            self.unwrap_required()
        }
    }

    pub(super) fn unwrap_required(self) -> WrappingType {
        match self {
            WrappingType::NonNull(inner) => *inner,
            _ => self,
        }
    }

    pub fn contains_list(&self) -> bool {
        match self {
            WrappingType::NonNull(inner) => inner.contains_list(),
            WrappingType::List(_) => true,
            WrappingType::Named => false,
        }
    }

    // Returns the arity of this WrappingType if it is either obviously a many
    // or obviously a one, otherwise returns none
    pub fn arity(&self) -> Option<Arity> {
        let mut found_list = false;
        let mut current = self;
        loop {
            match current {
                WrappingType::NonNull(inner) => {
                    current = inner.as_ref();
                }
                WrappingType::List(_) if found_list => {
                    // If we've got a nested list we return None because that's not a _simple_ many
                    // and I do not know how we'd handle that.
                    return None;
                }
                WrappingType::List(inner) => {
                    found_list = true;
                    current = inner.as_ref();
                }
                WrappingType::Named if found_list => return Some(Arity::Many),
                WrappingType::Named => return Some(Arity::One),
            }
        }
    }
}

impl OpenApiGraph {
    fn type_name(&self, node: NodeIndex) -> Option<String> {
        match &self.graph[node] {
            schema @ Node::Schema { .. } => Some(self.metadata.namespaced(&schema.name()?).to_pascal_case()),
            Node::Operation(_) | Node::Default(_) | Node::PossibleValue(_) | Node::AllOf => None,
            Node::Object | Node::Enum { .. } => {
                // OpenAPI objects are generally anonymous so we walk back up the graph to the
                // nearest named thing, and construct a name based on the fields in-betweeen.
                // Not ideal, but the best we can do.
                let filtered_graph = EdgeFiltered::from_fn(&self.graph, |edge| {
                    matches!(
                        edge.weight(),
                        Edge::HasField { .. }
                            | Edge::HasPathParameter { .. }
                            | Edge::HasQueryParameter { .. }
                            | Edge::HasRequestType { .. }
                            | Edge::HasResponseType { .. }
                            | Edge::HasType { .. }
                            | Edge::HasUnionMember
                    )
                });
                let filtered_reversed_graph = Reversed(&filtered_graph);

                let (_, mut path) = petgraph::algo::astar(
                    &filtered_reversed_graph,
                    node,
                    |current_node| self.graph[current_node].name().is_some(),
                    |_| 0,
                    |_| 0,
                )?;

                let named_node = *path.last()?;

                // Reverse our path so we can look things up in the original graph.
                path.reverse();

                let mut name_components = Vec::new();
                let mut path_iter = path.into_iter().peekable();
                while let Some(src_node) = path_iter.next() {
                    let Some(&dest_node) = path_iter.peek() else {
                        break;
                    };

                    name_components.extend(self.graph.edges_connecting(src_node, dest_node).find_map(|edge| {
                        match edge.weight() {
                            Edge::HasField { name, .. }
                            | Edge::HasPathParameter { name, .. }
                            | Edge::HasQueryParameter { name, .. } => Some(Cow::Borrowed(name.as_str())),
                            _ => None,
                        }
                    }));
                }

                let root_name = self.graph[named_node].name().unwrap();
                name_components.push(Cow::Borrowed(root_name.as_str()));
                name_components.push(Cow::Owned(self.metadata.unique_namespace()));
                name_components.reverse();

                Some(name_components.join("_").to_pascal_case())
            }
            Node::Scalar(kind) => Some(kind.type_name()),
            Node::PlaceholderType => {
                // Any placeholders that make it this far should just be mapped to JSON.
                Some(ScalarKind::Json.type_name())
            }
            Node::UnionWrappedScalar(kind) => Some(self.metadata.namespaced(&kind.type_name()).to_pascal_case()),
            Node::Union => {
                // First we check if this union has an immediate schema parent.
                // If so we use it's name for the union
                let reversed_graph = Reversed(&self.graph);
                if let Some(name) = reversed_graph
                    .edges(node)
                    .find(|edge| matches!(edge.weight(), Edge::HasType { .. }))
                    .and_then(|edge| self.graph[edge.target()].name())
                {
                    return Some(self.metadata.namespaced(&name).to_pascal_case());
                }

                // Unions are named based on the names of their constituent types.
                let name_components = self
                    .graph
                    .edges(node)
                    .filter_map(|edge| match edge.weight() {
                        Edge::HasUnionMember => OutputType::from_index(edge.target(), self)?.name(self),
                        _ => None,
                    })
                    .collect::<Vec<_>>();

                let prefix = self.metadata.unique_namespace().to_pascal_case();
                let name_components = name_components
                    .iter()
                    .map(|name| name.strip_prefix(&prefix).unwrap_or(name))
                    .collect::<Vec<_>>();

                let mut name = prefix;
                name.push_str(&name_components.join("Or"));
                name.push_str("Union");

                Some(name)
            }
        }
    }

    // Finds the type node of a schema node
    fn schema_target(&self, schema_index: NodeIndex) -> Option<NodeIndex> {
        Some(
            self.graph
                .edges(schema_index)
                .find(|edge| matches!(edge.weight(), Edge::HasType { .. }))?
                .target(),
        )
    }
}

impl Node {
    // Used to determine whether this specific node type has a name.
    // To generate the full name of a particular node you should use the OpenApiGraph::type_name
    // function.
    fn name(&self) -> Option<String> {
        match self {
            Node::Schema(schema) => Some(
                // We either use the resourceId if it's present or just take the name of the schema
                // in the document
                schema
                    .resource_id
                    .as_deref()
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

impl<'a> FieldName<'a> {
    pub fn from_openapi_name(name: &'a str) -> Self {
        FieldName(Cow::Borrowed(name))
    }

    pub fn openapi_name(&self) -> &str {
        self.0.as_ref()
    }

    pub fn will_be_valid_graphql(&self) -> bool {
        static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("^[A-Za-z_][A-Za-z0-9_]*$").unwrap());
        REGEX.is_match(&self.0.to_camel_case())
    }
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

    #[test]
    fn test_will_be_valid_graphql() {
        assert!(!FieldName::from_openapi_name("+1").will_be_valid_graphql());
        assert!(!FieldName::from_openapi_name("-1").will_be_valid_graphql());
        assert!(FieldName::from_openapi_name("some_field").will_be_valid_graphql());
        assert!(FieldName::from_openapi_name("someField").will_be_valid_graphql());
        assert!(FieldName::from_openapi_name("someField123").will_be_valid_graphql());
    }

    #[test]
    fn test_graph_size() {
        // Our graph can end up with a ton of nodes & edges so it's important
        // that they don't get too big.
        assert!(std::mem::size_of::<Node>() <= 48);
        assert!(std::mem::size_of::<Edge>() <= 48);
    }
}
