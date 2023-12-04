#![allow(dead_code)]

use std::{borrow::Cow, str::FromStr};

mod conversion;
mod field_set;
mod ids;
pub mod introspection;
mod names;
mod walkers;

pub use field_set::*;
pub use ids::*;
use introspection::{IntrospectionDataSource, IntrospectionResolver};
pub use names::Names;
pub use walkers::*;

/// This does NOT need to be backwards compatible. We'll probably cache it for performance, but it is not
/// the source of truth. If the cache is stale we would just re-create this Graph from its source:
/// federated_graph::FederatedGraph.
pub struct Schema {
    pub description: Option<StringId>,
    data_sources: Vec<DataSource>,
    subgraphs: Vec<Subgraph>,

    pub root_operation_types: RootOperationTypes,
    objects: Vec<Object>,
    // Sorted by object_id, field name (actual string)
    object_fields: Vec<ObjectField>,

    interfaces: Vec<Interface>,
    // Sorted by interface_id, field name (actual string)
    interface_fields: Vec<InterfaceField>,

    fields: Vec<Field>,

    enums: Vec<Enum>,
    unions: Vec<Union>,
    scalars: Vec<Scalar>,
    input_objects: Vec<InputObject>,
    input_values: Vec<InputValue>,
    resolvers: Vec<Resolver>,

    /// All the strings in the supergraph, deduplicated.
    strings: Vec<String>,
    /// All the field types in the supergraph, deduplicated.
    types: Vec<Type>,
    // All definitions sorted by their name (actual string)
    definitions: Vec<Definition>,
}

impl Schema {
    pub fn definition_by_name(&self, name: &str) -> Option<Definition> {
        self.definitions
            .binary_search_by_key(&name, |definition| self.definition_name(*definition))
            .map(|index| self.definitions[index])
            .ok()
    }

    pub fn object_field_by_name(&self, object_id: ObjectId, name: &str) -> Option<FieldId> {
        self.object_fields
            .binary_search_by_key(&(object_id, name), |ObjectField { object_id, field_id }| {
                (*object_id, &self[self[*field_id].name])
            })
            .map(|index| self.object_fields[index].field_id)
            .ok()
    }

    pub fn interface_field_by_name(&self, interface_id: InterfaceId, name: &str) -> Option<FieldId> {
        self.interface_fields
            .binary_search_by_key(&(interface_id, name), |InterfaceField { interface_id, field_id }| {
                (*interface_id, &self[self[*field_id].name])
            })
            .map(|index| self.interface_fields[index].field_id)
            .ok()
    }

    fn ensure_proper_ordering(&mut self) {
        let mut object_fields = std::mem::take(&mut self.object_fields);
        object_fields
            .sort_unstable_by_key(|ObjectField { object_id, field_id }| (*object_id, &self[self[*field_id].name]));
        self.object_fields = object_fields;

        let mut interface_fields = std::mem::take(&mut self.interface_fields);
        interface_fields.sort_unstable_by_key(|InterfaceField { interface_id, field_id }| {
            (*interface_id, &self[self[*field_id].name])
        });
        self.interface_fields = interface_fields;

        let mut definitions = std::mem::take(&mut self.definitions);
        definitions.sort_unstable_by_key(|definition| self.definition_name(*definition));
        self.definitions = definitions;

        for interface in &mut self.interfaces {
            interface.possible_types.sort_unstable();
        }
        for union in &mut self.unions {
            union.possible_types.sort_unstable();
        }
    }

    fn definition_name(&self, definition: Definition) -> &str {
        let name = match definition {
            Definition::Scalar(s) => self[s].name,
            Definition::Object(o) => self[o].name,
            Definition::Interface(i) => self[i].name,
            Definition::Union(u) => self[u].name,
            Definition::Enum(e) => self[e].name,
            Definition::InputObject(io) => self[io].name,
        };
        &self[name]
    }
}

#[derive(Debug)]
pub struct RootOperationTypes {
    pub query: ObjectId,
    pub mutation: Option<ObjectId>,
    pub subscription: Option<ObjectId>,
}

impl std::fmt::Debug for Schema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Schema").finish_non_exhaustive()
    }
}

pub enum DataSource {
    Introspection(Box<IntrospectionDataSource>),
    Subgraph(SubgraphId),
}

impl DataSource {
    pub fn as_introspection(&self) -> Option<&IntrospectionDataSource> {
        match self {
            DataSource::Introspection(introspection) => Some(introspection),
            DataSource::Subgraph(_) => None,
        }
    }
}

#[derive(Debug)]
pub struct Subgraph {
    pub name: StringId,
    pub url: StringId,
}

#[derive(Debug)]
pub struct Object {
    pub name: StringId,
    pub description: Option<StringId>,
    pub interfaces: Vec<InterfaceId>,
    /// All _resolvable_ keys.
    resolvable_keys: Vec<Key>,
    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
}

#[derive(PartialOrd, Ord, PartialEq, Eq)]
pub struct ObjectField {
    pub object_id: ObjectId,
    pub field_id: FieldId,
}

#[derive(Debug)]
pub struct Key {
    /// The subgraph that can resolve the entity with these fields.
    pub subgraph_id: SubgraphId,

    /// Corresponds to the fields in an `@key` directive.
    pub fields: FieldSet,
}

#[derive(Debug)]
pub struct Field {
    pub name: StringId,
    pub description: Option<StringId>,
    pub type_id: TypeId,
    pub resolvers: Vec<FieldResolver>,
    pub is_deprecated: bool,
    pub deprecation_reason: Option<StringId>,

    /// Special case when only going through this field children are accessible.
    provides: Vec<FieldProvides>,

    pub arguments: Vec<InputValueId>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
}

#[derive(Debug)]
pub struct FieldResolver {
    pub resolver_id: ResolverId,
    pub requires: FieldSet,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Resolver {
    Introspection(IntrospectionResolver),
    Subgraph(SubgraphResolver),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubgraphResolver {
    pub subgraph_id: SubgraphId,
}

impl Resolver {
    pub fn data_source_id(&self) -> DataSourceId {
        match self {
            Resolver::Subgraph(resolver) => DataSourceId::from(resolver.subgraph_id),
            Resolver::Introspection(resolver) => resolver.data_source_id,
        }
    }

    pub fn supports_aliases(&self) -> bool {
        match self {
            Resolver::Subgraph(_) | Resolver::Introspection(_) => true,
        }
    }

    pub fn requires(&self) -> Cow<'_, FieldSet> {
        Cow::Owned(FieldSet::default())
    }
}

#[derive(Debug)]
pub struct Directive {
    pub name: StringId,
    pub arguments: Vec<(StringId, Value)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    String(StringId),
    Int(i64),
    Float(StringId),
    Boolean(bool),
    EnumValue(StringId),
    Object(Vec<(StringId, Value)>),
    List(Vec<Value>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Definition {
    Scalar(ScalarId),
    Object(ObjectId),
    Interface(InterfaceId),
    Union(UnionId),
    Enum(EnumId),
    InputObject(InputObjectId),
}

impl From<ScalarId> for Definition {
    fn from(id: ScalarId) -> Self {
        Self::Scalar(id)
    }
}

impl From<ObjectId> for Definition {
    fn from(id: ObjectId) -> Self {
        Self::Object(id)
    }
}

impl From<InterfaceId> for Definition {
    fn from(id: InterfaceId) -> Self {
        Self::Interface(id)
    }
}

impl From<UnionId> for Definition {
    fn from(id: UnionId) -> Self {
        Self::Union(id)
    }
}

impl From<EnumId> for Definition {
    fn from(id: EnumId) -> Self {
        Self::Enum(id)
    }
}

impl From<InputObjectId> for Definition {
    fn from(id: InputObjectId) -> Self {
        Self::InputObject(id)
    }
}

#[derive(Debug)]
pub struct Type {
    pub inner: Definition,
    pub wrapping: Wrapping,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Wrapping {
    /// Is the innermost type required?
    ///
    /// Examples:
    ///
    /// - `String` => false
    /// - `String!` => true
    /// - `[String!]` => true
    /// - `[String]!` => false
    pub inner_is_required: bool,

    /// Innermost to outermost.
    pub list_wrapping: Vec<ListWrapping>,
}

impl Wrapping {
    pub fn is_required(&self) -> bool {
        self.list_wrapping
            .last()
            .map(|lw| matches!(lw, ListWrapping::RequiredList))
            .unwrap_or(self.inner_is_required)
    }

    pub fn nullable() -> Self {
        Wrapping {
            inner_is_required: false,
            list_wrapping: vec![],
        }
    }

    pub fn required() -> Self {
        Wrapping {
            inner_is_required: true,
            list_wrapping: vec![],
        }
    }

    #[must_use]
    pub fn nullable_list(self) -> Self {
        Wrapping {
            list_wrapping: [ListWrapping::NullableList]
                .into_iter()
                .chain(self.list_wrapping)
                .collect(),
            ..self
        }
    }

    #[must_use]
    pub fn required_list(self) -> Self {
        Wrapping {
            list_wrapping: [ListWrapping::RequiredList]
                .into_iter()
                .chain(self.list_wrapping)
                .collect(),
            ..self
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListWrapping {
    RequiredList,
    NullableList,
}

/// Represents an `@provides` directive on a field in a subgraph.
#[derive(Debug)]
pub struct FieldProvides {
    pub data_source_id: DataSourceId,
    pub fields: FieldSet,
}

#[derive(Debug)]
pub struct Interface {
    pub name: StringId,
    pub description: Option<StringId>,
    pub interfaces: Vec<InterfaceId>,

    /// sorted by ObjectId
    pub possible_types: Vec<ObjectId>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
}

#[derive(Debug)]
pub struct InterfaceField {
    pub interface_id: InterfaceId,
    pub field_id: FieldId,
}

#[derive(Debug)]
pub struct Enum {
    pub name: StringId,
    pub description: Option<StringId>,
    pub values: Vec<EnumValue>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
}

#[derive(Debug)]
pub struct EnumValue {
    pub name: StringId,
    pub description: Option<StringId>,
    pub is_deprecated: bool,
    pub deprecation_reason: Option<StringId>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
}

#[derive(Debug)]
pub struct Union {
    pub name: StringId,
    pub description: Option<StringId>,
    /// sorted by ObjectId
    pub possible_types: Vec<ObjectId>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
}

#[derive(Debug)]
pub struct Scalar {
    pub name: StringId,
    pub data_type: DataType,
    pub description: Option<StringId>,
    pub specified_by_url: Option<StringId>,
    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
}

/// Defines how a scalar should be represented and validated by the engine. They're almost the same
/// as scalars, but scalars like ID which have no own data format are just mapped to String.
/// https://the-guild.dev/graphql/scalars/docs
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString)]
pub enum DataType {
    String,
    Float,
    Int,
    BigInt,
    JSON,
    Boolean,
}

impl DataType {
    pub fn from_scalar_name(name: &str) -> DataType {
        DataType::from_str(name).ok().unwrap_or(match name {
            "ID" => DataType::String,
            _ => DataType::JSON,
        })
    }
}

#[derive(Debug)]
pub struct InputObject {
    pub name: StringId,
    pub description: Option<StringId>,
    pub input_fields: Vec<InputValueId>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
}

#[derive(Debug, Clone)]
pub struct InputValue {
    pub name: StringId,
    pub description: Option<StringId>,
    pub type_id: TypeId,
    pub default_value: Option<Value>,
}

impl Schema {
    pub fn default_walker(&self) -> SchemaWalker<'_, ()> {
        self.walker(self)
    }

    pub fn walker<'a>(&'a self, names: &'a dyn Names) -> SchemaWalker<'a, ()> {
        SchemaWalker::new((), self, names)
    }
}
