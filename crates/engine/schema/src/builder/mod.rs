mod coerce;
mod error;
mod external_sources;
mod field_set;
mod graph;
mod input_values;
mod interner;

use extension_catalog::ExtensionCatalog;
use external_sources::ExternalDataSources;
use fxhash::FxHashMap;
use url::Url;

use self::error::*;
use self::graph::GraphBuilder;
use self::interner::ProxyKeyInterner;

pub use self::error::BuildError;

use crate::*;
use field_set::*;
use interner::Interner;

pub(crate) async fn build(
    config: &gateway_config::Config,
    mut federated_graph: federated_graph::FederatedGraph,
    extension_catalog: &ExtensionCatalog,
    version: Version,
) -> Result<Schema, BuildError> {
    let mut ctx = BuildContext::new(config, extension_catalog, &federated_graph);
    let mut sources = ExternalDataSources::build(&mut ctx, &mut federated_graph)?;
    let (graph, introspection) = GraphBuilder::build(&mut ctx, &mut sources, federated_graph).await?;
    let subgraphs = SubGraphs {
        graphql_endpoints: sources.graphql_endpoints,
        virtual_subgraphs: sources.virtual_subgraphs,
        introspection,
    };
    ctx.finalize(subgraphs, graph, config, version)
}

pub(crate) struct BuildContext<'a> {
    pub config: &'a gateway_config::Config,
    pub extension_catalog: &'a ExtensionCatalog,
    pub strings: Interner<String, StringId>,
    pub regexps: ProxyKeyInterner<Regex, RegexId>,
    urls: Interner<Url, UrlId>,
    header_rules: Vec<HeaderRuleRecord>,
    scalar_mapping: FxHashMap<federated_graph::ScalarDefinitionId, ScalarDefinitionId>,
    enum_mapping: FxHashMap<federated_graph::EnumDefinitionId, EnumDefinitionId>,
}

impl std::ops::Index<StringId> for BuildContext<'_> {
    type Output = String;
    fn index(&self, id: StringId) -> &String {
        self.strings.get_by_id(id).unwrap()
    }
}

impl std::ops::Index<RegexId> for BuildContext<'_> {
    type Output = Regex;
    fn index(&self, index: RegexId) -> &Regex {
        &self.regexps[index]
    }
}

impl std::ops::Index<UrlId> for BuildContext<'_> {
    type Output = Url;
    fn index(&self, id: UrlId) -> &Url {
        self.urls.get_by_id(id).unwrap()
    }
}

impl<'a> BuildContext<'a> {
    fn new(
        config: &'a gateway_config::Config,
        extension_catalog: &'a ExtensionCatalog,
        federated_graph: &federated_graph::FederatedGraph,
    ) -> Self {
        Self {
            config,
            extension_catalog,
            strings: Interner::with_capacity(federated_graph.strings.len()),
            regexps: Default::default(),
            urls: Interner::with_capacity(federated_graph.subgraphs.len()),
            scalar_mapping: FxHashMap::with_capacity_and_hasher(
                federated_graph.scalar_definitions.len(),
                Default::default(),
            ),
            enum_mapping: FxHashMap::with_capacity_and_hasher(
                federated_graph.scalar_definitions.len(),
                Default::default(),
            ),
            header_rules: Vec::new(),
        }
    }

    fn ingest_header_rules(&mut self, rules: &[gateway_config::HeaderRule]) -> IdRange<HeaderRuleId> {
        use gateway_config::*;
        let start = self.header_rules.len();
        self.header_rules.extend(rules.iter().map(|rule| -> HeaderRuleRecord {
            match rule {
                HeaderRule::Forward(rule) => {
                    let name_id = match &rule.name {
                        NameOrPattern::Pattern(regex) => {
                            NameOrPatternId::Pattern(self.regexps.get_or_insert(regex.clone()))
                        }
                        NameOrPattern::Name(name) => NameOrPatternId::Name(self.strings.get_or_new(name.as_ref())),
                    };

                    let default_id = rule.default.as_ref().map(|s| self.strings.get_or_new(s.as_ref()));
                    let rename_id = rule.rename.as_ref().map(|s| self.strings.get_or_new(s.as_ref()));

                    HeaderRuleRecord::Forward(ForwardHeaderRuleRecord {
                        name_id,
                        default_id,
                        rename_id,
                    })
                }
                HeaderRule::Insert(rule) => {
                    let name_id = self.strings.get_or_new(rule.name.as_ref());
                    let value_id = self.strings.get_or_new(rule.value.as_ref());

                    HeaderRuleRecord::Insert(InsertHeaderRuleRecord { name_id, value_id })
                }
                HeaderRule::Remove(rule) => {
                    let name_id = match &rule.name {
                        NameOrPattern::Pattern(regex) => {
                            NameOrPatternId::Pattern(self.regexps.get_or_insert(regex.clone()))
                        }
                        NameOrPattern::Name(name) => NameOrPatternId::Name(self.strings.get_or_new(name.as_ref())),
                    };

                    HeaderRuleRecord::Remove(RemoveHeaderRuleRecord { name_id })
                }
                HeaderRule::RenameDuplicate(rule) => {
                    HeaderRuleRecord::RenameDuplicate(RenameDuplicateHeaderRuleRecord {
                        name_id: self.strings.get_or_new(rule.name.as_ref()),
                        default_id: rule
                            .default
                            .as_ref()
                            .map(|default| self.strings.get_or_new(default.as_ref())),
                        rename_id: self.strings.get_or_new(rule.rename.as_ref()),
                    })
                }
            }
        }));
        (start..self.header_rules.len()).into()
    }

    fn finalize(
        mut self,
        subgraphs: SubGraphs,
        graph: Graph,
        config: &gateway_config::Config,
        version: Version,
    ) -> Result<Schema, BuildError> {
        let default_header_rules = self.ingest_header_rules(&config.headers);

        let auth_config = config
            .authentication
            .as_ref()
            .map(|auth| AuthConfig::new(auth, self.extension_catalog));

        let response_extension = config
            .telemetry
            .exporters
            .response_extension
            .clone()
            .unwrap_or_default()
            .into();

        let executable_document_limit_bytes = config
            .executable_document_limit
            .bytes()
            .try_into()
            .expect("executable document limit should not be negative");

        let settings = PartialConfig {
            timeout: config.gateway.timeout,
            default_header_rules,
            auth_config,
            operation_limits: config.operation_limits.unwrap_or_default(),
            disable_introspection: !config.graph.introspection.unwrap_or_default(),
            retry: config.gateway.retry.enabled.then_some(config.gateway.retry.into()),
            batching: config.gateway.batching.clone(),
            complexity_control: (&config.complexity_control).into(),
            response_extension,
            apq_enabled: config.apq.enabled,
            executable_document_limit_bytes,
            trusted_documents: config.trusted_documents.clone().into(),
            websocket_forward_connection_init_payload: config.websockets.forward_connection_init_payload,
        };

        let strings = self
            .strings
            .into_iter()
            .map(|mut s| {
                s.shrink_to_fit();
                s
            })
            .collect();

        Ok(Schema {
            subgraphs,
            graph,
            version,
            strings,
            regexps: self.regexps.into(),
            urls: self.urls.into(),
            header_rules: self.header_rules,
            settings,
        })
    }

    fn convert_type(&self, federated_graph::Type { wrapping, definition }: federated_graph::Type) -> TypeRecord {
        TypeRecord {
            definition_id: self.convert_definition(definition),
            wrapping,
        }
    }

    fn convert_definition(&self, definition: federated_graph::Definition) -> DefinitionId {
        match definition {
            federated_graph::Definition::Scalar(id) => DefinitionId::Scalar(self.scalar_mapping[&id]),
            federated_graph::Definition::Object(id) => DefinitionId::Object(id.into()),
            federated_graph::Definition::Interface(id) => DefinitionId::Interface(id.into()),
            federated_graph::Definition::Union(id) => DefinitionId::Union(id.into()),
            federated_graph::Definition::Enum(id) => DefinitionId::Enum(self.enum_mapping[&id]),
            federated_graph::Definition::InputObject(id) => DefinitionId::InputObject(id.into()),
        }
    }
}

macro_rules! from_id_newtypes {
    ($($from:ty => $name:ident,)*) => {
        $(
            impl From<$from> for $name {
                fn from(id: $from) -> Self {
                    $name::from(usize::from(id))
                }
            }
        )*
    }
}

// EnumValueId from federated_graph can't be directly
// converted, we sort them by their name.
from_id_newtypes! {
    federated_graph::InputObjectId => InputObjectDefinitionId,
    federated_graph::InterfaceId => InterfaceDefinitionId,
    federated_graph::ObjectId => ObjectDefinitionId,
    federated_graph::UnionId => UnionDefinitionId,
    federated_graph::EnumValueId => EnumValueId,
    federated_graph::InputValueDefinitionId => InputValueDefinitionId,
    federated_graph::FieldId => FieldDefinitionId,
}

impl From<federated_graph::EntityDefinitionId> for EntityDefinitionId {
    fn from(id: federated_graph::EntityDefinitionId) -> Self {
        match id {
            federated_graph::EntityDefinitionId::Object(id) => EntityDefinitionId::Object(id.into()),
            federated_graph::EntityDefinitionId::Interface(id) => EntityDefinitionId::Interface(id.into()),
        }
    }
}
