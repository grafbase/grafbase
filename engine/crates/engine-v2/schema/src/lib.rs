use std::str::FromStr;

mod builder;
mod cache;
mod field_set;
mod ids;
mod input_value;
mod names;
mod resolver;
pub mod sources;
mod walkers;
mod wrapping;

pub use cache::*;
pub use field_set::*;
use id_newtypes::IdRange;
pub use ids::*;
pub use input_value::*;
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
    input_value_definitions: Vec<InputValueDefinition>,
    resolvers: Vec<Resolver>,
    /// All the field types in the supergraph, deduplicated.
    types: Vec<Type>,
    // All definitions sorted by their name (actual string)
    definitions: Vec<Definition>,
    directives: Vec<Directive>,
    enum_values: Vec<EnumValue>,

    /// All strings deduplicated.
    strings: Vec<String>,
    urls: Vec<url::Url>,
    /// Default input values & directive arguments
    input_values: SchemaInputValues,

    /// Headers we might want to send to a subgraph
    headers: Vec<Header>,
    default_headers: Vec<HeaderId>,
    cache_configs: Vec<CacheConfig>,

    pub auth_config: Option<config::latest::AuthConfig>,
    pub operation_limits: config::latest::OperationLimits,
    pub disable_introspection: bool,
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

    #[cfg(test)]
    pub(crate) fn empty() -> Self {
        Self {
            data_sources: Default::default(),
            description: None,
            root_operation_types: crate::RootOperationTypes {
                query: ObjectId::from(0),
                mutation: None,
                subscription: None,
            },
            objects: vec![Object {
                name: StringId::from(0),
                description: None,
                interfaces: Vec::new(),
                composed_directives: Directives::empty(),
                cache_config: None,
            }],
            object_fields: Vec::new(),
            interfaces: Vec::new(),
            interface_fields: Vec::new(),
            fields: Vec::new(),
            enums: Vec::new(),
            unions: Vec::new(),
            scalars: Vec::new(),
            input_objects: Vec::new(),
            input_value_definitions: Vec::new(),
            resolvers: Vec::new(),
            types: Vec::new(),
            definitions: Vec::new(),
            directives: Vec::new(),
            enum_values: Vec::new(),
            strings: vec![String::from("Query")],
            urls: Vec::new(),
            input_values: Default::default(),
            headers: Vec::new(),
            default_headers: Vec::new(),
            cache_configs: Vec::new(),
            auth_config: Default::default(),
            operation_limits: Default::default(),
            disable_introspection: false,
        }
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
    pub composed_directives: Directives,
    pub cache_config: Option<CacheConfigId>,
}

pub type Directives = IdRange<DirectiveId>;

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
    provides: Vec<FieldProvides>,
    /// The arguments referenced by this range are sorted by their name (string)
    pub argument_ids: IdRange<InputValueDefinitionId>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Directives,

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
pub enum Directive {
    Inaccessible,
    Deprecated { reason: Option<StringId> },
    Other { name: StringId, arguments: SchemaInputMap },
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

#[derive(Debug, Clone, Copy)]
pub struct Type {
    pub inner: Definition,
    pub wrapping: Wrapping,
}

impl Type {
    /// Determines whether a varia
    pub fn is_compatible_with(&self, other: Type) -> bool {
        self.inner == other.inner
            // if not a list, the current type can be coerced into the proper list wrapping.
            && (!self.wrapping.is_list()
                || self.wrapping.list_wrappings().len() == other.wrapping.list_wrappings().len())
            && (other.wrapping.is_nullable() || self.wrapping.is_required())
    }

    pub fn wrapped_by(self, list_wrapping: ListWrapping) -> Self {
        Self {
            inner: self.inner,
            wrapping: self.wrapping.wrapped_by(list_wrapping),
        }
    }
}

#[derive(Debug)]
pub struct Interface {
    pub name: StringId,
    pub description: Option<StringId>,
    pub interfaces: Vec<InterfaceId>,

    /// sorted by ObjectId
    pub possible_types: Vec<ObjectId>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Directives,
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
    /// The enum values referenced by this range are sorted by their name (string)
    pub value_ids: IdRange<EnumValueId>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Directives,
}

#[derive(Debug)]
pub struct EnumValue {
    pub name: StringId,
    pub description: Option<StringId>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Directives,
}

#[derive(Debug)]
pub struct Union {
    pub name: StringId,
    pub description: Option<StringId>,
    /// sorted by ObjectId
    pub possible_types: Vec<ObjectId>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Directives,
}

#[derive(Debug)]
pub struct Scalar {
    pub name: StringId,
    pub ty: ScalarType,
    pub description: Option<StringId>,
    pub specified_by_url: Option<StringId>,
    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Directives,
}

/// Defines how a scalar should be represented and validated by the engine. They're almost the same
/// as scalars, but scalars like ID which have no own data format are just mapped to String.
/// https://the-guild.dev/graphql/scalars/docs
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString)]
pub enum ScalarType {
    String,
    Float,
    Int,
    BigInt,
    JSON,
    Boolean,
}

impl ScalarType {
    pub fn from_scalar_name(name: &str) -> ScalarType {
        ScalarType::from_str(name).ok().unwrap_or(match name {
            "ID" => ScalarType::String,
            _ => ScalarType::JSON,
        })
    }
}

#[derive(Debug)]
pub struct InputObject {
    pub name: StringId,
    pub description: Option<StringId>,
    /// The input fields referenced by this range are sorted by their name (string)
    pub input_field_ids: IdRange<InputValueDefinitionId>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Directives,
}

#[derive(Debug, Clone)]
pub struct InputValueDefinition {
    pub name: StringId,
    pub description: Option<StringId>,
    pub type_id: TypeId,
    pub default_value: Option<SchemaInputValueId>,
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
