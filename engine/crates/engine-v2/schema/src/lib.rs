use std::{str::FromStr, sync::OnceLock};

mod builder;
mod definition;
mod directive;
mod entity;
mod enum_def;
mod field;
mod field_set;
mod generated;
mod ids;
mod input_value;
mod interface;
pub mod introspection;
mod object;
mod prelude;
mod resolver;
mod subgraph;
mod ty;
mod union;

pub use self::builder::BuildError;
pub use directive::*;
pub use field_set::*;
pub use generated::*;
use id_newtypes::IdRange;
pub use ids::*;
pub use input_value::*;
use regex::Regex;
pub use subgraph::*;
use walker::{Iter, Walk};
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
            None => built_info::BUILD_TOKEN.as_bytes().to_vec(),
        })
    }
}

pub type Walker<'a, T> = walker::Walker<'a, T, Schema>;

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
    pub subgraphs: SubGraphs,
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

id_newtypes::forward! {
    impl Index<SchemaInputValueId, Output = SchemaInputValueRecord> for Schema.graph.input_values,
    impl Index<SchemaInputObjectFieldValueId, Output = (InputValueDefinitionId, SchemaInputValueRecord)> for Schema.graph.input_values,
    impl Index<SchemaInputKeyValueId, Output = (StringId, SchemaInputValueRecord)> for Schema.graph.input_values,
    impl Index<GraphqlEndpointId, Output = GraphqlEndpointRecord> for Schema.subgraphs,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    default_header_rules: Vec<HeaderRuleId>,

    pub timeout: std::time::Duration,
    pub auth_config: Option<config::latest::AuthConfig>,
    pub operation_limits: config::latest::OperationLimits,
    pub disable_introspection: bool,
    pub retry: Option<RetryConfig>,
}

#[derive(serde::Serialize, serde::Deserialize, id_derives::IndexedFields)]
pub struct Graph {
    pub description_id: Option<StringId>,
    pub root_operation_types_record: RootOperationTypesRecord,

    // All type definitions sorted by their name (actual string)
    type_definitions_ordered_by_name: Vec<DefinitionId>,
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
    required_field_sets: Vec<RequiredFieldSetRecord>,
    // deduplicated
    #[indexed_by(RequiredFieldId)]
    required_fields: Vec<RequiredFieldRecord>,
    /// Default input values & directive arguments
    pub input_values: SchemaInputValues,

    #[indexed_by(RequiresScopesDirectiveId)]
    required_scopes: Vec<RequiresScopesDirectiveRecord>,
    #[indexed_by(AuthorizedDirectiveId)]
    authorized_directives: Vec<AuthorizedDirectiveRecord>,
}

#[derive(serde::Serialize, serde::Deserialize, id_derives::IndexedFields)]
pub struct SubGraphs {
    #[indexed_by(GraphqlEndpointId)]
    graphql_endpoints: Vec<GraphqlEndpointRecord>,
    pub introspection: introspection::IntrospectionMetadata,
}

impl Schema {
    pub fn walk<T: Walk<Self>>(&self, item: T) -> Walker<'_, T> {
        item.walk(self)
    }

    pub fn definitions(&self) -> impl Iter<Item = Definition<'_>> + '_ {
        self.graph.type_definitions_ordered_by_name.walk(self)
    }

    pub fn definition_by_name(&self, name: &str) -> Option<DefinitionId> {
        self.graph
            .type_definitions_ordered_by_name
            .binary_search_by_key(&name, |definition| self.definition_name(*definition))
            .map(|index| self.graph.type_definitions_ordered_by_name[index])
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

    pub fn default_header_rules(&self) -> impl Iter<Item = HeaderRule<'_>> + '_ {
        self.settings.default_header_rules.walk(self)
    }

    fn definition_name(&self, definition: DefinitionId) -> &str {
        definition.walk(self).name()
    }

    pub fn query(&self) -> ObjectDefinition<'_> {
        self.graph.root_operation_types_record.query_id.walk(self)
    }

    pub fn mutation(&self) -> Option<ObjectDefinition<'_>> {
        self.graph.root_operation_types_record.mutation_id.walk(self)
    }

    pub fn subscription(&self) -> Option<ObjectDefinition<'_>> {
        self.graph.root_operation_types_record.subscription_id.walk(self)
    }

    pub fn graphql_endpoints(&self) -> impl ExactSizeIterator<Item = GraphqlEndpoint<'_>> {
        (0..self.subgraphs.graphql_endpoints.len()).map(|i| {
            let id = GraphqlEndpointId::from(i);
            id.walk(self)
        })
    }

    pub fn scalar_definition_by_name(&self, name: &str) -> Option<ScalarDefinitionId> {
        self.graph
            .scalar_definitions
            .iter()
            .position(|definition| self[definition.name_id] == name)
            .map(ScalarDefinitionId::from)
    }
}

impl std::fmt::Debug for Schema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Schema").finish_non_exhaustive()
    }
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
