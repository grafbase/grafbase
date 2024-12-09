mod coerce;
mod error;
mod external_sources;
mod field_set;
mod graph;
mod input_values;
mod interner;

use std::collections::HashMap;
use std::mem::take;
use std::time::Duration;

use config::Config;
use external_sources::ExternalDataSources;
use url::Url;

use self::error::*;
use self::graph::GraphBuilder;
use self::interner::ProxyKeyInterner;

pub use self::error::BuildError;

use crate::*;
use field_set::*;
use interner::Interner;

pub(crate) fn build(mut config: Config, version: Version) -> Result<Schema, BuildError> {
    let mut ctx = BuildContext::new(&mut config);
    let sources = ExternalDataSources::build(&mut ctx, &mut config);
    let (graph, introspection) = GraphBuilder::build(&mut ctx, &sources, &mut config)?;
    let subgraphs = SubGraphs {
        graphql_endpoints: sources.graphql_endpoints,
        introspection,
    };
    ctx.finalize(subgraphs, graph, config, version)
}

pub(crate) struct BuildContext {
    pub strings: Interner<String, StringId>,
    pub regexps: ProxyKeyInterner<Regex, RegexId>,
    urls: Interner<Url, UrlId>,
    scalar_mapping: HashMap<federated_graph::ScalarDefinitionId, ScalarDefinitionId>,
    enum_mapping: HashMap<federated_graph::EnumDefinitionId, EnumDefinitionId>,
}

impl BuildContext {
    fn new(config: &mut Config) -> Self {
        Self {
            strings: Interner::from_vec(take(&mut config.graph.strings)),
            regexps: Default::default(),
            urls: Interner::default(),
            scalar_mapping: HashMap::new(),
            enum_mapping: HashMap::new(),
        }
    }

    fn finalize(
        mut self,
        subgraphs: SubGraphs,
        graph: Graph,
        mut config: Config,
        version: Version,
    ) -> Result<Schema, BuildError> {
        let header_rules: Vec<_> = take(&mut config.header_rules)
            .into_iter()
            .map(|rule| -> HeaderRuleRecord {
                match rule {
                    config::HeaderRule::Forward(rule) => {
                        let name_id = match rule.name {
                            config::NameOrPattern::Pattern(regex) => {
                                NameOrPatternId::Pattern(self.regexps.get_or_insert(regex))
                            }
                            config::NameOrPattern::Name(name) => {
                                NameOrPatternId::Name(self.strings.get_or_new(&config[name]))
                            }
                        };

                        let default_id = rule.default.map(|id| self.strings.get_or_new(&config[id]));
                        let rename_id = rule.rename.map(|id| self.strings.get_or_new(&config[id]));

                        HeaderRuleRecord::Forward(ForwardHeaderRuleRecord {
                            name_id,
                            default_id,
                            rename_id,
                        })
                    }
                    config::HeaderRule::Insert(rule) => {
                        let name_id = self.strings.get_or_new(&config[rule.name]);
                        let value_id = self.strings.get_or_new(&config[rule.value]);

                        HeaderRuleRecord::Insert(InsertHeaderRuleRecord { name_id, value_id })
                    }
                    config::HeaderRule::Remove(rule) => {
                        let name_id = match rule.name {
                            config::NameOrPattern::Pattern(regex) => {
                                NameOrPatternId::Pattern(self.regexps.get_or_insert(regex))
                            }
                            config::NameOrPattern::Name(name) => {
                                NameOrPatternId::Name(self.strings.get_or_new(&config[name]))
                            }
                        };

                        HeaderRuleRecord::Remove(RemoveHeaderRuleRecord { name_id })
                    }
                    config::HeaderRule::RenameDuplicate(rule) => {
                        HeaderRuleRecord::RenameDuplicate(RenameDuplicateHeaderRuleRecord {
                            name_id: self.strings.get_or_new(&config[rule.name]),
                            default_id: rule.default.map(|id| self.strings.get_or_new(&config[id])),
                            rename_id: self.strings.get_or_new(&config[rule.rename]),
                        })
                    }
                }
            })
            .collect();

        let default_header_rules = config
            .default_header_rules
            .into_iter()
            .map(|id| HeaderRuleId::from(id.0))
            .collect();

        Ok(Schema {
            subgraphs,
            graph,
            version,
            strings: self
                .strings
                .into_iter()
                .map(|mut s| {
                    s.shrink_to_fit();
                    s
                })
                .collect(),
            regexps: self.regexps.into(),
            urls: self.urls.into(),
            header_rules,
            settings: Settings {
                timeout: config.timeout.unwrap_or(DEFAULT_GATEWAY_TIMEOUT),
                default_header_rules,
                auth_config: take(&mut config.auth),
                operation_limits: take(&mut config.operation_limits),
                disable_introspection: config.disable_introspection,
                retry: config.retry.map(Into::into),
                batching: take(&mut config.batching),
                complexity_control: take(&mut config.complexity_control),
                response_extension: config.response_extension,
                apq_enabled: config.apq.enabled,
                executable_document_limit_bytes: config.executable_document_limit_bytes,
            },
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
    federated_graph::StringId => StringId,
    federated_graph::SubgraphId => GraphqlEndpointId,
    federated_graph::UnionId => UnionDefinitionId,
    federated_graph::EnumValueId => EnumValueId,
    federated_graph::InputValueDefinitionId => InputValueDefinitionId,
    federated_graph::FieldId => FieldDefinitionId,
    config::HeaderRuleId => HeaderRuleId,
}

impl From<federated_graph::EntityDefinitionId> for EntityDefinitionId {
    fn from(id: federated_graph::EntityDefinitionId) -> Self {
        match id {
            federated_graph::EntityDefinitionId::Object(id) => EntityDefinitionId::Object(id.into()),
            federated_graph::EntityDefinitionId::Interface(id) => EntityDefinitionId::Interface(id.into()),
        }
    }
}

const DEFAULT_GATEWAY_TIMEOUT: Duration = Duration::from_secs(30);
