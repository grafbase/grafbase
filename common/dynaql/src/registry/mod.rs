mod cache_control;
mod export_sdl;
pub mod relations;
pub mod resolvers;
pub mod scalars;
mod stringify_exec_doc;
pub mod utils;
pub mod variables;

use indexmap::map::IndexMap;
use indexmap::set::IndexSet;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::Hash;

use crate::auth::{AuthConfig, Operations};

pub use crate::model::__DirectiveLocation;

use crate::parser::types::{
    BaseType as ParsedBaseType, Field, Type as ParsedType, VariableDefinition,
};

use crate::validation::dynamic_validators::DynValidator;
use crate::{
    model, Any, Context, InputType, OutputType, Positioned, ServerResult, SubscriptionType, Value,
    VisitorContext,
};

pub use cache_control::CacheControl;

use self::relations::MetaRelation;
use self::resolvers::Resolver;

fn strip_brackets(type_name: &str) -> Option<&str> {
    type_name
        .strip_prefix('[')
        .map(|rest| &rest[..rest.len() - 1])
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MetaTypeName<'a> {
    List(&'a str),
    NonNull(&'a str),
    Named(&'a str),
}

impl<'a> std::fmt::Display for MetaTypeName<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetaTypeName::Named(name) => write!(f, "{name}"),
            MetaTypeName::NonNull(name) => write!(f, "{name}!"),
            MetaTypeName::List(name) => write!(f, "[{name}]"),
        }
    }
}

impl<'a> MetaTypeName<'a> {
    #[inline]
    pub fn create(type_name: &str) -> MetaTypeName {
        if let Some(type_name) = type_name.strip_suffix('!') {
            MetaTypeName::NonNull(type_name)
        } else if let Some(type_name) = strip_brackets(type_name) {
            MetaTypeName::List(type_name)
        } else {
            MetaTypeName::Named(type_name)
        }
    }

    #[inline]
    pub fn concrete_typename(type_name: &str) -> &str {
        match MetaTypeName::create(type_name) {
            MetaTypeName::List(type_name) => Self::concrete_typename(type_name),
            MetaTypeName::NonNull(type_name) => Self::concrete_typename(type_name),
            MetaTypeName::Named(type_name) => type_name,
        }
    }

    #[inline]
    pub fn is_non_null(&self) -> bool {
        matches!(self, MetaTypeName::NonNull(_))
    }

    #[inline]
    #[must_use]
    pub fn unwrap_non_null(&self) -> Self {
        match self {
            MetaTypeName::NonNull(ty) => MetaTypeName::create(ty),
            _ => *self,
        }
    }

    #[inline]
    pub fn is_subtype(&self, sub: &MetaTypeName<'_>) -> bool {
        match (self, sub) {
            (
                MetaTypeName::NonNull(super_type) | MetaTypeName::Named(super_type),
                MetaTypeName::NonNull(sub_type),
            ) => MetaTypeName::create(super_type).is_subtype(&MetaTypeName::create(sub_type)),
            (MetaTypeName::Named(super_type), MetaTypeName::Named(sub_type)) => {
                super_type == sub_type
            }
            (MetaTypeName::List(super_type), MetaTypeName::List(sub_type)) => {
                MetaTypeName::create(super_type).is_subtype(&MetaTypeName::create(sub_type))
            }
            _ => false,
        }
    }

    #[inline]
    pub fn is_list(&self) -> bool {
        match self {
            MetaTypeName::List(_) => true,
            MetaTypeName::NonNull(ty) => MetaTypeName::create(ty).is_list(),
            MetaTypeName::Named(name) => name.ends_with(']'),
        }
    }
}

#[derive(derivative::Derivative, Clone, serde::Deserialize, serde::Serialize)]
#[derivative(Debug, Hash, PartialEq)]
pub struct MetaInputValue {
    pub name: String,
    pub description: Option<String>,
    pub ty: String,
    #[derivative(Hash = "ignore")]
    pub default_value: Option<dynaql_value::ConstValue>,
    #[serde(skip)]
    #[derivative(Debug = "ignore", Hash = "ignore", PartialEq = "ignore")]
    pub visible: Option<MetaVisibleFn>,
    #[derivative(Debug = "ignore", Hash = "ignore", PartialEq = "ignore")]
    pub validators: Option<Vec<DynValidator>>,
    pub is_secret: bool,
}

impl Eq for MetaInputValue {}

type ComputeComplexityFn = fn(
    &VisitorContext<'_>,
    &[Positioned<VariableDefinition>],
    &Field,
    usize,
) -> ServerResult<usize>;

#[derive(Clone)]
pub enum ComplexityType {
    Const(usize),
    Fn(ComputeComplexityFn),
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub enum Deprecation {
    NoDeprecated,
    Deprecated { reason: Option<String> },
}

impl Default for Deprecation {
    fn default() -> Self {
        Deprecation::NoDeprecated
    }
}

impl Deprecation {
    #[inline]
    pub fn is_deprecated(&self) -> bool {
        matches!(self, Deprecation::Deprecated { .. })
    }

    #[inline]
    pub fn reason(&self) -> Option<&str> {
        match self {
            Deprecation::NoDeprecated => None,
            Deprecation::Deprecated { reason } => reason.as_deref(),
        }
    }
}

#[derive(
    Clone, Debug, derivative::Derivative, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq,
)]
pub enum ConstraintType {
    Unique,
}

#[derive(
    Clone, Debug, derivative::Derivative, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq,
)]
pub struct Constraint {
    pub field: String,
    pub r#type: ConstraintType,
}

impl From<Constraint> for dynamodb::export::graph_entities::ConstraintDefinition {
    fn from(Constraint { field, r#type }: Constraint) -> Self {
        Self {
            field,
            r#type: match r#type {
                ConstraintType::Unique => dynamodb::export::graph_entities::ConstraintType::Unique,
            },
        }
    }
}

#[derive(Clone, derivative::Derivative, serde::Deserialize, serde::Serialize)]
#[derivative(Debug)]
pub struct MetaField {
    pub name: String,
    pub description: Option<String>,
    pub args: IndexMap<String, MetaInputValue>,
    pub ty: String,
    pub deprecation: Deprecation,
    pub cache_control: CacheControl,
    pub external: bool,
    pub requires: Option<String>,
    pub provides: Option<String>,
    #[serde(skip)]
    #[derivative(Debug = "ignore")]
    pub visible: Option<MetaVisibleFn>,
    #[serde(skip)]
    #[derivative(Debug = "ignore")]
    pub compute_complexity: Option<ComplexityType>,
    /// Deprecated, to remove
    pub edges: Vec<String>,
    /// Define the relations of the Entity
    ///
    ///
    /// @todo: rename it to relations (String, String) where
    /// 0: RelationName,
    /// 1: Type,
    /// relation: (String, String)
    pub relation: Option<MetaRelation>,
    pub resolver: Option<Resolver>,
    pub required_operation: Option<Operations>,
    pub auth: Option<AuthConfig>,
}

impl Hash for MetaField {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.description.hash(state);
        self.args.as_slice().hash(state);
        self.ty.hash(state);
        self.deprecation.hash(state);
        self.cache_control.hash(state);
        self.external.hash(state);
        self.requires.hash(state);
        self.provides.hash(state);
        self.edges.hash(state);
        self.relation.hash(state);
        self.resolver.hash(state);
    }
}

impl PartialEq for MetaField {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
            && self.description.eq(&other.description)
            && self.args.as_slice().eq(&other.args.as_slice())
            && self.ty.eq(&other.ty)
            && self.deprecation.eq(&other.deprecation)
            && self.cache_control.eq(&other.cache_control)
            && self.external.eq(&other.external)
            && self.requires.eq(&other.requires)
            && self.provides.eq(&other.provides)
            && self.edges.eq(&other.edges)
            && self.relation.eq(&other.relation)
            && self.resolver.eq(&other.resolver)
    }
}

impl Eq for MetaField {}

/// Utility function
/// From a given type, check if the type is an Array Nullable/NonNullable.
pub fn is_array_basic_type(meta: &str) -> bool {
    let mut nested = Some(meta);

    if meta.starts_with('[') && meta.ends_with(']') {
        return true;
    }

    if meta.ends_with('!') {
        nested = nested.and_then(|x| x.strip_suffix('!'));
        return is_array_basic_type(nested.expect("Can't fail"));
    }

    false
}

/// Utility function
/// From a given type, get the base type.
pub fn get_basic_type(meta: &str) -> &str {
    let mut nested = Some(meta);

    if meta.starts_with('[') && meta.ends_with(']') {
        nested = nested.and_then(|x| x.strip_prefix('['));
        nested = nested.and_then(|x| x.strip_suffix(']'));
        return get_basic_type(nested.expect("Can't fail"));
    }

    if meta.ends_with('!') {
        nested = nested.and_then(|x| x.strip_suffix('!'));
        return get_basic_type(nested.expect("Can't fail"));
    }

    nested.expect("Can't fail")
}

impl MetaField {
    pub fn is_required(&self) -> bool {
        self.ty.ends_with('!')
    }

    pub fn is_array(&self) -> bool {
        self.ty.starts_with('[')
    }
}

#[derive(Clone, derivative::Derivative, serde::Serialize, serde::Deserialize)]
#[derivative(Debug, Hash, PartialEq)]
pub struct MetaEnumValue {
    pub name: String,
    pub description: Option<String>,
    pub deprecation: Deprecation,
    #[serde(skip)]
    #[derivative(Debug = "ignore", Hash = "ignore", PartialEq = "ignore")]
    pub visible: Option<MetaVisibleFn>,
}

impl Eq for MetaEnumValue {}

type MetaVisibleFn = fn(&Context<'_>) -> bool;

/// Define an Edge for a Node.
#[derive(Debug)]
pub struct Edge<'a>(pub &'a str);

impl<'a> ToString for Edge<'a> {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl MetaType {
    /// Get the edges of a current type.
    /// The edges are only for the top level.
    /// If one of the edge is a node with edges, those won't appear here.
    pub fn edges<'a>(&'a self) -> HashMap<&'a str, Vec<Edge<'a>>> {
        let mut result: HashMap<&'a str, Vec<Edge<'a>>> = HashMap::new();

        if let MetaType::Object { fields, .. } = self {
            for (field, ty) in fields {
                if !ty.edges.is_empty() {
                    let edges: Vec<Edge<'a>> = ty.edges.iter().map(|x| Edge(x.as_str())).collect();
                    result.insert(field.as_str(), edges);
                }
            }
        };

        result
    }

    /// Get the relations of a current type, select relations based on the
    /// selected fields.
    pub fn relations_by_selection<'a>(
        &'a self,
        selected_fields: Vec<&str>,
    ) -> HashMap<&'a str, &'a MetaRelation> {
        let mut result: HashMap<&'a str, &'a MetaRelation> = HashMap::new();

        if let MetaType::Object { fields, .. } = self {
            for (field, ty) in fields
                .iter()
                .filter(|f| selected_fields.contains(&f.0.as_str()))
            {
                if let Some(relation) = &ty.relation {
                    result.insert(field.as_str(), relation);
                }
            }
        };

        result
    }

    /// Get the relations of a current type
    pub fn relations<'a>(&'a self) -> IndexMap<&'a str, &'a MetaRelation> {
        let mut result: IndexMap<&'a str, &'a MetaRelation> = IndexMap::new();

        if let MetaType::Object { fields, .. } = self {
            for (field, ty) in fields {
                if let Some(relation) = &ty.relation {
                    result.insert(field.as_str(), relation);
                }
            }
        };

        result
    }
}

#[derive(derivative::Derivative, Clone, serde::Serialize, serde::Deserialize)]
#[derivative(Debug)]
pub enum MetaType {
    Scalar {
        name: String,
        description: Option<String>,
        #[derivative(Debug = "ignore")]
        #[serde(skip)]
        is_valid: Option<fn(value: &Value) -> bool>,
        #[derivative(Debug = "ignore")]
        #[serde(skip)]
        visible: Option<MetaVisibleFn>,
        specified_by_url: Option<String>,
    },
    Object {
        name: String,
        description: Option<String>,
        fields: IndexMap<String, MetaField>,
        cache_control: CacheControl,
        extends: bool,
        keys: Option<Vec<String>>,
        #[derivative(Debug = "ignore")]
        #[serde(skip)]
        visible: Option<MetaVisibleFn>,
        is_subscription: bool,
        /// Define if the current Object if a Node
        is_node: bool,
        rust_typename: String,
        constraints: Vec<Constraint>,
    },
    Interface {
        name: String,
        description: Option<String>,
        fields: IndexMap<String, MetaField>,
        possible_types: IndexSet<String>,
        extends: bool,
        keys: Option<Vec<String>>,
        #[derivative(Debug = "ignore")]
        #[serde(skip)]
        visible: Option<MetaVisibleFn>,
        rust_typename: String,
    },
    Union {
        name: String,
        description: Option<String>,
        possible_types: IndexSet<String>,
        #[derivative(Debug = "ignore")]
        #[serde(skip)]
        visible: Option<MetaVisibleFn>,
        rust_typename: String,
    },
    Enum {
        name: String,
        description: Option<String>,
        enum_values: IndexMap<String, MetaEnumValue>,
        #[derivative(Debug = "ignore")]
        #[serde(skip)]
        visible: Option<MetaVisibleFn>,
        rust_typename: String,
    },
    InputObject {
        name: String,
        description: Option<String>,
        input_fields: IndexMap<String, MetaInputValue>,
        #[derivative(Debug = "ignore")]
        #[serde(skip)]
        visible: Option<MetaVisibleFn>,
        rust_typename: String,
        oneof: bool,
    },
}

// Hash custom implementation must be done as we can't derive Hash Indexmap Implementation.
impl Hash for MetaType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Scalar {
                name,
                description,
                specified_by_url,
                visible: _,
                is_valid: _,
            } => {
                name.hash(state);
                description.hash(state);
                specified_by_url.hash(state);
            }
            Self::Object {
                name,
                description,
                fields,
                cache_control,
                extends,
                keys,
                visible: _,
                is_subscription,
                is_node,
                rust_typename,
                constraints,
            } => {
                name.hash(state);
                description.hash(state);
                fields.as_slice().hash(state);
                cache_control.hash(state);
                extends.hash(state);
                keys.hash(state);
                is_subscription.hash(state);
                is_node.hash(state);
                rust_typename.hash(state);
                constraints.hash(state);
            }
            Self::Interface {
                name,
                description,
                fields,
                possible_types,
                extends,
                keys,
                visible: _,
                rust_typename,
            } => {
                name.hash(state);
                description.hash(state);
                fields.as_slice().hash(state);
                possible_types.as_slice().hash(state);
                extends.hash(state);
                keys.hash(state);
                rust_typename.hash(state);
            }
            Self::Enum {
                name,
                description,
                enum_values,
                visible: _,
                rust_typename,
            } => {
                name.hash(state);
                description.hash(state);
                enum_values.as_slice().hash(state);
                rust_typename.hash(state);
            }
            Self::Union {
                name,
                description,
                possible_types,
                visible: _,
                rust_typename,
            } => {
                name.hash(state);
                description.hash(state);
                possible_types.as_slice().hash(state);
                rust_typename.hash(state);
            }
            Self::InputObject {
                name,
                description,
                input_fields,
                visible: _,
                rust_typename,
                oneof,
            } => {
                name.hash(state);
                description.hash(state);
                input_fields.as_slice().hash(state);
                oneof.hash(state);
                rust_typename.hash(state);
            }
        }
    }
}

// PartialEq custom implementation must be done as we can't derive Hash Indexmap Implementation.
impl PartialEq for MetaType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Scalar {
                    name,
                    description,
                    specified_by_url,
                    visible: _,
                    is_valid: _,
                },
                Self::Scalar {
                    name: o_name,
                    description: o_descrition,
                    specified_by_url: o_specified_by_url,
                    visible: _,
                    is_valid: _,
                },
            ) => {
                name.eq(o_name)
                    && description.eq(o_descrition)
                    && specified_by_url.eq(o_specified_by_url)
            }
            (
                Self::Object {
                    name,
                    description,
                    fields,
                    cache_control,
                    extends,
                    keys,
                    visible: _,
                    is_subscription,
                    is_node,
                    rust_typename,
                    constraints,
                },
                Self::Object {
                    name: o_name,
                    description: o_description,
                    fields: o_fields,
                    cache_control: o_cache_control,
                    extends: o_extends,
                    keys: o_keys,
                    visible: _,
                    is_subscription: o_is_subscription,
                    is_node: o_is_node,
                    rust_typename: o_rust_typename,
                    constraints: o_constraints,
                },
            ) => {
                name.eq(o_name)
                    && description.eq(o_description)
                    && fields.as_slice().eq(o_fields.as_slice())
                    && cache_control.eq(o_cache_control)
                    && extends.eq(o_extends)
                    && keys.eq(o_keys)
                    && is_subscription.eq(o_is_subscription)
                    && is_node.eq(o_is_node)
                    && rust_typename.eq(o_rust_typename)
                    && constraints.eq(o_constraints)
            }
            (
                Self::Interface {
                    name,
                    description,
                    fields,
                    possible_types,
                    extends,
                    keys,
                    visible: _,
                    rust_typename,
                },
                Self::Interface {
                    name: o_name,
                    description: o_description,
                    fields: o_fields,
                    possible_types: o_possible_types,
                    extends: o_extends,
                    keys: o_keys,
                    visible: _,
                    rust_typename: o_rust_typename,
                },
            ) => {
                name.eq(o_name)
                    && description.eq(o_description)
                    && fields.as_slice().eq(o_fields.as_slice())
                    && possible_types.as_slice().eq(o_possible_types.as_slice())
                    && extends.eq(o_extends)
                    && keys.eq(o_keys)
                    && rust_typename.eq(o_rust_typename)
            }
            (
                Self::Enum {
                    name,
                    description,
                    enum_values,
                    visible: _,
                    rust_typename,
                },
                Self::Enum {
                    name: o_name,
                    description: o_description,
                    enum_values: o_enum_values,
                    visible: _,
                    rust_typename: o_rust_typename,
                },
            ) => {
                name.eq(o_name)
                    && description.eq(o_description)
                    && enum_values.as_slice().eq(o_enum_values.as_slice())
                    && rust_typename.eq(o_rust_typename)
            }
            (
                Self::Union {
                    name,
                    description,
                    possible_types,
                    visible: _,
                    rust_typename,
                },
                Self::Union {
                    name: o_name,
                    description: o_description,
                    possible_types: o_possible_types,
                    visible: _,
                    rust_typename: o_rust_typename,
                },
            ) => {
                name.eq(o_name)
                    && description.eq(o_description)
                    && possible_types.as_slice().eq(o_possible_types.as_slice())
                    && rust_typename.eq(o_rust_typename)
            }
            (
                Self::InputObject {
                    name,
                    description,
                    input_fields,
                    visible: _,
                    rust_typename,
                    oneof,
                },
                Self::InputObject {
                    name: o_name,
                    description: o_description,
                    input_fields: o_input_fields,
                    visible: _,
                    rust_typename: o_rust_typename,
                    oneof: o_oneof,
                },
            ) => {
                name.eq(o_name)
                    && description.eq(o_description)
                    && input_fields.as_slice().eq(o_input_fields.as_slice())
                    && oneof.eq(o_oneof)
                    && rust_typename.eq(o_rust_typename)
            }
            _ => false,
        }
    }
}

impl Eq for MetaType {}

impl MetaType {
    #[inline]
    pub fn field_by_name(&self, name: &str) -> Option<&MetaField> {
        self.fields().and_then(|fields| fields.get(name))
    }

    #[inline]
    pub fn fields(&self) -> Option<&IndexMap<String, MetaField>> {
        match self {
            MetaType::Object { fields, .. } => Some(&fields),
            MetaType::Interface { fields, .. } => Some(&fields),
            _ => None,
        }
    }

    pub fn constraints(&self) -> &[Constraint] {
        match self {
            MetaType::Object { constraints, .. } => &constraints,
            _ => &[],
        }
    }

    #[inline]
    pub fn is_visible(&self, ctx: &Context<'_>) -> bool {
        let visible = match self {
            MetaType::Scalar { visible, .. } => visible,
            MetaType::Object { visible, .. } => visible,
            MetaType::Interface { visible, .. } => visible,
            MetaType::Union { visible, .. } => visible,
            MetaType::Enum { visible, .. } => visible,
            MetaType::InputObject { visible, .. } => visible,
        };
        is_visible(ctx, visible)
    }

    #[inline]
    pub fn name(&self) -> &str {
        match self {
            MetaType::Scalar { name, .. } => &name,
            MetaType::Object { name, .. } => name,
            MetaType::Interface { name, .. } => name,
            MetaType::Union { name, .. } => name,
            MetaType::Enum { name, .. } => name,
            MetaType::InputObject { name, .. } => name,
        }
    }

    #[inline]
    pub fn description(&self) -> Option<&str> {
        match self {
            MetaType::Scalar { description, .. } => description.as_deref(),
            MetaType::Object { description, .. } => description.as_deref(),
            MetaType::Interface { description, .. } => description.as_deref(),
            MetaType::Union { description, .. } => description.as_deref(),
            MetaType::Enum { description, .. } => description.as_deref(),
            MetaType::InputObject { description, .. } => description.as_deref(),
        }
    }

    #[inline]
    pub fn is_composite(&self) -> bool {
        matches!(
            self,
            MetaType::Object { .. } | MetaType::Interface { .. } | MetaType::Union { .. }
        )
    }

    #[inline]
    pub fn is_abstract(&self) -> bool {
        matches!(self, MetaType::Interface { .. } | MetaType::Union { .. })
    }

    #[inline]
    pub fn is_leaf(&self) -> bool {
        matches!(self, MetaType::Enum { .. } | MetaType::Scalar { .. })
    }

    #[inline]
    pub fn is_input(&self) -> bool {
        matches!(
            self,
            MetaType::Enum { .. } | MetaType::Scalar { .. } | MetaType::InputObject { .. }
        )
    }

    #[inline]
    pub fn is_possible_type(&self, type_name: &str) -> bool {
        match self {
            MetaType::Interface { possible_types, .. } => possible_types.contains(type_name),
            MetaType::Union { possible_types, .. } => possible_types.contains(type_name),
            MetaType::Object { name, .. } => name == type_name,
            _ => false,
        }
    }

    #[inline]
    pub fn possible_types(&self) -> Option<&IndexSet<String>> {
        match self {
            MetaType::Interface { possible_types, .. } => Some(possible_types),
            MetaType::Union { possible_types, .. } => Some(possible_types),
            _ => None,
        }
    }

    pub fn type_overlap(&self, ty: &MetaType) -> bool {
        if std::ptr::eq(self, ty) {
            return true;
        }

        match (self.is_abstract(), ty.is_abstract()) {
            (true, true) => self
                .possible_types()
                .iter()
                .copied()
                .flatten()
                .any(|type_name| ty.is_possible_type(type_name)),
            (true, false) => self.is_possible_type(ty.name()),
            (false, true) => ty.is_possible_type(self.name()),
            (false, false) => false,
        }
    }

    pub fn rust_typename(&self) -> Option<&String> {
        match self {
            MetaType::Scalar { .. } => None,
            MetaType::Object { rust_typename, .. } => Some(rust_typename),
            MetaType::Interface { rust_typename, .. } => Some(rust_typename),
            MetaType::Union { rust_typename, .. } => Some(rust_typename),
            MetaType::Enum { rust_typename, .. } => Some(rust_typename),
            MetaType::InputObject { rust_typename, .. } => Some(rust_typename),
        }
    }
}

#[derive(Clone, derivative::Derivative, serde::Serialize, serde::Deserialize)]
#[derivative(Debug)]
pub struct MetaDirective {
    pub name: String,
    pub description: Option<String>,
    pub locations: Vec<model::__DirectiveLocation>,
    pub args: IndexMap<String, MetaInputValue>,
    pub is_repeatable: bool,
    #[derivative(Debug = "ignore")]
    #[serde(skip)]
    pub visible: Option<MetaVisibleFn>,
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct Registry {
    pub types: BTreeMap<String, MetaType>,
    pub directives: HashMap<String, MetaDirective>,
    pub implements: HashMap<String, HashSet<String>>,
    pub query_type: String,
    pub mutation_type: Option<String>,
    pub subscription_type: Option<String>,
    pub disable_introspection: bool,
    pub enable_federation: bool,
    pub federation_subscription: bool,
    pub auth: AuthConfig,
}

impl Registry {
    pub fn query_root(&self) -> &MetaType {
        self.types.get(&self.query_type).unwrap()
    }

    pub fn mutation_root(&self) -> &MetaType {
        // TODO: Fix this.
        self.types
            .get(self.mutation_type.as_deref().unwrap())
            .unwrap()
    }

    /// Introspection type name
    ///
    /// Is the return value of field `__typename`, the interface and union should return the current type, and the others return `Type::type_name`.
    pub fn introspection_type_name<'a>(&self, node: &'a MetaType) -> &'a str {
        node.name()
    }
}

impl Registry {
    pub fn create_input_type<T: InputType + ?Sized, F: FnMut(&mut Registry) -> MetaType>(
        &mut self,
        mut f: F,
    ) -> String {
        self.create_type(&mut f, &T::type_name(), std::any::type_name::<T>());
        T::qualified_type_name()
    }

    pub fn create_output_type<T: OutputType + ?Sized, F: FnMut(&mut Registry) -> MetaType>(
        &mut self,
        mut f: F,
    ) -> String {
        self.create_type(&mut f, &T::type_name(), std::any::type_name::<T>());
        T::qualified_type_name()
    }

    pub fn create_subscription_type<
        T: SubscriptionType + ?Sized,
        F: FnMut(&mut Registry) -> MetaType,
    >(
        &mut self,
        mut f: F,
    ) -> String {
        self.create_type(&mut f, &T::type_name(), std::any::type_name::<T>());
        T::qualified_type_name()
    }

    pub fn create_type<F: FnMut(&mut Registry) -> MetaType>(
        &mut self,
        f: &mut F,
        name: &str,
        rust_typename: &str,
    ) {
        match self.types.get(name) {
            Some(ty) => {
                if let Some(prev_typename) = ty.rust_typename() {
                    if prev_typename.ne("__fake_type__") && prev_typename.ne(rust_typename) {
                        panic!(
                            "`{}` and `{}` have the same GraphQL name `{}`",
                            prev_typename, rust_typename, name,
                        );
                    }
                }
            }
            None => {
                // Inserting a fake type before calling the function allows recursive types to exist.
                self.types.insert(
                    name.to_string(),
                    MetaType::Object {
                        name: String::new(),
                        description: None,
                        fields: Default::default(),
                        cache_control: Default::default(),
                        extends: false,
                        keys: None,
                        visible: None,
                        is_subscription: false,
                        is_node: false,
                        rust_typename: "__fake_type__".to_string(),
                        constraints: vec![],
                    },
                );
                let ty = f(self);
                *self.types.get_mut(name).unwrap() = ty;
            }
        }
    }

    pub fn create_fake_output_type<T: OutputType>(&mut self) -> MetaType {
        T::create_type_info(self);
        self.types
            .get(&*T::type_name())
            .cloned()
            .expect("You definitely encountered a bug!")
    }

    pub fn create_fake_input_type<T: InputType>(&mut self) -> MetaType {
        T::create_type_info(self);
        self.types
            .get(&*T::type_name())
            .cloned()
            .expect("You definitely encountered a bug!")
    }

    pub fn create_fake_subscription_type<T: SubscriptionType>(&mut self) -> MetaType {
        T::create_type_info(self);
        self.types
            .get(&*T::type_name())
            .cloned()
            .expect("You definitely encountered a bug!")
    }

    pub fn add_directive(&mut self, directive: MetaDirective) {
        self.directives
            .insert(directive.name.to_string(), directive);
    }

    pub fn add_implements(&mut self, ty: &str, interface: &str) {
        self.implements
            .entry(ty.to_string())
            .and_modify(|interfaces| {
                interfaces.insert(interface.to_string());
            })
            .or_insert({
                let mut interfaces = HashSet::new();
                interfaces.insert(interface.to_string());
                interfaces
            });
    }

    pub fn add_keys(&mut self, ty: &str, keys: &str) {
        let all_keys = match self.types.get_mut(ty) {
            Some(MetaType::Object { keys: all_keys, .. }) => all_keys,
            Some(MetaType::Interface { keys: all_keys, .. }) => all_keys,
            _ => return,
        };
        if let Some(all_keys) = all_keys {
            all_keys.push(keys.to_string());
        } else {
            *all_keys = Some(vec![keys.to_string()]);
        }
    }

    pub fn concrete_type_by_name(&self, type_name: &str) -> Option<&MetaType> {
        self.types.get(MetaTypeName::concrete_typename(type_name))
    }

    pub fn concrete_type_by_parsed_type(&self, query_type: &ParsedType) -> Option<&MetaType> {
        match &query_type.base {
            ParsedBaseType::Named(name) => self.types.get(name.as_str()),
            ParsedBaseType::List(ty) => self.concrete_type_by_parsed_type(ty),
        }
    }

    pub(crate) fn has_entities(&self) -> bool {
        self.types.values().any(|ty| match ty {
            MetaType::Object {
                keys: Some(keys), ..
            }
            | MetaType::Interface {
                keys: Some(keys), ..
            } => !keys.is_empty(),
            _ => false,
        })
    }

    /// Each type annotated with @key should be added to the _Entity union.
    /// If no types are annotated with the key directive, then the _Entity union
    /// and Query._entities field should be removed from the schema.
    ///
    /// [Reference](https://www.apollographql.com/docs/federation/federation-spec/#resolve-requests-for-entities).
    fn create_entity_type_and_root_field(&mut self) {
        let possible_types: IndexSet<String> = self
            .types
            .values()
            .filter_map(|ty| match ty {
                MetaType::Object {
                    name,
                    keys: Some(keys),
                    ..
                } if !keys.is_empty() => Some(name.clone()),
                MetaType::Interface {
                    name,
                    keys: Some(keys),
                    ..
                } if !keys.is_empty() => Some(name.clone()),
                _ => None,
            })
            .collect();

        if !possible_types.is_empty() {
            self.types.insert(
                "_Entity".to_string(),
                MetaType::Union {
                    name: "_Entity".to_string(),
                    description: None,
                    possible_types,
                    visible: None,
                    rust_typename: "dynaql::federation::Entity".to_string(),
                },
            );

            let query_root = self.types.get_mut(&self.query_type).unwrap();
            if let MetaType::Object { fields, .. } = query_root {
                fields.insert(
                    "_service".to_string(),
                    MetaField {
                        name: "_service".to_string(),
                        description: None,
                        args: Default::default(),
                        ty: "_Service!".to_string(),
                        deprecation: Default::default(),
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None,
                        visible: None,
                        edges: Vec::new(),
                        relation: None,
                        compute_complexity: None,
                        resolver: None,
                        required_operation: None,
                        auth: None,
                    },
                );

                fields.insert(
                    "_entities".to_string(),
                    MetaField {
                        name: "_entities".to_string(),
                        description: None,
                        args: {
                            let mut args = IndexMap::new();
                            args.insert(
                                "representations".to_string(),
                                MetaInputValue {
                                    name: "representations".to_string(),
                                    description: None,
                                    ty: "[_Any!]!".to_string(),
                                    default_value: None,
                                    validators: None,
                                    visible: None,
                                    is_secret: false,
                                },
                            );
                            args
                        },
                        ty: "[_Entity]!".to_string(),
                        deprecation: Default::default(),
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        edges: Vec::new(),
                        relation: None,
                        provides: None,
                        visible: None,
                        compute_complexity: None,
                        resolver: None,
                        required_operation: None,
                        auth: None,
                    },
                );
            }
        }
    }

    pub(crate) fn create_federation_types(&mut self) {
        <Any as InputType>::create_type_info(self);

        self.types.insert(
            "_Service".to_string(),
            MetaType::Object {
                name: "_Service".to_string(),
                description: None,
                fields: {
                    let mut fields = IndexMap::new();
                    fields.insert(
                        "sdl".to_string(),
                        MetaField {
                            name: "sdl".to_string(),
                            description: None,
                            args: Default::default(),
                            ty: "String".to_string(),
                            deprecation: Default::default(),
                            cache_control: Default::default(),
                            external: false,
                            requires: None,
                            provides: None,
                            visible: None,
                            compute_complexity: None,
                            edges: Vec::new(),
                            relation: None,
                            resolver: None,
                            required_operation: None,
                            auth: None,
                        },
                    );
                    fields
                },
                cache_control: Default::default(),
                extends: false,
                keys: None,
                visible: None,
                is_subscription: false,
                is_node: false,
                rust_typename: "dynaql::federation::Service".to_string(),
                constraints: vec![],
            },
        );

        self.create_entity_type_and_root_field();
    }

    pub fn names(&self) -> Vec<String> {
        let mut names = HashSet::new();

        for d in self.directives.values() {
            names.insert(d.name.to_string());
            names.extend(d.args.values().map(|arg| arg.name.to_string()));
        }

        for ty in self.types.values() {
            match ty {
                MetaType::Scalar { name, .. } | MetaType::Union { name, .. } => {
                    names.insert(name.clone());
                }
                MetaType::Object { name, fields, .. }
                | MetaType::Interface { name, fields, .. } => {
                    names.insert(name.clone());
                    names.extend(
                        fields
                            .values()
                            .map(|field| {
                                std::iter::once(field.name.clone())
                                    .chain(field.args.values().map(|arg| arg.name.to_string()))
                            })
                            .flatten(),
                    );
                }
                MetaType::Enum {
                    name, enum_values, ..
                } => {
                    names.insert(name.clone());
                    names.extend(enum_values.values().map(|value| value.name.to_string()));
                }
                MetaType::InputObject {
                    name, input_fields, ..
                } => {
                    names.insert(name.clone());
                    names.extend(input_fields.values().map(|field| field.name.to_string()));
                }
            }
        }

        names.into_iter().collect()
    }

    pub fn set_description(&mut self, name: &str, desc: &'static str) {
        match self.types.get_mut(name) {
            Some(MetaType::Scalar { description, .. }) => *description = Some(desc.to_string()),
            Some(MetaType::Object { description, .. }) => *description = Some(desc.to_string()),
            Some(MetaType::Interface { description, .. }) => *description = Some(desc.to_string()),
            Some(MetaType::Union { description, .. }) => *description = Some(desc.to_string()),
            Some(MetaType::Enum { description, .. }) => *description = Some(desc.to_string()),
            Some(MetaType::InputObject { description, .. }) => {
                *description = Some(desc.to_string());
            }
            None => {}
        }
    }

    pub fn remove_unused_types(&mut self) {
        let mut used_types = BTreeSet::new();
        let mut unused_types = BTreeSet::new();

        fn traverse_field<'a>(
            types: &'a BTreeMap<String, MetaType>,
            used_types: &mut BTreeSet<&'a str>,
            field: &'a MetaField,
        ) {
            traverse_type(
                types,
                used_types,
                MetaTypeName::concrete_typename(&field.ty),
            );
            for arg in field.args.values() {
                traverse_input_value(types, used_types, arg);
            }
        }

        fn traverse_input_value<'a>(
            types: &'a BTreeMap<String, MetaType>,
            used_types: &mut BTreeSet<&'a str>,
            input_value: &'a MetaInputValue,
        ) {
            traverse_type(
                types,
                used_types,
                MetaTypeName::concrete_typename(&input_value.ty),
            );
        }

        fn traverse_type<'a>(
            types: &'a BTreeMap<String, MetaType>,
            used_types: &mut BTreeSet<&'a str>,
            type_name: &'a str,
        ) {
            if used_types.contains(type_name) {
                return;
            }

            if let Some(ty) = types.get(type_name) {
                used_types.insert(type_name);
                match ty {
                    MetaType::Object { fields, .. } => {
                        for field in fields.values() {
                            traverse_field(types, used_types, field);
                        }
                    }
                    MetaType::Interface {
                        fields,
                        possible_types,
                        ..
                    } => {
                        for field in fields.values() {
                            traverse_field(types, used_types, field);
                        }
                        for type_name in possible_types.iter() {
                            traverse_type(types, used_types, type_name);
                        }
                    }
                    MetaType::Union { possible_types, .. } => {
                        for type_name in possible_types.iter() {
                            traverse_type(types, used_types, type_name);
                        }
                    }
                    MetaType::InputObject { input_fields, .. } => {
                        for field in input_fields.values() {
                            traverse_input_value(types, used_types, field);
                        }
                    }
                    _ => {}
                }
            }
        }

        for directive in self.directives.values() {
            for arg in directive.args.values() {
                traverse_input_value(&self.types, &mut used_types, arg);
            }
        }

        for type_name in Some(&self.query_type)
            .into_iter()
            .chain(self.mutation_type.iter())
            .chain(self.subscription_type.iter())
        {
            traverse_type(&self.types, &mut used_types, type_name);
        }

        for ty in self.types.values().filter(|ty| match ty {
            MetaType::Object {
                keys: Some(keys), ..
            }
            | MetaType::Interface {
                keys: Some(keys), ..
            } => !keys.is_empty(),
            _ => false,
        }) {
            traverse_type(&self.types, &mut used_types, ty.name());
        }

        for ty in self.types.values() {
            let name = ty.name();
            if !is_system_type(name) && !used_types.contains(name) {
                unused_types.insert(name.to_string());
            }
        }

        for type_name in unused_types {
            self.types.remove(&type_name);
        }
    }

    pub fn find_visible_types(&self, ctx: &Context<'_>) -> HashSet<&str> {
        let mut visible_types = HashSet::new();

        fn traverse_field<'a>(
            ctx: &Context<'_>,
            types: &'a BTreeMap<String, MetaType>,
            visible_types: &mut HashSet<&'a str>,
            field: &'a MetaField,
        ) {
            if !is_visible(ctx, &field.visible) {
                return;
            }

            traverse_type(
                ctx,
                types,
                visible_types,
                MetaTypeName::concrete_typename(&field.ty),
            );
            for arg in field.args.values() {
                traverse_input_value(ctx, types, visible_types, arg);
            }
        }

        fn traverse_input_value<'a>(
            ctx: &Context<'_>,
            types: &'a BTreeMap<String, MetaType>,
            visible_types: &mut HashSet<&'a str>,
            input_value: &'a MetaInputValue,
        ) {
            if !is_visible(ctx, &input_value.visible) {
                return;
            }

            traverse_type(
                ctx,
                types,
                visible_types,
                MetaTypeName::concrete_typename(&input_value.ty),
            );
        }

        fn traverse_type<'a>(
            ctx: &Context<'_>,
            types: &'a BTreeMap<String, MetaType>,
            visible_types: &mut HashSet<&'a str>,
            type_name: &'a str,
        ) {
            if visible_types.contains(type_name) {
                return;
            }

            if let Some(ty) = types.get(type_name) {
                if !ty.is_visible(ctx) {
                    return;
                }

                visible_types.insert(type_name);
                match ty {
                    MetaType::Object { fields, .. } => {
                        for field in fields.values() {
                            traverse_field(ctx, types, visible_types, field);
                        }
                    }
                    MetaType::Interface {
                        fields,
                        possible_types,
                        ..
                    } => {
                        for field in fields.values() {
                            traverse_field(ctx, types, visible_types, field);
                        }
                        for type_name in possible_types.iter() {
                            traverse_type(ctx, types, visible_types, type_name);
                        }
                    }
                    MetaType::Union { possible_types, .. } => {
                        for type_name in possible_types.iter() {
                            traverse_type(ctx, types, visible_types, type_name);
                        }
                    }
                    MetaType::InputObject { input_fields, .. } => {
                        for field in input_fields.values() {
                            traverse_input_value(ctx, types, visible_types, field);
                        }
                    }
                    _ => {}
                }
            }
        }

        for directive in self.directives.values() {
            if is_visible(ctx, &directive.visible) {
                for arg in directive.args.values() {
                    traverse_input_value(ctx, &self.types, &mut visible_types, arg);
                }
            }
        }

        for type_name in Some(&self.query_type)
            .into_iter()
            .chain(self.mutation_type.iter())
            .chain(self.subscription_type.iter())
        {
            traverse_type(ctx, &self.types, &mut visible_types, type_name);
        }

        for ty in self.types.values().filter(|ty| match ty {
            MetaType::Object {
                keys: Some(keys), ..
            }
            | MetaType::Interface {
                keys: Some(keys), ..
            } => !keys.is_empty(),
            _ => false,
        }) {
            traverse_type(ctx, &self.types, &mut visible_types, ty.name());
        }

        for ty in self.types.values() {
            if let MetaType::Interface { possible_types, .. } = ty {
                if ty.is_visible(ctx) && !visible_types.contains(ty.name()) {
                    for type_name in possible_types.iter() {
                        if visible_types.contains(type_name.as_str()) {
                            traverse_type(ctx, &self.types, &mut visible_types, ty.name());
                            break;
                        }
                    }
                }
            }
        }

        self.types
            .values()
            .filter_map(|ty| {
                let name = ty.name();
                if is_system_type(name) || visible_types.contains(name) {
                    Some(name)
                } else {
                    None
                }
            })
            .collect()
    }
}

pub(crate) fn is_visible(ctx: &Context<'_>, visible: &Option<MetaVisibleFn>) -> bool {
    match visible {
        Some(f) => f(ctx),
        None => true,
    }
}

fn is_system_type(name: &str) -> bool {
    if name.starts_with("__") {
        return true;
    }

    name == "Boolean" || name == "Int" || name == "Float" || name == "String" || name == "ID"
}
