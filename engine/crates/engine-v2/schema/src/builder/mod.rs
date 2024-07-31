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
use url::Url;

use self::external_sources::ExternalDataSources;
use self::graph::GraphBuilder;
use self::ids::IdMaps;
use self::interner::ProxyKeyInterner;
use self::sources::graphql::GraphqlEndpointId;

use super::*;
use error::*;
use interner::Interner;
use requires::*;

impl TryFrom<Config> for Schema {
    type Error = BuildError;

    fn try_from(mut config: Config) -> Result<Self, Self::Error> {
        let mut ctx = BuildContext::new(&mut config);
        let sources = ExternalDataSources::build(&mut ctx, &mut config);
        let (graph, introspection) = GraphBuilder::build(&mut ctx, &sources, &mut config)?;
        let data_sources = DataSources {
            graphql: sources.graphql,
            introspection,
        };
        ctx.finalize(data_sources, graph, config)
    }
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
        use crate::builder::interner::ProxyKeyInterner;

        use self::sources::introspection::IntrospectionBuilder;

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
                query: ObjectId::from(0),
                mutation: None,
                subscription: None,
            },
            type_definitions: Vec::new(),
            object_definitions: vec![Object {
                name: ctx.strings.get_or_new("Query"),
                description: None,
                interfaces: Default::default(),
                directives: Default::default(),
                fields: IdRange::from_start_and_length((0, 2)),
            }],
            interface_definitions: Vec::new(),
            field_definitions: vec![
                FieldDefinition {
                    name: ctx.strings.get_or_new("__type"),
                    parent_entity: EntityId::Object(0.into()),
                    description: None,
                    // will be replaced by introspection, doesn't matter.
                    ty: Type {
                        inner: Definition::Object(ObjectId::from(0)),
                        wrapping: Default::default(),
                    },
                    resolvers: Default::default(),
                    only_resolvable_in: Default::default(),
                    requires: Default::default(),
                    provides: Default::default(),
                    argument_ids: Default::default(),
                    directives: Default::default(),
                },
                FieldDefinition {
                    name: ctx.strings.get_or_new("__schema"),
                    parent_entity: EntityId::Object(0.into()),
                    description: None,
                    // will be replaced by introspection, doesn't matter.
                    ty: Type {
                        inner: Definition::Object(ObjectId::from(0)),
                        wrapping: Default::default(),
                    },
                    resolvers: Default::default(),
                    only_resolvable_in: Default::default(),
                    requires: Default::default(),
                    provides: Default::default(),
                    argument_ids: Default::default(),
                    directives: Default::default(),
                },
            ],
            enum_definitions: Vec::new(),
            union_definitions: Vec::new(),
            scalar_definitions: Vec::new(),
            input_object_definitions: Vec::new(),
            input_value_definitions: Vec::new(),
            type_system_directives: Vec::new(),
            enum_value_definitions: Vec::new(),
            resolvers: Vec::new(),
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
            idmaps: IdMaps::new(config),
            next_subraph_id: 0,
        }
    }

    pub fn next_subgraph_id(&mut self) -> SubgraphId {
        let id = SubgraphId::from(self.next_subraph_id);
        self.next_subraph_id += 1;
        id
    }

    fn finalize(mut self, data_sources: DataSources, graph: Graph, mut config: Config) -> Result<Schema, BuildError> {
        let header_rules: Vec<_> = take(&mut config.header_rules)
            .into_iter()
            .map(|rule| -> HeaderRule {
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

                        HeaderRule::Forward { name, default, rename }
                    }
                    config::latest::HeaderRule::Insert(rule) => {
                        let name = self.strings.get_or_new(&config[rule.name]);
                        let value = self.strings.get_or_new(&config[rule.value]);

                        HeaderRule::Insert { name, value }
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

                        HeaderRule::Remove { name }
                    }
                    config::latest::HeaderRule::RenameDuplicate(rule) => HeaderRule::RenameDuplicate {
                        name: self.strings.get_or_new(&config[rule.name]),
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
    federated_graph::EnumId => EnumId,
    federated_graph::InputObjectId => InputObjectId,
    federated_graph::InterfaceId => InterfaceId,
    federated_graph::ObjectId => ObjectId,
    federated_graph::ScalarId => ScalarId,
    federated_graph::StringId => StringId,
    federated_graph::SubgraphId => GraphqlEndpointId,
    federated_graph::UnionId => UnionId,
    config::latest::HeaderRuleId => HeaderRuleId,
}

const DEFAULT_GATEWAY_TIMEOUT: Duration = Duration::from_secs(30);
