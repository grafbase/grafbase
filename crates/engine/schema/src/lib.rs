#![deny(clippy::future_not_send, unused_crate_dependencies)]

use builder::Builder;
use grafbase_workspace_hack as _;

mod builder;
mod composite_type;
mod config;
mod definition;
mod directive_site;
mod entity;
mod enum_def;
mod enum_value;
mod extension;
mod field;
mod field_set;
mod generated;
mod guid;
mod ids;
mod injection;
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
mod template;
mod ty;
mod union;

#[cfg(test)]
mod tests;

pub use builder::mutable::MutableSchema;
pub use config::*;
pub use extension::*;
use extension_catalog::ExtensionId;
pub use field::*;
pub use field_set::*;
pub use gateway_config::SubscriptionProtocol;
pub use generated::*;
use id_newtypes::{BitSet, IdRange};
pub use ids::*;
pub use injection::*;
pub use input_value::*;
use regex::Regex;
pub use subgraph::*;
pub use template::*;
use walker::{Iter, Walk};
pub use wrapping::*;

pub type Walker<'a, T> = walker::Walker<'a, T, &'a Schema>;

/// /!\ This is *NOT* backwards-compatible. /!\
/// Only a schema serialized with the exact same version is expected to work. For backwards
/// compatibility use engine-config instead.
#[derive(Clone, serde::Serialize, serde::Deserialize, id_derives::IndexedFields)]
pub struct Schema {
    pub subgraphs: SubGraphs,
    pub graph: Graph,
    // Cryptographic hash of the schema
    pub hash: [u8; 32],

    selections: Selections,

    // Kept for messages
    #[indexed_by(ExtensionId)]
    extensions: Vec<extension_catalog::Id>,

    /// All strings deduplicated.
    #[indexed_by(StringId)]
    strings: Vec<String>,
    #[serde(with = "serde_regex")]
    #[indexed_by(RegexId)]
    regexps: Vec<Regex>,
    #[indexed_by(UrlId)]
    pub(crate) urls: Vec<url::Url>,

    pub config: PartialConfig,
}

impl Schema {
    pub async fn from_sdl_or_panic(sdl: &str) -> Self {
        let mut config: gateway_config::Config = Default::default();
        config.graph.introspection = Some(true);

        Self::builder(sdl).config(&config).build().await.unwrap()
    }

    pub async fn empty() -> Self {
        Self::from_sdl_or_panic("").await
    }

    pub fn builder(sdl: &str) -> Builder<'_> {
        Builder::new(sdl)
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

id_newtypes::forward_with_range! {
    impl Index<SchemaInputValueId, Output = SchemaInputValueRecord> for Schema.graph.input_values,
    impl Index<SchemaInputObjectFieldValueId, Output = (InputValueDefinitionId, SchemaInputValueRecord)> for Schema.graph.input_values,
    impl Index<SchemaInputKeyValueId, Output = (StringId, SchemaInputValueRecord)> for Schema.graph.input_values,
    impl Index<GraphqlSubgraphId, Output = GraphqlSubgraphRecord> for Schema.subgraphs,
    impl Index<VirtualSubgraphId, Output = VirtualSubgraphRecord> for Schema.subgraphs,
    impl Index<HeaderRuleId, Output = HeaderRuleRecord> for Schema.subgraphs,
    impl Index<SchemaFieldId, Output = SchemaFieldRecord> for Schema.selections,
    impl Index<SchemaFieldArgumentId, Output = SchemaFieldArgumentRecord> for Schema.selections,
    impl Index<KeyValueInjectionId, Output = KeyValueInjectionRecord> for Schema.selections,
    impl Index<ValueInjectionId, Output = ValueInjection> for Schema.selections,
    impl Index<ArgumentInjectionId, Output = ArgumentInjectionRecord> for Schema.selections,
    impl Index<ArgumentValueInjectionId, Output = ArgumentValueInjection> for Schema.selections,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, id_derives::IndexedFields)]
pub struct Graph {
    pub description_id: Option<StringId>,
    pub root_operation_types_record: RootOperationTypesRecord,

    inaccessible: Inaccessible,
    interface_has_inaccessible_implementor: BitSet<InterfaceDefinitionId>,
    union_has_inaccessible_member: BitSet<UnionDefinitionId>,

    // All type definitions sorted by their name (actual string)
    type_definitions_ordered_by_name: Vec<TypeDefinitionId>,
    #[indexed_by(ObjectDefinitionId)]
    object_definitions: Vec<ObjectDefinitionRecord>,
    #[indexed_by(InterfaceDefinitionId)]
    interface_definitions: Vec<InterfaceDefinitionRecord>,
    #[indexed_by(FieldDefinitionId)]
    field_definitions: Vec<FieldDefinitionRecord>,
    #[indexed_by(DeriveDefinitionId)]
    derive_definitions: Vec<DeriveDefinitionRecord>,
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
    enum_values: Vec<EnumValueRecord>,

    #[indexed_by(ResolverDefinitionId)]
    resolver_definitions: Vec<ResolverDefinitionRecord>,
    #[indexed_by(LookupResolverDefinitionId)]
    lookup_resolver_definitions: Vec<LookupResolverDefinitionRecord>,

    /// Default input values & directive arguments
    pub input_values: SchemaInputValues,

    // Complexity control stuff
    #[indexed_by(CostDirectiveId)]
    cost_directives: Vec<CostDirectiveRecord>,
    #[indexed_by(ListSizeDirectiveId)]
    list_size_directives: Vec<ListSizeDirectiveRecord>,

    #[indexed_by(ExtensionDirectiveId)]
    extension_directives: Vec<ExtensionDirectiveRecord>,
    #[indexed_by(ExtensionDirectiveArgumentId)]
    extension_directive_arguments: Vec<ExtensionDirectiveArgumentRecord>,

    #[indexed_by(TemplateId)]
    templates: Vec<TemplateRecord>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Inaccessible {
    pub object_definitions: BitSet<ObjectDefinitionId>,
    pub interface_definitions: BitSet<InterfaceDefinitionId>,
    pub field_definitions: BitSet<FieldDefinitionId>,
    pub enum_definitions: BitSet<EnumDefinitionId>,
    pub enum_values: BitSet<EnumValueId>,
    pub union_definitions: BitSet<UnionDefinitionId>,
    pub scalar_definitions: BitSet<ScalarDefinitionId>,
    pub input_object_definitions: BitSet<InputObjectDefinitionId>,
    pub input_value_definitions: BitSet<InputValueDefinitionId>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, id_derives::IndexedFields)]
pub struct SubGraphs {
    #[indexed_by(GraphqlSubgraphId)]
    graphql_endpoints: Vec<GraphqlSubgraphRecord>,
    #[indexed_by(VirtualSubgraphId)]
    virtual_subgraphs: Vec<VirtualSubgraphRecord>,
    pub introspection: introspection::IntrospectionSubgraph,

    default_header_rules: IdRange<HeaderRuleId>,
    /// Headers we might want to send to a subgraph
    #[indexed_by(HeaderRuleId)]
    header_rules: Vec<HeaderRuleRecord>,
}

impl Schema {
    pub fn walk<T: for<'s> Walk<&'s Self>>(&self, item: T) -> Walker<'_, T> {
        item.walk(self)
    }

    pub fn type_definitions(&self) -> impl Iter<Item = TypeDefinition<'_>> + '_ {
        self.graph.type_definitions_ordered_by_name.walk(self)
    }

    pub fn field_definitions(&self) -> impl Iter<Item = FieldDefinition<'_>> + '_ {
        IdRange::<FieldDefinitionId>::from(0..self.graph.field_definitions.len()).walk(self)
    }

    pub fn object_definitions(&self) -> impl Iter<Item = ObjectDefinition<'_>> + '_ {
        IdRange::<ObjectDefinitionId>::from(0..self.graph.object_definitions.len()).walk(self)
    }

    pub fn interface_definitions(&self) -> impl Iter<Item = InterfaceDefinition<'_>> + '_ {
        IdRange::<InterfaceDefinitionId>::from(0..self.graph.interface_definitions.len()).walk(self)
    }

    pub fn type_definition_by_name(&self, name: &str) -> Option<TypeDefinition<'_>> {
        self.graph
            .type_definitions_ordered_by_name
            .binary_search_by_key(&name, |definition_id| definition_id.walk(self).name())
            .map(|index| self.graph.type_definitions_ordered_by_name[index])
            .ok()
            .walk(self)
    }

    pub fn default_header_rules(&self) -> impl Iter<Item = HeaderRule<'_>> + '_ {
        self.subgraphs.default_header_rules.walk(self)
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

    pub fn graphql_endpoints(&self) -> impl ExactSizeIterator<Item = GraphqlSubgraph<'_>> {
        (0..self.subgraphs.graphql_endpoints.len()).map(move |i| {
            let id = GraphqlSubgraphId::from(i);
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

    pub fn resolver_definitions(&self) -> impl Iterator<Item = ResolverDefinition<'_>> + '_ {
        IdRange::<ResolverDefinitionId>::from(0..self.graph.resolver_definitions.len()).walk(self)
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
    Boolean,
    /// Anything is accepted for this scalar.
    /// Treat this as arbitrary JSON
    Unknown,
}

impl ScalarType {
    pub fn from_scalar_name(name: &str) -> ScalarType {
        match name {
            "String" | "ID" => ScalarType::String,
            "Float" => ScalarType::Float,
            "Int" => ScalarType::Int,
            "Boolean" => ScalarType::Boolean,
            _ => ScalarType::Unknown,
        }
    }
}
