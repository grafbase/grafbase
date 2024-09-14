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
    /// Walks the given item within the context of the schema.
    ///
    /// This method allows traversal of the schema structure using the
    /// `Walk` trait, enabling access to related entities and data.
    ///
    /// # Type Parameters
    ///
    /// - `T`: A type that implements the `Walk` trait, allowing for
    ///   traversal.
    ///
    /// # Parameters
    ///
    /// - `item`: The item to walk through the schema.
    ///
    /// # Returns
    ///
    /// A `Walker` object that can be used to navigate the schema based
    /// on the provided item.
    pub fn walk<T: Walk<Self>>(&self, item: T) -> Walker<'_, T> {
        item.walk(self)
    }

    /// Retrieves an iterator over all defined types within the schema.
    ///
    /// This method allows you to access the various type definitions that are part of the schema,
    /// returning an iterator that can be used to traverse through each `Definition`.
    ///
    /// # Returns
    ///
    /// An iterator that yields `Definition` instances, providing access to the schema's types.
    pub fn definitions(&self) -> impl Iter<Item = Definition<'_>> + '_ {
        self.graph.type_definitions_ordered_by_name.walk(self)
    }

    /// Retrieves the identifier of a type definition by its name.
    ///
    /// This method searches for a type definition within the schema using the provided `name`.
    /// If a matching definition is found, its ID is returned; otherwise, `None` is returned.
    ///
    /// # Parameters
    ///
    /// - `name`: A string slice representing the name of the type definition to search for.
    ///
    /// # Returns
    ///
    /// An `Option<DefinitionId>`, which contains the identifier of the matching definition if found,
    /// or `None` if no definition exists with the specified name.
    pub fn definition_by_name(&self, name: &str) -> Option<DefinitionId> {
        self.graph
            .type_definitions_ordered_by_name
            .binary_search_by_key(&name, |definition| self.definition_name(*definition))
            .map(|index| self.graph.type_definitions_ordered_by_name[index])
            .ok()
    }

    /// Retrieves the identifier of a field within an object by its name.
    ///
    /// This method searches for a field within the specified object definition using the provided `name`.
    /// If a matching field is found, its ID is returned; otherwise, `None` is returned.
    ///
    /// # Parameters
    ///
    /// - `object_id`: The identifier of the object definition to search within.
    /// - `name`: A string slice representing the name of the field to search for.
    ///
    /// # Returns
    ///
    /// An `Option<FieldDefinitionId>`, which contains the identifier of the matching field if found,
    /// or `None` if no field exists with the specified name.
    pub fn object_field_by_name(&self, object_id: ObjectDefinitionId, name: &str) -> Option<FieldDefinitionId> {
        let fields = self[object_id].field_ids;
        self[fields]
            .iter()
            .position(|field| self[field.name_id] == name)
            .map(|pos| FieldDefinitionId::from(usize::from(fields.start) + pos))
    }

    /// Retrieves the identifier of a field within an interface by its name.
    ///
    /// This method searches for a field within the specified interface definition using the provided `name`.
    /// If a matching field is found, its ID is returned; otherwise, `None` is returned.
    ///
    /// # Parameters
    ///
    /// - `interface_id`: The identifier of the interface definition to search within.
    /// - `name`: A string slice representing the name of the field to search for.
    ///
    /// # Returns
    ///
    /// An `Option<FieldDefinitionId>`, which contains the identifier of the matching field if found,
    /// or `None` if no field exists with the specified name.
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

    /// Retrieves an iterator over the default header rules specified in the schema settings.
    ///
    /// This method provides access to all default header rules defined within the schema's settings,
    /// allowing you to iterate through each `HeaderRule` that may be applied.
    ///
    /// # Returns
    ///
    /// An iterator that yields `HeaderRule` instances, providing access to the default header rules.
    pub fn default_header_rules(&self) -> impl Iter<Item = HeaderRule<'_>> + '_ {
        self.settings.default_header_rules.walk(self)
    }

    /// Retrieves the name of the specified definition.
    ///
    /// # Parameters
    ///
    /// - `definition`: The identifier of the definition for which the name is requested.
    ///
    /// # Returns
    ///
    /// A string slice representing the name of the definition.
    fn definition_name(&self, definition: DefinitionId) -> &str {
        definition.walk(self).name()
    }

    /// Retrieves the root query type defined in the schema.
    ///
    /// # Returns
    ///
    /// An `ObjectDefinition` corresponding to the root query type.
    pub fn query(&self) -> ObjectDefinition<'_> {
        self.graph.root_operation_types_record.query_id.walk(self)
    }

    /// Retrieves the root mutation type defined in the schema, if available.
    ///
    /// # Returns
    ///
    /// An `Option<ObjectDefinition>`, which is `Some` if a mutation type
    /// is defined, or `None` if it is not.
    pub fn mutation(&self) -> Option<ObjectDefinition<'_>> {
        self.graph.root_operation_types_record.mutation_id.walk(self)
    }

    /// Retrieves the root subscription type defined in the schema, if available.
    ///
    /// # Returns
    ///
    /// An `Option<ObjectDefinition>`, which is `Some` if a subscription type
    /// is defined, or `None` if it is not.
    pub fn subscription(&self) -> Option<ObjectDefinition<'_>> {
        self.graph.root_operation_types_record.subscription_id.walk(self)
    }

    /// Retrieves an iterator over all GraphQL endpoints defined in the schema.
    ///
    /// This method provides access to all GraphQL endpoints associated with the schema,
    /// allowing you to iterate through each `GraphqlEndpoint` instance.
    ///
    /// # Returns
    ///
    /// An iterator that yields `GraphqlEndpoint` instances, representing the
    /// available endpoints defined within the schema.
    pub fn graphql_endpoints(&self) -> impl ExactSizeIterator<Item = GraphqlEndpoint<'_>> {
        (0..self.subgraphs.graphql_endpoints.len()).map(|i| {
            let id = GraphqlEndpointId::from(i);
            id.walk(self)
        })
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
    /// Creates a `ScalarType` from a given scalar name.
    ///
    /// This function attempts to convert a scalar name into its corresponding `ScalarType`.
    /// If the name matches a known scalar type, the appropriate `ScalarType` variant is returned.
    /// If the name is "ID", it returns `ScalarType::String`, as ID scalars are treated as strings.
    /// If the name does not match any known scalar types, `ScalarType::JSON` is returned by default.
    ///
    /// # Parameters
    ///
    /// - `name`: A string slice representing the name of the scalar.
    ///
    /// # Returns
    ///
    /// A `ScalarType` corresponding to the provided scalar name.
    pub fn from_scalar_name(name: &str) -> ScalarType {
        ScalarType::from_str(name).ok().unwrap_or(match name {
            "ID" => ScalarType::String,
            _ => ScalarType::JSON,
        })
    }
}
