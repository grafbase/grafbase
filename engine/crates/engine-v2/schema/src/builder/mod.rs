mod coerce;
mod error;
mod external_sources;
mod graph;
mod ids;
mod input_values;
mod interner;
mod requires;

use std::mem::take;
use std::time::Duration;

use config::latest::Config;
use sources::graphql::GraphqlEndpointId;
use url::Url;

use self::error::*;
use self::external_sources::ExternalDataSources;
use self::graph::GraphBuilder;
use self::ids::IdMaps;
use self::interner::ProxyKeyInterner;

pub use self::error::BuildError;

use crate::*;
use interner::Interner;
use requires::*;

pub(crate) fn build(mut config: Config, version: Version) -> Result<Schema, BuildError> {
    let mut ctx = BuildContext::new(&mut config);
    let sources = ExternalDataSources::build(&mut ctx, &mut config);
    let (graph, introspection) = GraphBuilder::build(&mut ctx, &sources, &mut config)?;
    let data_sources = DataSources {
        graphql: sources.graphql,
        introspection,
    };
    ctx.finalize(data_sources, graph, config, version)
}

pub(crate) struct BuildContext {
    pub strings: Interner<String, StringId>,
    pub regexps: ProxyKeyInterner<Regex, RegexId>,
    urls: Interner<Url, UrlId>,
    idmaps: IdMaps,
    next_subraph_id: usize,
}

impl BuildContext {
    #[cfg(test)]
    pub fn build_with<T>(build: impl FnOnce(&mut Self, &mut Graph) -> T) -> (Schema, T) {
        use sources::introspection::IntrospectionBuilder;

        use crate::builder::interner::ProxyKeyInterner;

        let mut ctx = Self {
            strings: Interner::from_vec(Vec::new()),
            regexps: ProxyKeyInterner::default(),
            urls: Interner::default(),
            idmaps: IdMaps::empty(),
            next_subraph_id: 0,
        };

        let mut graph = Graph {
            description: None,
            root_operation_types: RootOperationTypes {
                query_id: ObjectDefinitionId::from(0),
                mutation_id: None,
                subscription_id: None,
            },
            type_definitions: Vec::new(),
            object_definitions: vec![ObjectDefinitionRecord {
                name_id: ctx.strings.get_or_new("Query"),
                description_id: None,
                interface_ids: Default::default(),
                directive_ids: Default::default(),
                field_ids: IdRange::from_start_and_length((0, 2)),
            }],
            interface_definitions: Vec::new(),
            field_definitions: vec![
                FieldDefinitionRecord {
                    name_id: ctx.strings.get_or_new("__type"),
                    parent_entity_id: EntityDefinitionId::Object(0.into()),
                    description_id: None,
                    // will be replaced by introspection, doesn't matter.
                    ty: TypeRecord {
                        definition_id: Definition::Object(ObjectDefinitionId::from(0)),
                        wrapping: Default::default(),
                    },
                    resolver_ids: Default::default(),
                    only_resolvable_in_ids: Default::default(),
                    requires: Default::default(),
                    provides: Default::default(),
                    argument_ids: Default::default(),
                    directive_ids: Default::default(),
                },
                FieldDefinitionRecord {
                    name_id: ctx.strings.get_or_new("__schema"),
                    parent_entity_id: EntityDefinitionId::Object(0.into()),
                    description_id: None,
                    // will be replaced by introspection, doesn't matter.
                    ty: TypeRecord {
                        definition_id: Definition::Object(ObjectDefinitionId::from(0)),
                        wrapping: Default::default(),
                    },
                    resolver_ids: Default::default(),
                    only_resolvable_in_ids: Default::default(),
                    requires: Default::default(),
                    provides: Default::default(),
                    argument_ids: Default::default(),
                    directive_ids: Default::default(),
                },
            ],
            enum_definitions: Vec::new(),
            union_definitions: Vec::new(),
            scalar_definitions: Vec::new(),
            input_object_definitions: Vec::new(),
            input_value_definitions: Vec::new(),
            type_system_directives: Vec::new(),
            enum_value_definitions: Vec::new(),
            resolver_definitions: Vec::new(),
            required_field_sets: Vec::new(),
            required_fields: Vec::new(),
            cache_control: Vec::new(),
            input_values: Default::default(),
            required_scopes: Vec::new(),
            authorized_directives: Vec::new(),
        };

        let out = build(&mut ctx, &mut graph);
        let introspection = IntrospectionBuilder::create_data_source_and_insert_fields(&mut ctx, &mut graph);

        let schema = Schema {
            data_sources: DataSources {
                graphql: Default::default(),
                introspection,
            },
            version: Version(Vec::new()),
            graph,
            strings: ctx.strings.into(),
            regexps: Default::default(),
            urls: Default::default(),
            header_rules: Default::default(),
            settings: Default::default(),
        };

        (schema, out)
    }

    fn new(config: &mut Config) -> Self {
        Self {
            strings: Interner::from_vec(take(&mut config.graph.strings)),
            regexps: Default::default(),
            urls: Interner::default(),
            idmaps: IdMaps::new(&config.graph),
            next_subraph_id: 0,
        }
    }

    pub fn next_subgraph_id(&mut self) -> SubgraphId {
        let id = SubgraphId::from(self.next_subraph_id);
        self.next_subraph_id += 1;
        id
    }

    fn finalize(
        mut self,
        data_sources: DataSources,
        graph: Graph,
        mut config: Config,
        version: Version,
    ) -> Result<Schema, BuildError> {
        let header_rules: Vec<_> = take(&mut config.header_rules)
            .into_iter()
            .map(|rule| -> HeaderRuleRecord {
                match rule {
                    config::latest::HeaderRule::Forward(rule) => {
                        let name = match rule.name {
                            config::latest::NameOrPattern::Pattern(regex) => {
                                NameOrPattern::Pattern(self.regexps.get_or_insert(regex))
                            }
                            config::latest::NameOrPattern::Name(name) => {
                                NameOrPattern::Name(self.strings.get_or_new(&config[name]))
                            }
                        };

                        let default = rule.default.map(|id| self.strings.get_or_new(&config[id]));
                        let rename = rule.rename.map(|id| self.strings.get_or_new(&config[id]));

                        HeaderRuleRecord::Forward {
                            name_id: name,
                            default,
                            rename,
                        }
                    }
                    config::latest::HeaderRule::Insert(rule) => {
                        let name = self.strings.get_or_new(&config[rule.name]);
                        let value = self.strings.get_or_new(&config[rule.value]);

                        HeaderRuleRecord::Insert { name_id: name, value }
                    }
                    config::latest::HeaderRule::Remove(rule) => {
                        let name = match rule.name {
                            config::latest::NameOrPattern::Pattern(regex) => {
                                NameOrPattern::Pattern(self.regexps.get_or_insert(regex))
                            }
                            config::latest::NameOrPattern::Name(name) => {
                                NameOrPattern::Name(self.strings.get_or_new(&config[name]))
                            }
                        };

                        HeaderRuleRecord::Remove { name_id: name }
                    }
                    config::latest::HeaderRule::RenameDuplicate(rule) => HeaderRuleRecord::RenameDuplicate {
                        name_id: self.strings.get_or_new(&config[rule.name]),
                        default: rule.default.map(|id| self.strings.get_or_new(&config[id])),
                        rename: self.strings.get_or_new(&config[rule.rename]),
                    },
                }
            })
            .collect();

        let default_header_rules = config
            .default_header_rules
            .into_iter()
            .map(|id| HeaderRuleId::from(id.0))
            .collect();

        Ok(Schema {
            data_sources,
            graph,
            version,
            strings: self.strings.into(),
            regexps: self.regexps.into(),
            urls: self.urls.into(),
            header_rules,
            settings: Settings {
                timeout: config.timeout.unwrap_or(DEFAULT_GATEWAY_TIMEOUT),
                default_header_rules,
                auth_config: take(&mut config.auth),
                operation_limits: take(&mut config.operation_limits),
                disable_introspection: config.disable_introspection,
                retry: config.retry,
            },
        })
    }
}

macro_rules! from_id_newtypes {
    ($($from:ty => $name:ident,)*) => {
        $(
            impl From<$from> for $name {
                fn from(id: $from) -> Self {
                    $name::from(id.0)
                }
            }
        )*
    }
}

// EnumValueId from federated_graph can't be directly
// converted, we sort them by their name.
from_id_newtypes! {
    federated_graph::EnumId => EnumDefinitionId,
    federated_graph::InputObjectId => InputObjectDefinitionId,
    federated_graph::InterfaceId => InterfaceDefinitionId,
    federated_graph::ObjectId => ObjectDefinitionId,
    federated_graph::ScalarId => ScalarDefinitionId,
    federated_graph::StringId => StringId,
    federated_graph::SubgraphId => GraphqlEndpointId,
    federated_graph::UnionId => UnionDefinitionId,
    config::latest::HeaderRuleId => HeaderRuleId,
}

const DEFAULT_GATEWAY_TIMEOUT: Duration = Duration::from_secs(30);
