mod coerce;
mod error;
mod external_sources;
mod graph;
mod ids;
mod interner;
mod requires;

use std::mem::take;

use config::latest::Config;
use url::Url;

use self::external_sources::ExternalDataSources;
use self::graph::GraphBuilder;
use self::ids::IdMaps;
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
    urls: Interner<Url, UrlId>,
    idmaps: IdMaps,
    next_subraph_id: usize,
}

impl BuildContext {
    #[cfg(test)]
    pub fn build_with<T>(build: impl FnOnce(&mut Self, &mut Graph) -> T) -> (Schema, T) {
        let mut ctx = Self {
            strings: Interner::from_vec(Vec::new()),
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
                name: ctx.strings.get_or_insert("Query"),
                description: None,
                interfaces: Default::default(),
                directives: Default::default(),
                fields: IdRange::from_start_and_length((0, 2)),
            }],
            interface_definitions: Vec::new(),
            field_definitions: vec![
                FieldDefinition {
                    name: ctx.strings.get_or_insert("__type"),
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
                    name: ctx.strings.get_or_insert("__schema"),
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
            required_fields_arguments: Vec::new(),
            cache_control: Vec::new(),
            input_values: Default::default(),
            required_scopes: Vec::new(),
        };
        let out = build(&mut ctx, &mut graph);
        let introspection =
            sources::introspection::IntrospectionBuilder::create_data_source_and_insert_fields(&mut ctx, &mut graph);
        let schema = Schema {
            data_sources: DataSources {
                graphql: Default::default(),
                introspection,
            },
            graph,
            strings: ctx.strings.into(),
            urls: Default::default(),
            headers: Default::default(),
            settings: Default::default(),
        };
        (schema, out)
    }

    fn new(config: &mut Config) -> Self {
        Self {
            strings: Interner::from_vec(take(&mut config.graph.strings)),
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
        let headers = take(&mut config.headers)
            .into_iter()
            .map(|header| Header {
                name: self.strings.get_or_insert(&config[header.name]),
                value: match header.value {
                    config::latest::HeaderValue::Forward(id) => {
                        HeaderValue::Forward(self.strings.get_or_insert(&config[id]))
                    }
                    config::latest::HeaderValue::Static(id) => {
                        HeaderValue::Static(self.strings.get_or_insert(&config[id]))
                    }
                },
            })
            .collect();

        Ok(Schema {
            data_sources,
            graph,
            strings: self.strings.into(),
            urls: self.urls.into(),
            headers,
            settings: Settings {
                default_headers: take(&mut config.default_headers).into_iter().map(Into::into).collect(),
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
    config::latest::HeaderId => HeaderId,
}
