use std::sync::OnceLock;

mod builder;
mod composite_type;
mod definition;
mod directive;
mod entity;
mod enum_def;
mod enum_value;
mod field;
mod field_set;
mod generated;
mod ids;
mod input_object;
mod input_value;
mod input_value_def;
mod interface;
pub mod introspection;
mod object;
mod prelude;
mod resolver;
mod scalar;
mod subgraph;
mod ty;
mod union;

pub use self::builder::BuildError;
use config::ResponseExtensionConfig;
pub use directive::*;
use extension_catalog::ExtensionCatalog;
pub use field_set::*;
pub use gateway_config::SubscriptionProtocol;
pub use generated::*;
use id_newtypes::{BitSet, IdRange};
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

pub type Walker<'a, T> = walker::Walker<'a, T, &'a Schema>;

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
/// compatibility use engine-config instead.
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
    pub fn from_sdl_or_panic(sdl: &str) -> Self {
        let graph = federated_graph::FederatedGraph::from_sdl(sdl).unwrap();
        let config = config::Config::from_graph(graph);
        Self::build(config, Version::from(Vec::new()), &Default::default()).unwrap()
    }

    pub fn build(
        config: config::Config,
        version: Version,
        extension_catalog: &ExtensionCatalog,
    ) -> Result<Schema, BuildError> {
        builder::build(config, version, extension_catalog)
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
    impl Index<VirtualSubgraphId, Output = VirtualSubgraphRecord> for Schema.subgraphs,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    default_header_rules: Vec<HeaderRuleId>,

    pub timeout: std::time::Duration,
    pub auth_config: Option<config::AuthConfig>,
    pub operation_limits: config::OperationLimits,
    pub disable_introspection: bool,
    pub retry: Option<RetryConfig>,
    pub batching: config::BatchingConfig,
    pub complexity_control: config::ComplexityControl,
    pub response_extension: ResponseExtensionConfig,
    pub apq_enabled: bool,
    pub executable_document_limit_bytes: usize,
    pub trusted_documents: config::TrustedDocumentsConfig,
    pub websocket_forward_connection_init_payload: bool,
}

#[derive(serde::Serialize, serde::Deserialize, id_derives::IndexedFields)]
pub struct Graph {
    pub description_id: Option<StringId>,
    pub root_operation_types_record: RootOperationTypesRecord,

    // All type definitions sorted by their name (actual string)
    type_definitions_ordered_by_name: Vec<DefinitionId>,
    #[indexed_by(ObjectDefinitionId)]
    object_definitions: Vec<ObjectDefinitionRecord>,
    inaccessible_object_definitions: BitSet<ObjectDefinitionId>,
    #[indexed_by(InterfaceDefinitionId)]
    interface_definitions: Vec<InterfaceDefinitionRecord>,
    inaccessible_interface_definitions: BitSet<InterfaceDefinitionId>,
    interface_has_inaccessible_implementor: BitSet<InterfaceDefinitionId>,
    #[indexed_by(FieldDefinitionId)]
    field_definitions: Vec<FieldDefinitionRecord>,
    inaccessible_field_definitions: BitSet<FieldDefinitionId>,
    #[indexed_by(EnumDefinitionId)]
    enum_definitions: Vec<EnumDefinitionRecord>,
    inaccessible_enum_definitions: BitSet<EnumDefinitionId>,
    #[indexed_by(UnionDefinitionId)]
    union_definitions: Vec<UnionDefinitionRecord>,
    inaccessible_union_definitions: BitSet<UnionDefinitionId>,
    union_has_inaccessible_member: BitSet<UnionDefinitionId>,
    #[indexed_by(ScalarDefinitionId)]
    scalar_definitions: Vec<ScalarDefinitionRecord>,
    inaccessible_scalar_definitions: BitSet<ScalarDefinitionId>,
    #[indexed_by(InputObjectDefinitionId)]
    input_object_definitions: Vec<InputObjectDefinitionRecord>,
    inaccessible_input_object_definitions: BitSet<InputObjectDefinitionId>,
    #[indexed_by(InputValueDefinitionId)]
    input_value_definitions: Vec<InputValueDefinitionRecord>,
    inaccessible_input_value_definitions: BitSet<InputValueDefinitionId>,
    #[indexed_by(EnumValueId)]
    enum_values: Vec<EnumValueRecord>,
    inaccessible_enum_values: BitSet<EnumValueId>,

    #[indexed_by(ResolverDefinitionId)]
    resolver_definitions: Vec<ResolverDefinitionRecord>,

    #[indexed_by(FieldSetId)]
    field_sets: Vec<FieldSetRecord>,
    // deduplicated
    #[indexed_by(SchemaFieldId)]
    fields: Vec<SchemaFieldRecord>,
    #[indexed_by(SchemaFieldArgumentId)]
    field_arguments: Vec<SchemaFieldArgumentRecord>,

    /// Default input values & directive arguments
    pub input_values: SchemaInputValues,

    #[indexed_by(RequiresScopesDirectiveId)]
    required_scopes: Vec<RequiresScopesDirectiveRecord>,
    #[indexed_by(AuthorizedDirectiveId)]
    authorized_directives: Vec<AuthorizedDirectiveRecord>,

    // Complexity control stuff
    #[indexed_by(CostDirectiveId)]
    cost_directives: Vec<CostDirectiveRecord>,
    #[indexed_by(ListSizeDirectiveId)]
    list_size_directives: Vec<ListSizeDirectiveRecord>,

    #[indexed_by(ExtensionDirectiveId)]
    extension_directives: Vec<ExtensionDirectiveRecord>,
}

#[derive(serde::Serialize, serde::Deserialize, id_derives::IndexedFields)]
pub struct SubGraphs {
    #[indexed_by(GraphqlEndpointId)]
    graphql_endpoints: Vec<GraphqlEndpointRecord>,
    #[indexed_by(VirtualSubgraphId)]
    virtual_subgraphs: Vec<VirtualSubgraphRecord>,
    pub introspection: introspection::IntrospectionMetadata,
}

impl Schema {
    pub fn walk<T: for<'s> Walk<&'s Self>>(&self, item: T) -> Walker<'_, T> {
        item.walk(self)
    }

    pub fn definitions(&self) -> impl Iter<Item = Definition<'_>> + '_ {
        self.graph.type_definitions_ordered_by_name.walk(self)
    }

    pub fn definition_by_name(&self, name: &str) -> Option<Definition<'_>> {
        self.graph
            .type_definitions_ordered_by_name
            .binary_search_by_key(&name, |definition_id| definition_id.walk(self).name())
            .map(|index| self.graph.type_definitions_ordered_by_name[index])
            .ok()
            .walk(self)
    }

    pub fn default_header_rules(&self) -> impl Iter<Item = HeaderRule<'_>> + '_ {
        self.settings.default_header_rules.walk(self)
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
        (0..self.subgraphs.graphql_endpoints.len()).map(move |i| {
            let id = GraphqlEndpointId::from(i);
            id.walk(self)
        })
    }

    pub fn subgraphs(&self) -> impl Iterator<Item = Subgraph<'_>> + '_ {
        let virt = (0..self.subgraphs.virtual_subgraphs.len()).map(move |i| {
            let id = VirtualSubgraphId::from(i);
            Subgraph::from(id.walk(self))
        });

        self.graphql_endpoints()
            .map(Into::into)
            .chain(virt)
            .chain(std::iter::once(Subgraph::Introspection(self)))
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ScalarType {
    String,
    Float,
    Int,
    BigInt,
    Boolean,
    /// Anything is accepted for this scalar.
    Unknown,
}

impl ScalarType {
    pub fn from_scalar_name(name: &str) -> ScalarType {
        match name {
            "String" | "ID" => ScalarType::String,
            "Float" => ScalarType::Float,
            "Int" => ScalarType::Int,
            "BigInt" => ScalarType::BigInt,
            "Boolean" => ScalarType::Boolean,
            _ => ScalarType::Unknown,
        }
    }
}
