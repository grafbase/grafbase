use std::str::FromStr;

mod builder;
mod directives;
mod ids;
mod input_value;
mod names;
mod provides;
mod requires;
mod resolver;
pub mod sources;
mod walkers;

pub use directives::*;
use id_newtypes::IdRange;
pub use ids::*;
pub use input_value::*;
pub use names::Names;
pub use provides::*;
pub use requires::*;
pub use resolver::*;
pub use walkers::*;
pub use wrapping::*;

mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

impl Schema {
    pub fn commit_sha() -> Vec<u8> {
        hex::decode(built_info::GIT_COMMIT_HASH.expect("No git commit hash found")).expect("Expect hex format")
    }
}

/// /!\ This is *NOT* backwards-compatible. /!\
/// Only a schema serialized with the exact same version is expected to work. For backwards
/// compatibility use engine-v2-config instead.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Schema {
    data_sources: DataSources,
    graph: Graph,

    /// All strings deduplicated.
    strings: Vec<String>,
    urls: Vec<url::Url>,
    /// Headers we might want to send to a subgraph
    headers: Vec<Header>,

    pub settings: Settings,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    default_headers: Vec<HeaderId>,

    pub auth_config: Option<config::latest::AuthConfig>,
    pub operation_limits: config::latest::OperationLimits,
    pub disable_introspection: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Graph {
    pub description: Option<StringId>,
    pub root_operation_types: RootOperationTypes,

    // All type definitions sorted by their name (actual string)
    type_definitions: Vec<Definition>,
    object_definitions: Vec<Object>,
    interface_definitions: Vec<Interface>,
    field_definitions: Vec<FieldDefinition>,
    field_to_parent_entity: Vec<EntityId>,
    enum_definitions: Vec<Enum>,
    union_definitions: Vec<Union>,
    scalar_definitions: Vec<Scalar>,
    input_object_definitions: Vec<InputObject>,
    input_value_definitions: Vec<InputValueDefinition>,
    enum_value_definitions: Vec<EnumValue>,

    type_system_directives: Vec<TypeSystemDirective>,
    resolvers: Vec<Resolver>,
    required_field_sets: Vec<RequiredFieldSet>,
    // deduplicated
    required_fields: Vec<RequiredField>,
    /// Default input values & directive arguments
    input_values: SchemaInputValues,
    cache_control: Vec<CacheControl>,
    required_scopes: Vec<RequiredScopes>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct DataSources {
    graphql: sources::GraphqlEndpoints,
    pub introspection: sources::IntrospectionMetadata,
}

impl Schema {
    pub fn definition_by_name(&self, name: &str) -> Option<Definition> {
        self.graph
            .type_definitions
            .binary_search_by_key(&name, |definition| self.definition_name(*definition))
            .map(|index| self.graph.type_definitions[index])
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
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Object {
    pub name: StringId,
    pub description: Option<StringId>,
    pub interfaces: Vec<InterfaceId>,
    pub directives: IdRange<TypeSystemDirectiveId>,
    pub fields: IdRange<FieldDefinitionId>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
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
    pub directives: IdRange<TypeSystemDirectiveId>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FieldProvides {
    subgraph_id: SubgraphId,
    field_set: ProvidableFieldSet,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FieldRequires {
    subgraph_id: SubgraphId,
    field_set_id: RequiredFieldSetId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub enum EntityId {
    Object(ObjectId),
    Interface(InterfaceId),
}

impl From<EntityId> for Definition {
    fn from(value: EntityId) -> Self {
        match value {
            EntityId::Interface(id) => Definition::Interface(id),
            EntityId::Object(id) => Definition::Object(id),
        }
    }
}

impl EntityId {
    pub fn maybe_from(definition: Definition) -> Option<EntityId> {
        match definition {
            Definition::Object(id) => Some(EntityId::Object(id)),
            Definition::Interface(id) => Some(EntityId::Interface(id)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Interface {
    pub name: StringId,
    pub description: Option<StringId>,
    pub interfaces: Vec<InterfaceId>,

    /// sorted by ObjectId
    pub possible_types: Vec<ObjectId>,
    pub directives: IdRange<TypeSystemDirectiveId>,
    pub fields: IdRange<FieldDefinitionId>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Enum {
    pub name: StringId,
    pub description: Option<StringId>,
    /// The enum values referenced by this range are sorted by their name (string)
    pub value_ids: IdRange<EnumValueId>,
    pub directives: IdRange<TypeSystemDirectiveId>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EnumValue {
    pub name: StringId,
    pub description: Option<StringId>,
    pub directives: IdRange<TypeSystemDirectiveId>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Union {
    pub name: StringId,
    pub description: Option<StringId>,
    /// sorted by ObjectId
    pub possible_types: Vec<ObjectId>,
    pub directives: IdRange<TypeSystemDirectiveId>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Scalar {
    pub name: StringId,
    pub ty: ScalarType,
    pub description: Option<StringId>,
    pub specified_by_url: Option<StringId>,
    pub directives: IdRange<TypeSystemDirectiveId>,
}

/// Defines how a scalar should be represented and validated by the engine. They're almost the same
/// as scalars, but scalars like ID which have no own data format are just mapped to String.
/// https://the-guild.dev/graphql/scalars/docs
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString, serde::Serialize, serde::Deserialize,
)]
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

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct InputObject {
    pub name: StringId,
    pub description: Option<StringId>,
    /// The input fields referenced by this range are sorted by their name (string)
    pub input_field_ids: IdRange<InputValueDefinitionId>,
    pub directives: IdRange<TypeSystemDirectiveId>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InputValueDefinition {
    pub name: StringId,
    pub description: Option<StringId>,
    pub ty: Type,
    pub default_value: Option<SchemaInputValueId>,
    pub directives: IdRange<TypeSystemDirectiveId>,
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

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Header {
    pub name: StringId,
    pub value: HeaderValue,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum HeaderValue {
    Forward(StringId),
    Static(StringId),
}
