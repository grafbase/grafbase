use std::str::FromStr;

mod cache;
mod conversion;
mod field_set;
mod ids;
mod names;
mod resolver;
pub mod sources;
mod walkers;

pub use cache::*;
pub use field_set::*;
pub use ids::*;
pub use names::Names;
pub use resolver::*;
pub use walkers::*;

/// This does NOT need to be backwards compatible. We'll probably cache it for performance, but it is not
/// the source of truth. If the cache is stale we would just re-create this Graph from its source:
/// federated_graph::FederatedGraph.
pub struct Schema {
    pub description: Option<StringId>,
    pub root_operation_types: RootOperationTypes,
    data_sources: DataSources,

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

    /// Headers we might want to send to a subgraph
    headers: Vec<Header>,

    default_headers: Vec<HeaderId>,

    cache_configs: Vec<CacheConfig>,
}

#[derive(Default)]
struct DataSources {
    federation: sources::federation::DataSource,
    introspection: sources::introspection::DataSource,
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

    fn finalize(mut self) -> Self {
        self.definitions = Vec::with_capacity(
            self.scalars.len()
                + self.objects.len()
                + self.interfaces.len()
                + self.unions.len()
                + self.enums.len()
                + self.input_objects.len(),
        );
        // Adding all definitions for introspection & query binding
        self.definitions
            .extend((0..self.scalars.len()).map(|id| Definition::Scalar(ScalarId::from(id))));
        self.definitions
            .extend((0..self.objects.len()).map(|id| Definition::Object(ObjectId::from(id))));
        self.definitions
            .extend((0..self.interfaces.len()).map(|id| Definition::Interface(InterfaceId::from(id))));
        self.definitions
            .extend((0..self.unions.len()).map(|id| Definition::Union(UnionId::from(id))));
        self.definitions
            .extend((0..self.enums.len()).map(|id| Definition::Enum(EnumId::from(id))));
        self.definitions
            .extend((0..self.input_objects.len()).map(|id| Definition::InputObject(InputObjectId::from(id))));

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

        assert!(matches!(self.resolvers.last(), Some(Resolver::Introspection(_))));

        self
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

    /// Special case when only going through this field children are accessible.
    /// If this field is shared across different sources, we assume identical behavior
    /// and thus identical `@provides` if any.
    pub provides: FieldSet,

    pub arguments: Vec<InputValueId>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,

    pub cache_config: Option<CacheConfigId>,
}

#[derive(Debug)]
pub struct FieldResolver {
    resolver_id: ResolverId,
    requires: FieldSet,
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

    pub fn is_list(&self) -> bool {
        !self.list_wrapping.is_empty()
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
