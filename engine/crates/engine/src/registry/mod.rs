pub mod builder;
mod cache_control;
mod connector_headers;
pub mod enums;
mod export_sdl;
pub mod federation;
pub mod field_set;
pub mod relations;
pub mod resolvers;
pub mod scalars;
mod serde_preserve_enum;
mod stringify_exec_doc;
pub mod type_kinds;
mod type_names;
pub mod union_discriminator;
pub mod utils;
pub mod variables;

#[cfg(test)]
mod tests;

use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fmt::{Display, Formatter},
    hash::Hash,
    sync::atomic::AtomicU16,
};

use common_types::auth::Operations;
use engine_parser::types::OperationType::{self, Query};
use engine_value::ConstValue;
use graph_entities::NodeID;
use indexmap::{map::IndexMap, set::IndexSet};
use inflector::Inflector;
use postgres_connector_types::database_definition::DatabaseDefinition;
use serde::{Deserialize, Serialize};

pub use self::{
    cache_control::{
        CacheAccessScope, CacheControl, CacheControlError, CacheInvalidation, CacheInvalidationPolicy,
        CachePartialRegistry,
    },
    connector_headers::{ConnectorHeaderValue, ConnectorHeaders},
    field_set::FieldSet,
    type_names::{
        InputValueType, MetaFieldType, ModelName, NamedType, TypeCondition, TypeReference, WrappingType,
        WrappingTypeIter,
    },
    union_discriminator::UnionDiscriminator,
};
use self::{
    federation::FederationEntity,
    relations::MetaRelation,
    resolvers::Resolver,
    type_kinds::{SelectionSetTarget, TypeKind},
};
pub use crate::model::__DirectiveLocation;
use crate::{
    auth::AuthConfig,
    model,
    parser::types::{BaseType as ParsedBaseType, Field, Type as ParsedType, VariableDefinition},
    validation::dynamic_validators::DynValidator,
    Any, ContextExt, ContextField, Error, LegacyInputType, LegacyOutputType, Positioned, ServerResult,
    SubscriptionType, Value, VisitorContext,
};

fn strip_brackets(type_name: &str) -> Option<&str> {
    type_name.strip_prefix('[').map(|rest| &rest[..rest.len() - 1])
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
            (MetaTypeName::NonNull(super_type) | MetaTypeName::Named(super_type), MetaTypeName::NonNull(sub_type)) => {
                MetaTypeName::create(super_type).is_subtype(&MetaTypeName::create(sub_type))
            }
            (MetaTypeName::Named(super_type), MetaTypeName::Named(sub_type)) => super_type == sub_type,
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

#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::skip_serializing_defaults(Option, bool)]
#[derive(derivative::Derivative, Clone, serde::Deserialize, serde::Serialize)]
#[derivative(Debug, Hash, PartialEq)]
pub struct MetaInputValue {
    pub name: String,
    pub description: Option<String>,
    pub ty: InputValueType,
    #[derivative(Hash = "ignore")]
    #[serde(with = "serde_preserve_enum")]
    pub default_value: Option<engine_value::ConstValue>,
    #[serde(skip)]
    #[derivative(Debug = "ignore", Hash = "ignore", PartialEq = "ignore")]
    pub visible: Option<MetaVisibleFn>,
    #[derivative(Debug = "ignore", Hash = "ignore", PartialEq = "ignore")]
    pub validators: Option<Vec<DynValidator>>,
    pub is_secret: bool,
    pub rename: Option<String>,
}

impl MetaInputValue {
    pub fn new(name: impl Into<String>, ty: impl Into<InputValueType>) -> MetaInputValue {
        MetaInputValue {
            name: name.into(),
            description: None,
            ty: ty.into(),
            default_value: None,
            visible: None,
            validators: None,
            is_secret: false,
            rename: None,
        }
    }

    pub fn with_description(self, description: impl Into<String>) -> MetaInputValue {
        MetaInputValue {
            description: Some(description.into()),
            ..self
        }
    }

    pub fn with_rename(self, rename: Option<String>) -> MetaInputValue {
        MetaInputValue { rename, ..self }
    }

    pub fn with_default(self, default: ConstValue) -> MetaInputValue {
        MetaInputValue {
            default_value: Some(default),
            ..self
        }
    }
}

impl Eq for MetaInputValue {}

type ComputeComplexityFn =
    fn(&VisitorContext<'_>, &[Positioned<VariableDefinition>], &Field, usize) -> ServerResult<usize>;

#[derive(Clone)]
pub enum ComplexityType {
    Const(usize),
    Fn(ComputeComplexityFn),
}

#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::minify_variant_names(serialize = "minified", deserialize = "minified")]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq, Default)]
pub enum Deprecation {
    #[default]
    NoDeprecated,
    Deprecated {
        reason: Option<String>,
    },
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

#[derive(Clone, Debug, derivative::Derivative, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq, Default)]
pub enum ConstraintType {
    #[default]
    Unique,
}

#[derive(Clone, Debug, derivative::Derivative, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::skip_serializing_defaults(Option, Vec, ConstraintType)]
pub struct Constraint {
    // This is an option for backwards compatability reasons.
    // Constraints didn't always have a name.
    // Can possibly make it required in the future.
    name: Option<String>,
    fields: Vec<String>,
    // This is also here for backwards compatability
    field: String,
    pub r#type: ConstraintType,
}

impl Constraint {
    pub fn name(&self) -> &str {
        self.name
            .as_deref()
            .or_else(|| Some(self.fields.first()?))
            .unwrap_or(&self.field)
    }

    pub fn fields(&self) -> Vec<String> {
        if self.fields.is_empty() {
            return vec![self.field.clone()];
        }
        self.fields.clone()
    }

    pub fn unique(name: String, fields: Vec<String>) -> Constraint {
        Constraint {
            name: Some(name),
            fields,
            field: String::new(),
            r#type: ConstraintType::Unique,
        }
    }
}

impl From<Constraint> for dynamodb::export::graph_entities::ConstraintDefinition {
    fn from(constraint: Constraint) -> Self {
        Self {
            fields: constraint.fields(),
            r#type: match constraint.r#type {
                ConstraintType::Unique => dynamodb::export::graph_entities::ConstraintType::Unique,
            },
        }
    }
}

#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::skip_serializing_defaults(Option, Vec, bool, CacheControl, Deprecation)]
#[derive(Clone, Default, derivative::Derivative, serde::Deserialize, serde::Serialize)]
#[derivative(Debug)]
pub struct MetaField {
    pub name: String,
    pub mapped_name: Option<String>,
    pub description: Option<String>,
    pub args: IndexMap<String, MetaInputValue>,
    pub ty: MetaFieldType,
    pub deprecation: Deprecation,
    pub cache_control: CacheControl,
    pub external: bool,
    pub shareable: bool,
    pub requires: Option<FieldSet>,
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
    #[serde(skip_serializing_if = "Resolver::is_parent", default)]
    pub resolver: Resolver,
    pub required_operation: Option<Operations>,
    pub auth: Option<AuthConfig>,
    pub r#override: Option<String>,
    pub tags: Vec<String>,
    pub inaccessible: bool,
}

impl MetaField {
    pub fn new(name: impl Into<String>, ty: impl Into<MetaFieldType>) -> MetaField {
        MetaField {
            name: name.into(),
            ty: ty.into(),
            ..Default::default()
        }
    }

    pub fn with_cache_control(self, cache_control: CacheControl) -> Self {
        Self { cache_control, ..self }
    }

    pub fn target_field_name(&self) -> &str {
        self.mapped_name.as_deref().unwrap_or(&self.name)
    }
}

impl Hash for MetaField {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.mapped_name.hash(state);
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
            && self.args.as_slice().eq(other.args.as_slice())
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

pub enum CacheTag {
    Type {
        type_name: String,
    },
    List {
        type_name: String,
    },
    Field {
        type_name: String,
        field_name: String,
        value: String,
    },
}

impl From<CacheTag> for String {
    fn from(value: CacheTag) -> Self {
        value.to_string()
    }
}

impl Display for CacheTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            CacheTag::Type { type_name } => f.write_str(type_name),
            CacheTag::List { type_name } => write!(f, "{type_name}#List"),
            CacheTag::Field {
                type_name,
                field_name,
                value,
            } => write!(f, "{type_name}#{field_name}:{value}"),
        }
    }
}

impl MetaField {
    pub async fn check_cache_tag(
        &self,
        ctx: &ContextField<'_>,
        resolved_field_type: &str,
        resolved_field_name: &str,
        resolved_field_value: Option<&ConstValue>,
    ) {
        use crate::names::{
            DELETE_PAYLOAD_RETURN_TY_SUFFIX, OUTPUT_FIELD_DELETED_ID, OUTPUT_FIELD_DELETED_IDS, OUTPUT_FIELD_ID,
        };
        let cache_invalidation = ctx
            .query_env
            .cache_invalidations
            .iter()
            .find(|cache_invalidation| cache_invalidation.ty == resolved_field_type);

        if let Some(cache_invalidation) = cache_invalidation {
            let mut cache_type = cache_invalidation.ty.clone();

            // This is very specific to deletions, not all queries return the @cache type ...
            // Reads, Creates and Updates do return the @cache type but Deletes do not.
            // Deletions return a `xDeletionPayload` with only a `deletedId`
            if cache_invalidation.ty.ends_with(DELETE_PAYLOAD_RETURN_TY_SUFFIX) {
                cache_type = cache_invalidation.ty.replace(DELETE_PAYLOAD_RETURN_TY_SUFFIX, "");
            }

            let cache_tags = match &cache_invalidation.policy {
                CacheInvalidationPolicy::Entity { field: target_field } => {
                    if target_field == resolved_field_name
                    // Deletions return a `xDeletionPayload` with only a `deletedId`
                    // If an invalidation policy is of type `entity.id`, on deletes `id` is the `deletedId`
                    || (target_field == OUTPUT_FIELD_ID && resolved_field_name == OUTPUT_FIELD_DELETED_ID)
                    {
                        let Some(resolved_field_value) = resolved_field_value else {
                            log::warn!(
                                ctx.trace_id(),
                                "missing field valued for resolved {}#{} and cache type {}",
                                resolved_field_type,
                                resolved_field_name,
                                cache_invalidation.ty,
                            );

                            return;
                        };

                        let resolved_field_value = match resolved_field_value {
                            // remove double quotes
                            ConstValue::String(quoted_string) => quoted_string.as_str().to_string(),
                            value => value.to_string(),
                        };

                        vec![CacheTag::Field {
                            type_name: cache_type,
                            field_name: target_field.to_string(),
                            value: resolved_field_value,
                        }]
                    } else if target_field == OUTPUT_FIELD_ID && OUTPUT_FIELD_DELETED_IDS == resolved_field_name {
                        let ids = Vec::<String>::deserialize(resolved_field_value.unwrap_or(&ConstValue::Null).clone())
                            .unwrap_or_default();

                        ids.into_iter()
                            .map(|value| CacheTag::Field {
                                type_name: cache_type.clone(),
                                field_name: target_field.to_string(),
                                value,
                            })
                            .collect()
                    } else {
                        return;
                    }
                }
                CacheInvalidationPolicy::List => vec![CacheTag::List { type_name: cache_type }],
                CacheInvalidationPolicy::Type => vec![CacheTag::Type { type_name: cache_type }],
            };

            ctx.response().await.add_cache_tags(cache_tags);
        }
    }
}

#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::skip_serializing_defaults(Option, Deprecation)]
#[derive(Clone, derivative::Derivative, serde::Serialize, serde::Deserialize)]
#[derivative(Debug, Hash, PartialEq)]
pub struct MetaEnumValue {
    pub name: String,
    pub description: Option<String>,
    pub deprecation: Deprecation,
    #[serde(skip)]
    #[derivative(Debug = "ignore", Hash = "ignore", PartialEq = "ignore")]
    pub visible: Option<MetaVisibleFn>,
    // The value that will be used for this MetaEnumValue when sent to a
    // non-GraphQL downstream API
    pub value: Option<String>,
}

impl MetaEnumValue {
    pub fn new(name: String) -> Self {
        MetaEnumValue {
            name,
            description: None,
            deprecation: Deprecation::NoDeprecated,
            visible: None,
            value: None,
        }
    }

    pub fn with_description(self, description: Option<String>) -> Self {
        MetaEnumValue { description, ..self }
    }

    pub fn with_deprecation(self, deprecation: Deprecation) -> Self {
        MetaEnumValue { deprecation, ..self }
    }
}

impl Eq for MetaEnumValue {}

type MetaVisibleFn = fn(&ContextField<'_>) -> bool;

/// Define an Edge for a Node.
#[derive(Debug)]
pub struct Edge<'a>(pub &'a str);

impl<'a> ToString for Edge<'a> {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl ObjectType {
    /// Get the relations of a current type
    pub fn relations<'a>(&'a self) -> IndexMap<&'a str, &'a MetaRelation> {
        let mut result: IndexMap<&'a str, &'a MetaRelation> = IndexMap::new();

        for (field, ty) in &self.fields {
            if let Some(relation) = &ty.relation {
                result.insert(field.as_str(), relation);
            }
        }

        result
    }
}

#[serde_with::minify_variant_names(serialize = "minified", deserialize = "minified")]
#[derive(derivative::Derivative, Clone, serde::Serialize, serde::Deserialize)]
#[derivative(Debug)]
pub enum MetaType {
    Scalar(ScalarType),
    Object(ObjectType),
    Interface(InterfaceType),
    Union(UnionType),
    Enum(EnumType),
    InputObject(InputObjectType),
}

impl MetaType {
    pub fn object(&self) -> Option<&ObjectType> {
        match self {
            MetaType::Object(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn is_object(&self) -> bool {
        matches!(self, MetaType::Object(_))
    }

    pub fn is_input_object(&self) -> bool {
        matches!(self, MetaType::InputObject(_))
    }

    pub fn is_node(&self) -> bool {
        match self {
            MetaType::Object(object) => object.is_node,
            _ => false,
        }
    }

    pub fn get_input_field(&self, name: &str) -> Option<&MetaInputValue> {
        if let MetaType::InputObject(ref object) = self {
            object.input_fields.get(name)
        } else {
            None
        }
    }
}

#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::skip_serializing_defaults(Option, Vec, bool, CacheControl, IndexMap)]
#[derive(derivative::Derivative, Clone, serde::Serialize, serde::Deserialize)]
#[derivative(Debug)]
pub struct ScalarType {
    pub name: String,
    pub description: Option<String>,
    #[derivative(Debug = "ignore")]
    #[serde(skip)]
    pub is_valid: Option<fn(value: &Value) -> bool>,
    #[derivative(Debug = "ignore")]
    #[serde(skip)]
    pub visible: Option<MetaVisibleFn>,
    pub specified_by_url: Option<String>,
    #[serde(default)]
    pub parser: ScalarParser,
}

impl From<ScalarType> for MetaType {
    fn from(val: ScalarType) -> Self {
        MetaType::Scalar(val)
    }
}

#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::skip_serializing_defaults(Option, Vec, bool, CacheControl, IndexMap)]
#[derive(derivative::Derivative, Clone, serde::Serialize, serde::Deserialize)]
#[derivative(Debug)]
pub struct ObjectType {
    pub name: String,
    pub description: Option<String>,
    pub fields: IndexMap<String, MetaField>,
    pub cache_control: CacheControl,
    pub extends: bool,
    #[derivative(Debug = "ignore")]
    #[serde(skip)]
    pub visible: Option<MetaVisibleFn>,
    pub is_subscription: bool,
    /// Define if the current Object if a Node
    pub is_node: bool,
    pub rust_typename: String,
    pub constraints: Vec<Constraint>,
    pub external: bool,
    pub shareable: bool,
}

impl ObjectType {
    pub fn new(name: impl Into<String>, fields: impl IntoIterator<Item = MetaField>) -> ObjectType {
        let name = name.into();
        ObjectType {
            rust_typename: name.clone(),
            name,
            fields: fields.into_iter().map(|field| (field.name.clone(), field)).collect(),
            description: None,
            cache_control: Default::default(),
            extends: false,
            visible: None,
            is_subscription: false,
            is_node: false,
            constraints: vec![],
            external: false,
            shareable: false,
        }
    }

    pub fn with_description(self, description: impl Into<Option<String>>) -> Self {
        ObjectType {
            description: description.into(),
            ..self
        }
    }

    pub fn with_cache_control(self, cache_control: CacheControl) -> Self {
        ObjectType { cache_control, ..self }
    }

    pub fn with_external(self, external: bool) -> Self {
        ObjectType { external, ..self }
    }

    pub fn with_shareable(self, shareable: bool) -> Self {
        ObjectType { shareable, ..self }
    }

    #[inline]
    pub fn field_by_name(&self, name: &str) -> Option<&MetaField> {
        self.fields.get(name)
    }
}

impl From<ObjectType> for MetaType {
    fn from(val: ObjectType) -> Self {
        MetaType::Object(val)
    }
}

impl<'a> TryFrom<&'a MetaType> for &'a ObjectType {
    type Error = Error;

    fn try_from(value: &'a MetaType) -> Result<Self, Self::Error> {
        match value {
            MetaType::Object(inner) => Ok(inner),
            _ => Err(Error::unexpected_kind(value.name(), value.kind(), TypeKind::Object)),
        }
    }
}

#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::skip_serializing_defaults(Option, Vec, bool, CacheControl, IndexMap)]
#[derive(derivative::Derivative, Clone, serde::Serialize, serde::Deserialize)]
#[derivative(Debug)]
pub struct InterfaceType {
    pub name: String,
    pub description: Option<String>,
    pub fields: IndexMap<String, MetaField>,
    pub possible_types: IndexSet<String>,
    pub extends: bool,
    #[derivative(Debug = "ignore")]
    #[serde(skip)]
    pub visible: Option<MetaVisibleFn>,
    pub rust_typename: String,
}

impl InterfaceType {
    pub fn new(name: impl Into<String>, fields: impl IntoIterator<Item = MetaField>) -> Self {
        InterfaceType {
            name: name.into(),
            description: None,
            fields: fields.into_iter().map(|field| (field.name.clone(), field)).collect(),
            possible_types: Default::default(),
            extends: false,
            visible: None,
            rust_typename: String::new(),
        }
    }

    pub fn field_by_name(&self, name: &str) -> Option<&MetaField> {
        self.fields.get(name)
    }
}

impl From<InterfaceType> for MetaType {
    fn from(val: InterfaceType) -> Self {
        MetaType::Interface(val)
    }
}

#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::skip_serializing_defaults(Option, Vec, bool, CacheControl, IndexMap)]
#[derive(derivative::Derivative, Clone, serde::Serialize, serde::Deserialize)]
#[derivative(Debug)]
pub struct UnionType {
    pub name: String,
    pub description: Option<String>,
    pub possible_types: IndexSet<String>,
    #[derivative(Debug = "ignore")]
    #[serde(skip)]
    pub visible: Option<MetaVisibleFn>,
    pub rust_typename: String,
    pub discriminators: Option<Vec<(String, UnionDiscriminator)>>,
}

impl UnionType {
    pub fn new<T: Into<String>>(name: impl Into<String>, possible_types: impl IntoIterator<Item = T>) -> UnionType {
        let name = name.into();
        UnionType {
            rust_typename: name.clone(),
            name,
            description: None,
            possible_types: possible_types.into_iter().map(Into::into).collect(),
            visible: None,
            discriminators: None,
        }
    }
}

impl From<UnionType> for MetaType {
    fn from(val: UnionType) -> Self {
        MetaType::Union(val)
    }
}

#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::skip_serializing_defaults(Option, Vec, bool, CacheControl, IndexMap)]
#[derive(derivative::Derivative, Clone, serde::Serialize, serde::Deserialize)]
#[derivative(Debug)]
pub struct EnumType {
    pub name: String,
    pub description: Option<String>,
    pub enum_values: IndexMap<String, MetaEnumValue>,
    #[derivative(Debug = "ignore")]
    #[serde(skip)]
    pub visible: Option<MetaVisibleFn>,
    pub rust_typename: String,
}

impl EnumType {
    pub fn new(name: String, values: impl IntoIterator<Item = MetaEnumValue>) -> Self {
        EnumType {
            rust_typename: name.clone(),
            name,
            enum_values: values.into_iter().map(|value| (value.name.clone(), value)).collect(),
            description: None,
            visible: None,
        }
    }

    pub fn with_description(self, description: Option<String>) -> Self {
        EnumType { description, ..self }
    }
}

impl From<EnumType> for MetaType {
    fn from(val: EnumType) -> Self {
        MetaType::Enum(val)
    }
}

#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::skip_serializing_defaults(Option, Vec, bool, CacheControl, IndexMap)]
#[derive(derivative::Derivative, Clone, serde::Serialize, serde::Deserialize)]
#[derivative(Debug)]
pub struct InputObjectType {
    pub name: String,
    pub description: Option<String>,
    pub input_fields: IndexMap<String, MetaInputValue>,
    #[derivative(Debug = "ignore")]
    #[serde(skip)]
    pub visible: Option<MetaVisibleFn>,
    pub rust_typename: String,
    pub oneof: bool,
}

impl InputObjectType {
    pub fn new(name: String, input_fields: impl IntoIterator<Item = MetaInputValue>) -> Self {
        InputObjectType {
            rust_typename: name.clone(),
            name,
            description: None,
            input_fields: input_fields.into_iter().map(|v| (v.name.clone(), v)).collect(),
            visible: None,
            oneof: false,
        }
    }

    pub fn with_description(self, description: Option<String>) -> Self {
        InputObjectType { description, ..self }
    }

    pub fn with_oneof(self, oneof: bool) -> Self {
        InputObjectType { oneof, ..self }
    }
}

impl From<InputObjectType> for MetaType {
    fn from(val: InputObjectType) -> Self {
        MetaType::InputObject(val)
    }
}

impl Error {
    fn unexpected_kind(name: &str, kind: TypeKind, expected: TypeKind) -> Self {
        Error::new(format!(
            "Type {name} appeared in a position where we expected a {expected:?} but it is a {kind:?}",
        ))
    }
}

/// The type of parser to be used for scalar values.
#[derive(Default, derivative::Derivative, Clone, serde::Serialize, serde::Deserialize)]
#[derivative(Debug)]
pub enum ScalarParser {
    /// Do not parse scalars, instead match the [`serde_json::Value`] type directly to the relevant
    /// [`Value`] type.
    PassThrough,

    /// Parse the scalar based on a list of well-known formats, trying to match the value to one of
    /// the formats. If no match is found, an error is returned.
    ///
    /// See [`PossibleScalar`] for more details.
    #[default]
    BestEffort,
}

impl Hash for MetaType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Scalar(ScalarType {
                name,
                description,
                specified_by_url,
                ..
            }) => {
                name.hash(state);
                description.hash(state);
                specified_by_url.hash(state);
            }
            Self::Object(ObjectType {
                name,
                description,
                fields,
                cache_control,
                extends,
                visible: _,
                is_subscription,
                is_node,
                rust_typename,
                constraints,
                external,
                shareable,
            }) => {
                name.hash(state);
                description.hash(state);
                fields.as_slice().hash(state);
                cache_control.hash(state);
                extends.hash(state);
                is_subscription.hash(state);
                is_node.hash(state);
                rust_typename.hash(state);
                constraints.hash(state);
                external.hash(state);
                shareable.hash(state);
            }
            Self::Interface(InterfaceType {
                name,
                description,
                fields,
                possible_types,
                extends,
                visible: _,
                rust_typename,
            }) => {
                name.hash(state);
                description.hash(state);
                fields.as_slice().hash(state);
                possible_types.as_slice().hash(state);
                extends.hash(state);
                rust_typename.hash(state);
            }
            Self::Enum(EnumType {
                name,
                description,
                enum_values,
                visible: _,
                rust_typename,
            }) => {
                name.hash(state);
                description.hash(state);
                enum_values.as_slice().hash(state);
                rust_typename.hash(state);
            }
            Self::Union(UnionType {
                name,
                description,
                possible_types,
                visible: _,
                rust_typename,
                discriminators,
            }) => {
                name.hash(state);
                description.hash(state);
                possible_types.as_slice().hash(state);
                rust_typename.hash(state);
                discriminators.hash(state);
            }
            Self::InputObject(InputObjectType {
                name,
                description,
                input_fields,
                visible: _,
                rust_typename,
                oneof,
            }) => {
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
                Self::Scalar(ScalarType {
                    name,
                    description,
                    specified_by_url,
                    ..
                }),
                Self::Scalar(ScalarType {
                    name: o_name,
                    description: o_descrition,
                    specified_by_url: o_specified_by_url,
                    ..
                }),
            ) => name.eq(o_name) && description.eq(o_descrition) && specified_by_url.eq(o_specified_by_url),
            (
                Self::Object(ObjectType {
                    name,
                    description,
                    fields,
                    cache_control,
                    extends,
                    visible: _,
                    is_subscription,
                    is_node,
                    rust_typename,
                    constraints,
                    external,
                    shareable,
                }),
                Self::Object(ObjectType {
                    name: o_name,
                    description: o_description,
                    fields: o_fields,
                    cache_control: o_cache_control,
                    extends: o_extends,
                    visible: _,
                    is_subscription: o_is_subscription,
                    is_node: o_is_node,
                    rust_typename: o_rust_typename,
                    constraints: o_constraints,
                    external: o_external,
                    shareable: o_shareable,
                }),
            ) => {
                name.eq(o_name)
                    && description.eq(o_description)
                    && fields.as_slice().eq(o_fields.as_slice())
                    && cache_control.eq(o_cache_control)
                    && extends.eq(o_extends)
                    && is_subscription.eq(o_is_subscription)
                    && is_node.eq(o_is_node)
                    && rust_typename.eq(o_rust_typename)
                    && constraints.eq(o_constraints)
                    && external.eq(o_external)
                    && shareable.eq(o_shareable)
            }
            (
                Self::Interface(InterfaceType {
                    name,
                    description,
                    fields,
                    possible_types,
                    extends,
                    visible: _,
                    rust_typename,
                }),
                Self::Interface(InterfaceType {
                    name: o_name,
                    description: o_description,
                    fields: o_fields,
                    possible_types: o_possible_types,
                    extends: o_extends,
                    visible: _,
                    rust_typename: o_rust_typename,
                }),
            ) => {
                name.eq(o_name)
                    && description.eq(o_description)
                    && fields.as_slice().eq(o_fields.as_slice())
                    && possible_types.as_slice().eq(o_possible_types.as_slice())
                    && extends.eq(o_extends)
                    && rust_typename.eq(o_rust_typename)
            }
            (
                Self::Enum(EnumType {
                    name,
                    description,
                    enum_values,
                    visible: _,
                    rust_typename,
                }),
                Self::Enum(EnumType {
                    name: o_name,
                    description: o_description,
                    enum_values: o_enum_values,
                    visible: _,
                    rust_typename: o_rust_typename,
                }),
            ) => {
                name.eq(o_name)
                    && description.eq(o_description)
                    && enum_values.as_slice().eq(o_enum_values.as_slice())
                    && rust_typename.eq(o_rust_typename)
            }
            (
                Self::Union(UnionType {
                    name,
                    description,
                    possible_types,
                    visible: _,
                    rust_typename,
                    discriminators,
                }),
                Self::Union(UnionType {
                    name: o_name,
                    description: o_description,
                    possible_types: o_possible_types,
                    visible: _,
                    rust_typename: o_rust_typename,
                    discriminators: o_discrimnators,
                }),
            ) => {
                name.eq(o_name)
                    && description.eq(o_description)
                    && possible_types.as_slice().eq(o_possible_types.as_slice())
                    && rust_typename.eq(o_rust_typename)
                    && discriminators.eq(o_discrimnators)
            }
            (
                Self::InputObject(InputObjectType {
                    name,
                    description,
                    input_fields,
                    visible: _,
                    rust_typename,
                    oneof,
                }),
                Self::InputObject(InputObjectType {
                    name: o_name,
                    description: o_description,
                    input_fields: o_input_fields,
                    visible: _,
                    rust_typename: o_rust_typename,
                    oneof: o_oneof,
                }),
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
    pub fn field_by_name_mut(&mut self, name: &str) -> Option<&mut MetaField> {
        self.fields_mut().and_then(|fields| fields.get_mut(name))
    }

    #[inline]
    pub fn fields(&self) -> Option<&IndexMap<String, MetaField>> {
        match self {
            MetaType::Object(inner) => Some(&inner.fields),
            MetaType::Interface(inner) => Some(&inner.fields),
            _ => None,
        }
    }

    #[inline]
    pub fn fields_mut(&mut self) -> Option<&mut IndexMap<String, MetaField>> {
        match self {
            MetaType::Object(inner) => Some(&mut inner.fields),
            MetaType::Interface(inner) => Some(&mut inner.fields),
            _ => None,
        }
    }

    pub fn constraints(&self) -> &[Constraint] {
        match self {
            MetaType::Object(inner) => &inner.constraints,
            _ => &[],
        }
    }

    #[inline]
    pub fn is_visible(&self, ctx: &ContextField<'_>) -> bool {
        let visible = match self {
            MetaType::Scalar(inner) => &inner.visible,
            MetaType::Object(inner) => &inner.visible,
            MetaType::Interface(inner) => &inner.visible,
            MetaType::Union(inner) => &inner.visible,
            MetaType::Enum(inner) => &inner.visible,
            MetaType::InputObject(inner) => &inner.visible,
        };
        is_visible(ctx, visible)
    }

    #[inline]
    pub fn name(&self) -> &str {
        match self {
            MetaType::Scalar(inner) => &inner.name,
            MetaType::Object(inner) => &inner.name,
            MetaType::Interface(inner) => &inner.name,
            MetaType::Union(inner) => &inner.name,
            MetaType::Enum(inner) => &inner.name,
            MetaType::InputObject(inner) => &inner.name,
        }
    }

    #[inline]
    pub fn description(&self) -> Option<&str> {
        match self {
            MetaType::Scalar(inner) => inner.description.as_deref(),
            MetaType::Object(inner) => inner.description.as_deref(),
            MetaType::Interface(inner) => inner.description.as_deref(),
            MetaType::Union(inner) => inner.description.as_deref(),
            MetaType::Enum(inner) => inner.description.as_deref(),
            MetaType::InputObject(inner) => inner.description.as_deref(),
        }
    }

    #[inline]
    pub fn is_composite(&self) -> bool {
        matches!(self, MetaType::Object(_) | MetaType::Interface(_) | MetaType::Union(_))
    }

    #[inline]
    pub fn is_abstract(&self) -> bool {
        matches!(self, MetaType::Interface(_) | MetaType::Union(_))
    }

    #[inline]
    pub fn is_leaf(&self) -> bool {
        matches!(self, MetaType::Enum(_) | MetaType::Scalar(_))
    }

    #[inline]
    pub fn is_input(&self) -> bool {
        matches!(self, MetaType::Enum(_) | MetaType::Scalar(_) | MetaType::InputObject(_))
    }

    #[inline]
    pub fn is_enum(&self) -> bool {
        matches!(self, MetaType::Enum(_))
    }

    #[inline]
    pub fn is_possible_type(&self, type_name: &str) -> bool {
        match self {
            MetaType::Interface(inner) => inner.possible_types.contains(type_name),
            MetaType::Union(inner) => inner.possible_types.contains(type_name),
            MetaType::Object(inner) => inner.name == type_name,
            _ => false,
        }
    }

    #[inline]
    pub fn possible_types(&self) -> Option<&IndexSet<String>> {
        match self {
            MetaType::Interface(inner) => Some(&inner.possible_types),
            MetaType::Union(inner) => Some(&inner.possible_types),
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
            MetaType::Object(inner) => Some(&inner.rust_typename),
            MetaType::Interface(inner) => Some(&inner.rust_typename),
            MetaType::Union(inner) => Some(&inner.rust_typename),
            MetaType::Enum(inner) => Some(&inner.rust_typename),
            MetaType::InputObject(inner) => Some(&inner.rust_typename),
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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MongoDBConfiguration {
    pub name: String,
    pub api_key: String,
    pub url: String,
    pub data_source: String,
    pub database: String,
    pub namespace: bool,
}

#[derive(
    Debug,
    Clone,
    Copy,
    derivative::Derivative,
    serde::Serialize,
    serde::Deserialize,
    Hash,
    Eq,
    Ord,
    PartialOrd,
    PartialEq,
)]
#[repr(transparent)]
pub struct SchemaID(u16);

#[derive(Default)]
pub struct SchemaIDGenerator {
    cur: AtomicU16,
}

impl SchemaIDGenerator {
    /// Generate a new ID for a schema ID.
    pub fn new_id(&self) -> SchemaID {
        let val = self.cur.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        SchemaID(val)
    }
}

#[derive(Default)]
pub struct ConnectorIdGenerator {
    cur: AtomicU16,
}

impl ConnectorIdGenerator {
    /// Generate a new connector ID.
    pub fn new_id(&self) -> u16 {
        self.cur.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}

#[derive(Deserialize, Serialize)]
pub struct VersionedRegistry<'a> {
    pub registry: Cow<'a, Registry>,
    pub deployment_id: Cow<'a, str>,
}

// TODO(@miaxos): Remove this to a separate create as we'll need to use it outside engine
// for a LogicalQuery
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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
    #[serde(default)]
    pub mongodb_configurations: HashMap<String, MongoDBConfiguration>,
    #[serde(default)]
    pub http_headers: BTreeMap<String, ConnectorHeaders>,
    #[serde(default)]
    pub postgres_databases: HashMap<String, DatabaseDefinition>,
    #[serde(default)]
    pub search_config: runtime::search::Config,
    #[serde(default)]
    pub enable_caching: bool,
    #[serde(default)]
    pub enable_kv: bool,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub federation_entities: BTreeMap<String, FederationEntity>,
    #[serde(default)]
    pub enable_ai: bool,
    #[serde(default)]
    pub grafbase_cli_version: Option<String>,
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            types: Default::default(),
            directives: Default::default(),
            implements: Default::default(),
            query_type: Query.to_string().to_pascal_case(),
            mutation_type: None,
            subscription_type: None,
            disable_introspection: false,
            enable_federation: false,
            federation_subscription: false,
            auth: Default::default(),
            mongodb_configurations: Default::default(),
            http_headers: Default::default(),
            postgres_databases: Default::default(),
            search_config: Default::default(),
            enable_caching: false,
            enable_kv: false,
            federation_entities: Default::default(),
            enable_ai: false,
            grafbase_cli_version: None,
        }
    }
}

impl Registry {
    /// Looks up a particular type in the registry, using the default kind for the given TypeName.
    ///
    /// Will error if the type doesn't exist or is of an unexpected kind.
    pub fn lookup<'a, Name>(&'a self, name: &Name) -> Result<Name::ExpectedType<'a>, Error>
    where
        Name: TypeReference,
        Name::ExpectedType<'a>: TryFrom<&'a MetaType>,
        <Name::ExpectedType<'a> as TryFrom<&'a MetaType>>::Error: Into<Error>,
    {
        self.lookup_by_str(name.named_type().as_str())?
            .try_into()
            .map_err(Into::into)
    }

    /// Looks up a particular type in the registry, with the expectation that it is of a particular kind.
    ///
    /// Will error if the type doesn't exist or is of an unexpected kind.
    pub fn lookup_expecting<'a, Expected>(&'a self, name: &impl TypeReference) -> Result<Expected, Error>
    where
        Expected: TryFrom<&'a MetaType> + 'a,
        <Expected as TryFrom<&'a MetaType>>::Error: Into<Error>,
    {
        self.lookup_by_str(name.named_type().as_str())?
            .try_into()
            .map_err(Into::into)
    }

    fn lookup_by_str<'a>(&'a self, name: &str) -> Result<&'a MetaType, Error> {
        self.types
            .get(name)
            .ok_or_else(|| Error::new(format!("Couldn't find a type named {name}")))
    }

    pub fn root_type(&self, operation_type: OperationType) -> SelectionSetTarget<'_> {
        match operation_type {
            OperationType::Query => self.query_root(),
            OperationType::Mutation => self.mutation_root(),
            OperationType::Subscription => {
                // We don't do subscriptions but may as well implement anyway.
                self.concrete_type_by_name(
                    self.subscription_type
                        .as_deref()
                        .expect("we shouldnt get here if theres no subscription type"),
                )
                .expect("the registry to be valid")
            }
        }
        .try_into()
        .expect("root type should always be a composite type")
    }
}

pub mod vectorize {
    use std::iter::FromIterator;

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<'a, T, K, V, S>(target: T, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: IntoIterator<Item = (&'a K, &'a V)>,
        K: Serialize + 'a,
        V: Serialize + 'a,
    {
        ser.collect_seq(target)
    }

    pub fn deserialize<'de, T, K, V, D>(des: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: FromIterator<(K, V)>,
        K: Deserialize<'de>,
        V: Deserialize<'de>,
    {
        let container: Vec<_> = serde::Deserialize::deserialize(des)?;
        Ok(container.into_iter().collect::<T>())
    }
}

impl Registry {
    pub fn new() -> Registry {
        let type_query = Query.to_string().to_pascal_case();
        let mut registry = Registry {
            query_type: type_query.clone(),
            ..Registry::default()
        };
        registry
            .types
            .insert(type_query.clone(), ObjectType::new(type_query, []).into());
        registry
    }

    /// Fill the `Registry` with sample data.
    ///
    /// This can be useful for testing purposes.
    pub fn with_sample_data(mut self) -> Self {
        let fields = self.query_root_mut().fields_mut().unwrap();

        fields.insert(
            "scalar".to_owned(),
            MetaField {
                name: "scalar".to_owned(),
                description: Some("test scalar".to_owned()),
                ty: "MyScalar".into(),
                ..Default::default()
            },
        );

        self.types.insert(
            "MyScalar".to_owned(),
            MetaType::Scalar(ScalarType {
                name: "MyScalar".to_owned(),
                description: Some("test scalar".to_owned()),
                is_valid: None,
                visible: None,
                specified_by_url: None,
                parser: ScalarParser::default(),
            }),
        );

        self
    }

    pub fn query_root(&self) -> &MetaType {
        self.types.get(&self.query_type).unwrap()
    }

    pub fn query_root_mut(&mut self) -> &mut MetaType {
        self.types.get_mut(&self.query_type).unwrap()
    }

    pub fn mutation_root(&self) -> &MetaType {
        // TODO: Fix this.
        self.types.get(self.mutation_type.as_deref().unwrap()).unwrap()
    }

    pub fn mutation_root_mut(&mut self) -> &mut MetaType {
        self.types.get_mut(self.mutation_type.as_deref().unwrap()).unwrap()
    }

    pub fn find_ty_with_id(&self, node_id: &NodeID<'_>) -> Option<&MetaType> {
        let ty = node_id.ty();
        self.types
            .iter()
            .find(|(key, _value)| key.to_lowercase() == ty)
            .map(|(_, val)| val)
    }
}

impl Registry {
    pub fn create_input_type<T: LegacyInputType + ?Sized, F: FnOnce(&mut Registry) -> MetaType>(
        &mut self,
        f: F,
    ) -> InputValueType {
        self.create_type(f, &T::type_name(), std::any::type_name::<T>());
        T::qualified_type_name()
    }

    pub fn create_output_type<T: LegacyOutputType + ?Sized, F: FnOnce(&mut Registry) -> MetaType>(
        &mut self,
        f: F,
    ) -> MetaFieldType {
        self.create_type(f, &T::type_name(), std::any::type_name::<T>());
        T::qualified_type_name()
    }

    pub fn create_subscription_type<T: SubscriptionType + ?Sized, F: FnOnce(&mut Registry) -> MetaType>(
        &mut self,
        f: F,
    ) -> String {
        self.create_type(f, &T::type_name(), std::any::type_name::<T>());
        T::qualified_type_name().to_string()
    }

    pub fn insert_type(&mut self, ty: impl Into<MetaType>) {
        let ty = ty.into();
        self.types.insert(ty.name().to_string(), ty);
    }

    pub fn create_mongo_config<F>(&mut self, f: F, name: &str)
    where
        F: FnOnce(&mut Registry) -> MongoDBConfiguration,
    {
        if self.mongodb_configurations.get(name).is_some() {
            panic!("MongoDB directive with `{name}` already exists.");
        }

        let config = f(self);
        self.mongodb_configurations.insert(name.to_string(), config);
    }

    pub fn create_type<F: FnOnce(&mut Registry) -> MetaType>(&mut self, f: F, name: &str, rust_typename: &str) {
        match self.types.get(name) {
            Some(ty) => {
                if let Some(prev_typename) = ty.rust_typename() {
                    if prev_typename.ne("__fake_type__") && prev_typename.ne(rust_typename) {
                        panic!("`{prev_typename}` and `{rust_typename}` have the same GraphQL name `{name}`",);
                    }
                }
            }
            None => {
                // Inserting a fake type before calling the function allows recursive types to exist.
                self.types.insert(
                    name.to_string(),
                    ObjectType {
                        rust_typename: "__fake_type__".to_string(),
                        ..ObjectType::new(String::new(), [])
                    }
                    .into(),
                );
                let ty = f(self);
                *self.types.get_mut(name).unwrap() = ty;
            }
        }
    }

    pub fn create_fake_output_type<T: LegacyOutputType>(&mut self) -> MetaType {
        T::create_type_info(self);
        self.types
            .get(&*T::type_name())
            .cloned()
            .expect("You definitely encountered a bug!")
    }

    pub fn create_fake_input_type<T: LegacyInputType>(&mut self) -> MetaType {
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
        self.directives.insert(directive.name.to_string(), directive);
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
        !self.federation_entities.is_empty()
    }

    /// Each type annotated with @key should be added to the _Entity union.
    /// If no types are annotated with the key directive, then the _Entity union
    /// and Query._entities field should be removed from the schema.
    ///
    /// [Reference](https://www.apollographql.com/docs/federation/federation-spec/#resolve-requests-for-entities).
    fn create_entity_type_and_root_field(&mut self) {
        let possible_types: IndexSet<_> = self.federation_entities.keys().cloned().collect();

        if !possible_types.is_empty() {
            self.types.insert(
                "_Entity".to_string(),
                UnionType {
                    name: "_Entity".to_string(),
                    description: None,
                    possible_types,
                    visible: None,
                    rust_typename: "engine::federation::Entity".to_string(),
                    discriminators: None,
                }
                .into(),
            );

            let query_root = self.types.get_mut(&self.query_type).unwrap();
            if let MetaType::Object(object) = query_root {
                object.fields.insert(
                    "_service".to_string(),
                    MetaField {
                        name: "_service".to_string(),
                        ty: "_Service!".into(),
                        resolver: Resolver::Introspection(resolvers::IntrospectionResolver::FederationServiceField),
                        ..Default::default()
                    },
                );

                object.fields.insert(
                    "_entities".to_string(),
                    MetaField {
                        name: "_entities".to_string(),
                        args: {
                            let mut args = IndexMap::new();
                            args.insert(
                                "representations".to_string(),
                                MetaInputValue::new("representations", "[_Any!]!"),
                            );
                            args
                        },
                        ty: "[_Entity]!".into(),
                        resolver: Resolver::FederationEntitiesResolver,
                        ..Default::default()
                    },
                );
            }
        }
    }

    pub(crate) fn create_federation_types(&mut self) {
        <Any as LegacyInputType>::create_type_info(self);

        self.types.insert(
            "_Service".to_string(),
            ObjectType {
                rust_typename: "engine::federation::Service".to_string(),
                ..ObjectType::new(
                    "_Service",
                    [MetaField {
                        name: "sdl".to_string(),
                        ty: "String".into(),
                        ..Default::default()
                    }],
                )
            }
            .into(),
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
                MetaType::Scalar(_) | MetaType::Union(_) => {
                    names.insert(ty.name().to_string());
                }
                MetaType::Object(ObjectType { name, fields, .. })
                | MetaType::Interface(InterfaceType { name, fields, .. }) => {
                    names.insert(name.clone());
                    names.extend(fields.values().flat_map(|field| {
                        std::iter::once(field.name.clone()).chain(field.args.values().map(|arg| arg.name.to_string()))
                    }));
                }
                MetaType::Enum(EnumType { name, enum_values, .. }) => {
                    names.insert(name.clone());
                    names.extend(enum_values.values().map(|value| value.name.to_string()));
                }
                MetaType::InputObject(InputObjectType { name, input_fields, .. }) => {
                    names.insert(name.clone());
                    names.extend(input_fields.values().map(|field| field.name.to_string()));
                }
            }
        }

        names.into_iter().collect()
    }

    pub fn set_description(&mut self, name: &str, desc: &'static str) {
        match self.types.get_mut(name) {
            Some(MetaType::Scalar(inner)) => inner.description = Some(desc.to_string()),
            Some(MetaType::Object(inner)) => inner.description = Some(desc.to_string()),
            Some(MetaType::Interface(inner)) => inner.description = Some(desc.to_string()),
            Some(MetaType::Union(inner)) => inner.description = Some(desc.to_string()),
            Some(MetaType::Enum(inner)) => inner.description = Some(desc.to_string()),
            Some(MetaType::InputObject(inner)) => {
                inner.description = Some(desc.to_string());
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
            traverse_type(types, used_types, field.ty.named_type().as_str());
            for arg in field.args.values() {
                traverse_input_value(types, used_types, arg);
            }
        }

        fn traverse_input_value<'a>(
            types: &'a BTreeMap<String, MetaType>,
            used_types: &mut BTreeSet<&'a str>,
            input_value: &'a MetaInputValue,
        ) {
            traverse_type(types, used_types, input_value.ty.named_type().as_str());
        }

        fn traverse_type<'a>(
            types: &'a BTreeMap<String, MetaType>,
            used_types: &mut BTreeSet<&'a str>,
            type_name: &str,
        ) {
            if used_types.contains(type_name) {
                return;
            }

            if let Some(ty) = types.get(type_name) {
                used_types.insert(ty.name());
                match ty {
                    MetaType::Object(object) => {
                        for field in object.fields.values() {
                            traverse_field(types, used_types, field);
                        }
                    }
                    MetaType::Interface(interface) => {
                        for field in interface.fields.values() {
                            traverse_field(types, used_types, field);
                        }
                        for type_name in &interface.possible_types {
                            traverse_type(types, used_types, type_name);
                        }
                    }
                    MetaType::Union(union_type) => {
                        for type_name in &union_type.possible_types {
                            traverse_type(types, used_types, type_name);
                        }
                    }
                    MetaType::InputObject(input_object) => {
                        for field in input_object.input_fields.values() {
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

        for ty in self.federation_entities.keys() {
            traverse_type(&self.types, &mut used_types, ty);
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

    pub fn find_visible_types(&self, ctx: &ContextField<'_>) -> HashSet<&str> {
        let mut visible_types = HashSet::new();

        fn traverse_field<'a>(
            ctx: &ContextField<'_>,
            types: &'a BTreeMap<String, MetaType>,
            visible_types: &mut HashSet<&'a str>,
            field: &'a MetaField,
        ) {
            if !is_visible(ctx, &field.visible) {
                return;
            }

            traverse_type(ctx, types, visible_types, field.ty.named_type().as_str());
            for arg in field.args.values() {
                traverse_input_value(ctx, types, visible_types, arg);
            }
        }

        fn traverse_input_value<'a>(
            ctx: &ContextField<'_>,
            types: &'a BTreeMap<String, MetaType>,
            visible_types: &mut HashSet<&'a str>,
            input_value: &'a MetaInputValue,
        ) {
            if !is_visible(ctx, &input_value.visible) {
                return;
            }

            traverse_type(ctx, types, visible_types, input_value.ty.named_type().as_str());
        }

        fn traverse_type<'a>(
            ctx: &ContextField<'_>,
            types: &'a BTreeMap<String, MetaType>,
            visible_types: &mut HashSet<&'a str>,
            type_name: &str,
        ) {
            if visible_types.contains(type_name) {
                return;
            }

            if let Some(ty) = types.get(type_name) {
                if !ty.is_visible(ctx) {
                    return;
                }

                visible_types.insert(ty.name());
                match ty {
                    MetaType::Object(object) => {
                        for field in object.fields.values() {
                            traverse_field(ctx, types, visible_types, field);
                        }
                    }
                    MetaType::Interface(interface) => {
                        for field in interface.fields.values() {
                            traverse_field(ctx, types, visible_types, field);
                        }
                        for type_name in &interface.possible_types {
                            traverse_type(ctx, types, visible_types, type_name);
                        }
                    }
                    MetaType::Union(union_type) => {
                        for type_name in &union_type.possible_types {
                            traverse_type(ctx, types, visible_types, type_name);
                        }
                    }
                    MetaType::InputObject(input_object) => {
                        for field in input_object.input_fields.values() {
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

        for ty in self.federation_entities.keys() {
            traverse_type(ctx, &self.types, &mut visible_types, ty);
        }

        for ty in self.types.values() {
            if let MetaType::Interface(interface) = ty {
                if ty.is_visible(ctx) && !visible_types.contains(ty.name()) {
                    for type_name in &interface.possible_types {
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

pub(crate) fn is_visible(ctx: &ContextField<'_>, visible: &Option<MetaVisibleFn>) -> bool {
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
