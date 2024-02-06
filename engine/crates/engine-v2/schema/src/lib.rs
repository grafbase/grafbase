use std::str::FromStr;

mod builder;
mod cache;
mod field_set;
mod ids;
mod names;
mod resolver;
pub mod sources;
mod walkers;
mod wrapping;

pub use cache::*;
pub use field_set::*;
pub use ids::*;
pub use names::Names;
pub use resolver::*;
pub use walkers::*;
pub use wrapping::*;

/// This does NOT need to be backwards compatible. We'll probably cache it for performance, but it is not
/// the source of truth. If the cache is stale we would just re-create this Graph from its source:
/// federated_graph::FederatedGraph.
pub struct Schema {
    pub data_sources: DataSources,

    pub description: Option<StringId>,
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
    /// All the field types in the supergraph, deduplicated.
    types: Vec<Type>,
    // All definitions sorted by their name (actual string)
    definitions: Vec<Definition>,

    /// All strings deduplicated.
    strings: Vec<String>,
    urls: Vec<url::Url>,

    /// Headers we might want to send to a subgraph
    headers: Vec<Header>,
    default_headers: Vec<HeaderId>,
    cache_configs: Vec<CacheConfig>,

    pub auth_config: Option<config::latest::AuthConfig>,
    pub operation_limits: config::latest::OperationLimits,
}

#[derive(Default)]
pub struct DataSources {
    federation: sources::federation::DataSource,
    pub introspection: sources::introspection::DataSource,
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

    // Used as the default resolver
    pub fn introspection_resolver_id(&self) -> ResolverId {
        (self.resolvers.len() - 1).into()
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

#[derive(Debug)]
pub struct Object {
    pub name: StringId,
    pub description: Option<StringId>,
    pub interfaces: Vec<InterfaceId>,
    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
    pub cache_config: Option<CacheConfigId>,
}

#[derive(PartialOrd, Ord, PartialEq, Eq)]
pub struct ObjectField {
    pub object_id: ObjectId,
    pub field_id: FieldId,
}

#[derive(Debug)]
pub struct Field {
    pub name: StringId,
    pub description: Option<StringId>,
    pub type_id: TypeId,
    pub resolvers: Vec<FieldResolver>,
    pub is_deprecated: bool,
    pub deprecation_reason: Option<StringId>,
    provides: Vec<FieldProvides>,
    pub arguments: Vec<InputValueId>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,

    pub cache_config: Option<CacheConfigId>,
}

#[derive(Debug)]
pub enum FieldProvides {
    // provided only if the current resolver is part of the group.
    IfResolverGroup { group: ResolverGroup, field_set: FieldSet },
}

#[derive(Debug)]
pub struct FieldResolver {
    resolver_id: ResolverId,
    field_requires: FieldSet,
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
    pub fn walk<I>(&self, item: I) -> SchemaWalker<'_, I> {
        SchemaWalker::new(item, self, &())
    }

    pub fn walker(&self) -> SchemaWalker<'_, ()> {
        self.walker_with(&())
    }

    pub fn walker_with<'a>(&'a self, names: &'a dyn Names) -> SchemaWalker<'a, ()> {
        SchemaWalker::new((), self, names)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Header {
    pub name: StringId,
    pub value: HeaderValue,
}

#[derive(Debug, Clone, Copy)]
pub enum HeaderValue {
    Forward(StringId),
    Static(StringId),
}
