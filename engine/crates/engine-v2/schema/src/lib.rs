use std::{str::FromStr, sync::OnceLock};

mod builder;
mod directives;
mod ids;
mod input_value;
mod input_value_set;
mod provides;
mod requires;
mod resolver;
pub mod sources;
mod walkers;

pub use self::builder::BuildError;
pub use directives::*;
use id_newtypes::IdRange;
pub use ids::*;
pub use input_value::*;
pub use input_value_set::*;
pub use provides::*;
use regex::Regex;
pub use requires::*;
pub use resolver::*;
use sources::{GraphqlEndpoints, IntrospectionMetadata};
pub use walkers::*;
pub use wrapping::*;

mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

impl Schema {
    /// A unique identifier of this build of the engine to version cache keys.
    /// If built in a git repository, the cache version is taken from the git commit id.
    /// For builds outside of a git repository, the build time is used.
    pub fn build_identifier() -> &'static [u8] {
        static SHA: OnceLock<Vec<u8>> = OnceLock::new();

        SHA.get_or_init(|| match built_info::GIT_COMMIT_HASH {
            Some(hash) => hex::decode(hash).expect("Expect hex format"),
            None => built_info::BUILT_TIME_UTC.as_bytes().to_vec(),
        })
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Version(Vec<u8>);

impl<T: AsRef<[u8]>> From<T> for Version {
    fn from(value: T) -> Self {
        Version(value.as_ref().to_vec())
    }
}

impl std::ops::Deref for Version {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// /!\ This is *NOT* backwards-compatible. /!\
/// Only a schema serialized with the exact same version is expected to work. For backwards
/// compatibility use engine-v2-config instead.
#[derive(serde::Serialize, serde::Deserialize, id_derives::IndexedFields)]
pub struct Schema {
    data_sources: DataSources,
    pub graph: Graph,
    pub version: Version,

    /// All strings deduplicated.
    #[indexed_by(StringId)]
    strings: Vec<String>,
    #[serde(with = "serde_regex")]
    #[indexed_by(RegexId)]
    regexps: Vec<Regex>,
    #[indexed_by(UrlId)]
    urls: Vec<url::Url>,
    /// Headers we might want to send to a subgraph
    #[indexed_by(HeaderRuleId)]
    header_rules: Vec<HeaderRuleRecord>,

    pub settings: Settings,
}

impl Schema {
    pub fn build(config: config::latest::Config, version: Version) -> Result<Schema, BuildError> {
        builder::build(config, version)
    }
}

impl<T> std::ops::Index<T> for Schema
where
    Graph: std::ops::Index<T>,
{
    type Output = <Graph as std::ops::Index<T>>::Output;
    fn index(&self, index: T) -> &Self::Output {
        &self.graph[index]
    }
}

impl std::ops::Index<SchemaInputValueId> for Schema {
    type Output = SchemaInputValueRecord;
    fn index(&self, index: SchemaInputValueId) -> &Self::Output {
        &self.graph.input_values[index]
    }
}

impl std::ops::Index<SchemaInputKeyValueId> for Schema {
    type Output = (StringId, SchemaInputValueRecord);
    fn index(&self, index: SchemaInputKeyValueId) -> &Self::Output {
        &self.graph.input_values[index]
    }
}

impl std::ops::Index<SchemaInputObjectFieldValueId> for Schema {
    type Output = (InputValueDefinitionId, SchemaInputValueRecord);
    fn index(&self, index: SchemaInputObjectFieldValueId) -> &Self::Output {
        &self.graph.input_values[index]
    }
}

impl std::ops::Index<IdRange<SchemaInputValueId>> for Schema {
    type Output = [SchemaInputValueRecord];
    fn index(&self, index: IdRange<SchemaInputValueId>) -> &Self::Output {
        &self.graph.input_values[index]
    }
}

impl std::ops::Index<IdRange<SchemaInputKeyValueId>> for Schema {
    type Output = [(StringId, SchemaInputValueRecord)];
    fn index(&self, index: IdRange<SchemaInputKeyValueId>) -> &Self::Output {
        &self.graph.input_values[index]
    }
}

impl std::ops::Index<IdRange<SchemaInputObjectFieldValueId>> for Schema {
    type Output = [(InputValueDefinitionId, SchemaInputValueRecord)];
    fn index(&self, index: IdRange<SchemaInputObjectFieldValueId>) -> &Self::Output {
        &self.graph.input_values[index]
    }
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    default_header_rules: Vec<HeaderRuleId>,

    pub timeout: std::time::Duration,
    pub auth_config: Option<config::latest::AuthConfig>,
    pub operation_limits: config::latest::OperationLimits,
    pub disable_introspection: bool,
    pub retry: Option<config::latest::RetryConfig>,
}

#[derive(serde::Serialize, serde::Deserialize, id_derives::IndexedFields)]
pub struct Graph {
    pub description: Option<StringId>,
    pub root_operation_types: RootOperationTypes,

    // All type definitions sorted by their name (actual string)
    #[indexed_by(DefinitionId)]
    type_definitions: Vec<Definition>,
    #[indexed_by(ObjectDefinitionId)]
    object_definitions: Vec<ObjectDefinitionRecord>,
    #[indexed_by(InterfaceDefinitionId)]
    interface_definitions: Vec<InterfaceDefinitionRecord>,
    #[indexed_by(FieldDefinitionId)]
    field_definitions: Vec<FieldDefinitionRecord>,
    #[indexed_by(EnumDefinitionId)]
    enum_definitions: Vec<EnumDefinitionRecord>,
    #[indexed_by(UnionDefinitionId)]
    union_definitions: Vec<UnionDefinitionRecord>,
    #[indexed_by(ScalarDefinitionId)]
    scalar_definitions: Vec<ScalarDefinitionRecord>,
    #[indexed_by(InputObjectDefinitionId)]
    input_object_definitions: Vec<InputObjectDefinitionRecord>,
    #[indexed_by(InputValueDefinitionId)]
    input_value_definitions: Vec<InputValueDefinitionRecord>,
    #[indexed_by(EnumValueId)]
    enum_value_definitions: Vec<EnumValueRecord>,

    #[indexed_by(ResolverDefinitionId)]
    resolver_definitions: Vec<ResolverDefinitionRecord>,
    #[indexed_by(RequiredFieldSetId)]
    required_field_sets: Vec<RequiredFieldSet>,
    // deduplicated
    #[indexed_by(RequiredFieldId)]
    required_fields: Vec<RequiredField>,
    /// Default input values & directive arguments
    pub input_values: SchemaInputValues,

    #[indexed_by(TypeSystemDirectiveId)]
    type_system_directives: Vec<TypeSystemDirective>,
    #[indexed_by(CacheControlId)]
    cache_control: Vec<CacheControl>,
    #[indexed_by(RequiredScopesId)]
    required_scopes: Vec<RequiredScopesRecord>,
    #[indexed_by(AuthorizedDirectiveId)]
    authorized_directives: Vec<AuthorizedDirectiveRecord>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct DataSources {
    graphql: GraphqlEndpoints,
    pub introspection: IntrospectionMetadata,
}

impl Schema {
    pub fn definition_by_name(&self, name: &str) -> Option<Definition> {
        self.graph
            .type_definitions
            .binary_search_by_key(&name, |definition| self.definition_name(*definition))
            .map(|index| self.graph.type_definitions[index])
            .ok()
    }

    pub fn object_field_by_name(&self, object_id: ObjectDefinitionId, name: &str) -> Option<FieldDefinitionId> {
        let fields = self[object_id].field_ids;
        self[fields]
            .iter()
            .position(|field| self[field.name_id] == name)
            .map(|pos| FieldDefinitionId::from(usize::from(fields.start) + pos))
    }

    pub fn interface_field_by_name(
        &self,
        interface_id: InterfaceDefinitionId,
        name: &str,
    ) -> Option<FieldDefinitionId> {
        let fields = self[interface_id].field_ids;
        self[fields]
            .iter()
            .position(|field| self[field.name_id] == name)
            .map(|pos| FieldDefinitionId::from(usize::from(fields.start) + pos))
    }

    fn definition_name(&self, definition: Definition) -> &str {
        let name = match definition {
            Definition::Scalar(s) => self[s].name_id,
            Definition::Object(o) => self[o].name_id,
            Definition::Interface(i) => self[i].name_id,
            Definition::Union(u) => self[u].name_id,
            Definition::Enum(e) => self[e].name_id,
            Definition::InputObject(io) => self[io].name_id,
        };
        &self[name]
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RootOperationTypes {
    pub query_id: ObjectDefinitionId,
    pub mutation_id: Option<ObjectDefinitionId>,
    pub subscription_id: Option<ObjectDefinitionId>,
}

impl std::fmt::Debug for Schema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Schema").finish_non_exhaustive()
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ObjectDefinitionRecord {
    pub name_id: StringId,
    pub description_id: Option<StringId>,
    pub interface_ids: Vec<InterfaceDefinitionId>,
    pub directive_ids: IdRange<TypeSystemDirectiveId>,
    pub field_ids: IdRange<FieldDefinitionId>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FieldDefinitionRecord {
    pub name_id: StringId,
    pub parent_entity_id: EntityDefinitionId,
    pub description_id: Option<StringId>,
    pub ty: TypeRecord,
    pub resolver_ids: Vec<ResolverDefinitionId>,
    /// By default a field is considered shared and providable by *any* subgraph that exposes it.
    /// It's up to the composition to ensure it. If this field is specific to some subgraphs, they
    /// will be specified in this Vec.
    pub only_resolvable_in_ids: Vec<SubgraphId>,
    pub requires: Vec<FieldRequires>,
    pub provides: Vec<FieldProvides>,
    /// The arguments referenced by this range are sorted by their name (string)
    pub argument_ids: IdRange<InputValueDefinitionId>,
    pub directive_ids: IdRange<TypeSystemDirectiveId>,
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
pub enum EntityDefinitionId {
    Object(ObjectDefinitionId),
    Interface(InterfaceDefinitionId),
}

impl From<ObjectDefinitionId> for EntityDefinitionId {
    fn from(id: ObjectDefinitionId) -> Self {
        EntityDefinitionId::Object(id)
    }
}

impl From<InterfaceDefinitionId> for EntityDefinitionId {
    fn from(id: InterfaceDefinitionId) -> Self {
        EntityDefinitionId::Interface(id)
    }
}

impl From<EntityDefinitionId> for Definition {
    fn from(value: EntityDefinitionId) -> Self {
        match value {
            EntityDefinitionId::Interface(id) => Definition::Interface(id),
            EntityDefinitionId::Object(id) => Definition::Object(id),
        }
    }
}

impl EntityDefinitionId {
    pub fn maybe_from(definition: Definition) -> Option<EntityDefinitionId> {
        match definition {
            Definition::Object(id) => Some(EntityDefinitionId::Object(id)),
            Definition::Interface(id) => Some(EntityDefinitionId::Interface(id)),
            _ => None,
        }
    }

    pub fn is_object(&self) -> bool {
        matches!(self, EntityDefinitionId::Object(_))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub enum Definition {
    Scalar(ScalarDefinitionId),
    Object(ObjectDefinitionId),
    Interface(InterfaceDefinitionId),
    Union(UnionDefinitionId),
    Enum(EnumDefinitionId),
    InputObject(InputObjectDefinitionId),
}

impl Definition {
    pub fn is_input_object(&self) -> bool {
        matches!(self, Definition::InputObject(_))
    }
}

impl From<ScalarDefinitionId> for Definition {
    fn from(id: ScalarDefinitionId) -> Self {
        Self::Scalar(id)
    }
}

impl From<ObjectDefinitionId> for Definition {
    fn from(id: ObjectDefinitionId) -> Self {
        Self::Object(id)
    }
}

impl From<InterfaceDefinitionId> for Definition {
    fn from(id: InterfaceDefinitionId) -> Self {
        Self::Interface(id)
    }
}

impl From<UnionDefinitionId> for Definition {
    fn from(id: UnionDefinitionId) -> Self {
        Self::Union(id)
    }
}

impl From<EnumDefinitionId> for Definition {
    fn from(id: EnumDefinitionId) -> Self {
        Self::Enum(id)
    }
}

impl From<InputObjectDefinitionId> for Definition {
    fn from(id: InputObjectDefinitionId) -> Self {
        Self::InputObject(id)
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct TypeRecord {
    pub definition_id: Definition,
    pub wrapping: Wrapping,
}

impl TypeRecord {
    /// Determines whether a varia
    pub fn is_compatible_with(&self, other: TypeRecord) -> bool {
        self.definition_id == other.definition_id
            // if not a list, the current type can be coerced into the proper list wrapping.
            && (!self.wrapping.is_list()
                || self.wrapping.list_wrappings().len() == other.wrapping.list_wrappings().len())
            && (other.wrapping.is_nullable() || self.wrapping.is_required())
    }

    pub fn wrapped_by(self, list_wrapping: ListWrapping) -> Self {
        Self {
            definition_id: self.definition_id,
            wrapping: self.wrapping.wrapped_by(list_wrapping),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct InterfaceDefinitionRecord {
    pub name_id: StringId,
    pub description_id: Option<StringId>,
    pub interface_ids: Vec<InterfaceDefinitionId>,

    /// sorted by ObjectId
    pub possible_type_ids: Vec<ObjectDefinitionId>,
    pub possible_types_ordered_by_typename_ids: Vec<ObjectDefinitionId>,
    pub directive_ids: IdRange<TypeSystemDirectiveId>,
    pub field_ids: IdRange<FieldDefinitionId>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EnumDefinitionRecord {
    pub name_id: StringId,
    pub description_id: Option<StringId>,
    /// The enum values referenced by this range are sorted by their name (string)
    pub value_ids: IdRange<EnumValueId>,
    pub directive_ids: IdRange<TypeSystemDirectiveId>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EnumValueRecord {
    pub name_id: StringId,
    pub description_id: Option<StringId>,
    pub directive_ids: IdRange<TypeSystemDirectiveId>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct UnionDefinitionRecord {
    pub name_id: StringId,
    pub description_id: Option<StringId>,
    /// sorted by ObjectId
    pub possible_type_ids: Vec<ObjectDefinitionId>,
    pub possible_types_ordered_by_typename_ids: Vec<ObjectDefinitionId>,
    pub directive_ids: IdRange<TypeSystemDirectiveId>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ScalarDefinitionRecord {
    pub name_id: StringId,
    pub ty: ScalarType,
    pub description_id: Option<StringId>,
    pub specified_by_url_id: Option<StringId>,
    pub directive_ids: IdRange<TypeSystemDirectiveId>,
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
pub struct InputObjectDefinitionRecord {
    pub name_id: StringId,
    pub description_id: Option<StringId>,
    /// The input fields referenced by this range are sorted by their name (string)
    pub input_field_ids: IdRange<InputValueDefinitionId>,
    pub directive_ids: IdRange<TypeSystemDirectiveId>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InputValueDefinitionRecord {
    pub name_id: StringId,
    pub description_id: Option<StringId>,
    pub ty: TypeRecord,
    pub default_value_id: Option<SchemaInputValueId>,
    pub directive_ids: IdRange<TypeSystemDirectiveId>,
}

impl Schema {
    pub fn walk<I>(&self, item: I) -> SchemaWalker<'_, I> {
        SchemaWalker::new(item, self)
    }

    pub fn walker(&self) -> SchemaWalker<'_, ()> {
        SchemaWalker::new((), self)
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub enum NameOrPattern {
    /// A regex pattern matching multiple headers.
    Pattern(RegexId),
    /// A static single name.
    Name(StringId),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum HeaderRuleRecord {
    Forward {
        name_id: NameOrPattern,
        default: Option<StringId>,
        rename: Option<StringId>,
    },
    Insert {
        name_id: StringId,
        value: StringId,
    },
    Remove {
        name_id: NameOrPattern,
    },
    RenameDuplicate {
        name_id: StringId,
        default: Option<StringId>,
        rename: StringId,
    },
}
