use std::str::FromStr;

mod builder;
mod cache;
mod ids;
mod input_value;
mod names;
mod provides;
mod requires;
mod resolver;
pub mod sources;
mod walkers;

pub use cache::*;
use id_newtypes::IdRange;
pub use ids::*;
pub use input_value::*;
pub use names::Names;
pub use provides::*;
pub use requires::*;
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
    interfaces: Vec<Interface>,
    field_definitions: Vec<FieldDefinition>,
    enums: Vec<Enum>,
    unions: Vec<Union>,
    scalars: Vec<Scalar>,
    input_objects: Vec<InputObject>,
    input_value_definitions: Vec<InputValueDefinition>,
    resolvers: Vec<Resolver>,
    // All definitions sorted by their name (actual string)
    definitions: Vec<Definition>,
    directives: Vec<Directive>,
    enum_values: Vec<EnumValue>,

    required_field_sets: Vec<RequiredFieldSet>,
    // deduplicated
    required_fields_arguments: Vec<RequiredFieldArguments>,

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
    graphql: sources::GraphqlEndpoints,
    pub introspection: sources::Introspection,
}

impl Schema {
    pub fn definition_by_name(&self, name: &str) -> Option<Definition> {
        self.definitions
            .binary_search_by_key(&name, |definition| self.definition_name(*definition))
            .map(|index| self.definitions[index])
            .ok()
    }

    pub fn object_field_by_name(&self, object_id: ObjectId, name: &str) -> Option<FieldDefinitionId> {
        let fields = self[object_id].fields;
        self[fields]
            .iter()
            .position(|field| self[field.name] == name)
            .map(|pos| FieldDefinitionId::from(usize::from(fields.start) + pos))
    }

    pub fn interface_field_by_name(&self, interface_id: InterfaceId, name: &str) -> Option<FieldDefinitionId> {
        let fields = self[interface_id].fields;
        self[fields]
            .iter()
            .position(|field| self[field.name] == name)
            .map(|pos| FieldDefinitionId::from(usize::from(fields.start) + pos))
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
                composed_directives: IdRange::empty(),
                cache_config: None,
                fields: IdRange::empty(),
            }],
            required_field_sets: Vec::new(),
            required_fields_arguments: Vec::new(),
            interfaces: Vec::new(),
            field_definitions: Vec::new(),
            enums: Vec::new(),
            unions: Vec::new(),
            scalars: Vec::new(),
            input_objects: Vec::new(),
            input_value_definitions: Vec::new(),
            resolvers: Vec::new(),
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
    pub composed_directives: IdRange<DirectiveId>,
    pub cache_config: Option<CacheConfigId>,
    pub fields: IdRange<FieldDefinitionId>,
}

#[derive(Debug)]
pub struct FieldDefinition {
    pub name: StringId,
    pub description: Option<StringId>,
    pub ty: Type,
    pub resolvers: Vec<ResolverId>,
    /// By default a field is considered shared and providable by *any* subgraph that exposes it.
    /// It's up to the composition to ensure it. If this field is specific to some subgraphs, they
    /// will be specified in this Vec.
    pub only_resolvable_in: Vec<SubgraphId>,
    pub requires: Vec<FieldRequires>,
    pub provides: Vec<FieldProvides>,
    /// The arguments referenced by this range are sorted by their name (string)
    pub argument_ids: IdRange<InputValueDefinitionId>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: IdRange<DirectiveId>,

    pub cache_config: Option<CacheConfigId>,
}

#[derive(Debug)]
pub struct FieldProvides {
    subgraph_id: SubgraphId,
    field_set: ProvidableFieldSet,
}

#[derive(Debug)]
pub struct FieldRequires {
    subgraph_id: SubgraphId,
    field_set_id: RequiredFieldSetId,
}

#[derive(Debug)]
pub enum Directive {
    Inaccessible,
    Authenticated,
    Policy(Vec<Vec<StringId>>),
    RequiresScopes(Vec<Vec<StringId>>),
    Deprecated { reason: Option<StringId> },
    Other,
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
    pub composed_directives: IdRange<DirectiveId>,

    pub fields: IdRange<FieldDefinitionId>,
}

#[derive(Debug)]
pub struct Enum {
    pub name: StringId,
    pub description: Option<StringId>,
    /// The enum values referenced by this range are sorted by their name (string)
    pub value_ids: IdRange<EnumValueId>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: IdRange<DirectiveId>,
}

#[derive(Debug)]
pub struct EnumValue {
    pub name: StringId,
    pub description: Option<StringId>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: IdRange<DirectiveId>,
}

#[derive(Debug)]
pub struct Union {
    pub name: StringId,
    pub description: Option<StringId>,
    /// sorted by ObjectId
    pub possible_types: Vec<ObjectId>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: IdRange<DirectiveId>,
}

#[derive(Debug)]
pub struct Scalar {
    pub name: StringId,
    pub ty: ScalarType,
    pub description: Option<StringId>,
    pub specified_by_url: Option<StringId>,
    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: IdRange<DirectiveId>,
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
    pub composed_directives: IdRange<DirectiveId>,
}

#[derive(Debug, Clone)]
pub struct InputValueDefinition {
    pub name: StringId,
    pub description: Option<StringId>,
    pub ty: Type,
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
