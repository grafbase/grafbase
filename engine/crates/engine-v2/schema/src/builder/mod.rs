mod coerce;
mod error;
mod ids;
mod interner;
mod requires;

use std::collections::{HashMap, HashSet};
use std::mem::take;

use config::latest::{CacheConfigTarget, Config};
use url::Url;

use self::ids::IdMaps;

use super::*;
use crate::sources;
use crate::sources::graphql::GraphqlEndpointId;
use crate::sources::introspection::IntrospectionSchemaBuilder;
use error::*;
use interner::Interner;
use requires::*;

impl TryFrom<Config> for Schema {
    type Error = BuildError;

    fn try_from(config: Config) -> Result<Self, Self::Error> {
        SchemaBuilder::build_schema(config)
    }
}

pub(crate) struct SchemaBuilder {
    pub schema: Schema,
    pub strings: Interner<String, StringId>,
    urls: Interner<Url, UrlId>,
    idmaps: IdMaps,
    required_field_sets_buffer: RequiredFieldSetBuffer,
    next_subraph_id: usize,
}

impl SchemaBuilder {
    fn build_schema(mut config: Config) -> Result<Schema, BuildError> {
        let mut builder = Self::initialize(&mut config);
        builder.insert_graphql_datasource(&mut config);
        builder.insert_headers(&mut config);

        builder.insert_enums(&mut config);
        builder.insert_graphql_schema(&mut config);
        builder.insert_directives(&mut config);

        let introspection_subgraph_id = builder.next_subraph_id();
        IntrospectionSchemaBuilder::insert_introspection_fields(&mut builder, introspection_subgraph_id);

        builder.build()
    }

    fn initialize(config: &mut Config) -> Self {
        let mut builder = Self {
            idmaps: IdMaps::default(),
            required_field_sets_buffer: Default::default(),
            next_subraph_id: 0,
            strings: Interner::from_vec(take(&mut config.graph.strings)),
            urls: Interner::default(),
            schema: Schema {
                description: None,
                root_operation_types: RootOperationTypes {
                    query: config.graph.root_operation_types.query.into(),
                    mutation: config.graph.root_operation_types.mutation.map(Into::into),
                    subscription: config.graph.root_operation_types.subscription.map(Into::into),
                },
                objects: Vec::with_capacity(config.graph.objects.len()),
                field_definitions: Vec::with_capacity(config.graph.fields.len()),
                interfaces: Vec::with_capacity(config.graph.interfaces.len()),
                enums: Vec::new(),
                unions: Vec::with_capacity(0),
                scalars: Vec::with_capacity(config.graph.scalars.len()),
                input_objects: Vec::new(),
                directives: Vec::new(),
                input_value_definitions: Vec::new(),
                enum_values: Vec::new(),
                headers: Vec::new(),
                strings: Vec::new(),
                resolvers: Vec::new(),
                definitions: Vec::new(),
                data_sources: DataSources::default(),
                default_headers: Vec::new(),
                cache_configs: Vec::new(),
                auth_config: take(&mut config.auth),
                operation_limits: take(&mut config.operation_limits),
                disable_introspection: config.disable_introspection,
                urls: Vec::new(),
                input_values: SchemaInputValues::default(),
                required_field_sets: Vec::new(),
                required_fields_arguments: Vec::new(),
            },
        };

        for (idx, input_value_definition) in take(&mut config.graph.input_value_definitions).into_iter().enumerate() {
            if is_inaccessible(&config.graph, input_value_definition.directives) {
                builder
                    .idmaps
                    .input_value
                    .skip(federated_graph::InputValueDefinitionId(idx));
            } else {
                builder
                    .schema
                    .input_value_definitions
                    .push(input_value_definition.into());
            }
        }

        for (i, field) in config.graph.fields.iter().enumerate() {
            if is_inaccessible(&config.graph, field.composed_directives) {
                builder.idmaps.field.skip(federated_graph::FieldId(i))
            }
        }

        builder.schema.input_objects = take(&mut config.graph.input_objects)
            .into_iter()
            .map(|input_object| builder.convert_input_object(input_object))
            .collect();

        builder.schema.unions = take(&mut config.graph.unions)
            .into_iter()
            .map(|union| Union {
                name: union.name.into(),
                description: None,
                possible_types: union
                    .members
                    .into_iter()
                    .filter(|object_id| !is_inaccessible(&config.graph, config.graph[*object_id].composed_directives))
                    .map(Into::into)
                    .collect(),
                composed_directives: IdRange::from_start_and_length(union.composed_directives),
            })
            .collect();

        builder
    }

    fn insert_directives(&mut self, config: &mut Config) {
        // FIXME: remove stuff that isn't needed at runtime...
        let mut directives = Vec::with_capacity(config.graph.directives.len());
        for directive in take(&mut config.graph.directives) {
            let directive = match directive {
                federated_graph::Directive::Authenticated => Directive::Authenticated,
                federated_graph::Directive::Policy(args) => Directive::Policy(
                    args.into_iter()
                        .map(|inner| inner.into_iter().map(|string| string.into()).collect())
                        .collect(),
                ),
                federated_graph::Directive::RequiresScopes(args) => Directive::RequiresScopes(
                    args.into_iter()
                        .map(|inner| inner.into_iter().map(|string| string.into()).collect())
                        .collect(),
                ),
                federated_graph::Directive::Inaccessible => Directive::Inaccessible,
                federated_graph::Directive::Deprecated { reason } => Directive::Deprecated {
                    reason: reason.map(Into::into),
                },
                federated_graph::Directive::Other { .. } => Directive::Other,
            };
            directives.push(directive);
        }
        self.schema.directives = directives;
    }

    fn insert_headers(&mut self, config: &mut Config) {
        self.schema.headers = take(&mut config.headers)
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
        self.schema.default_headers = take(&mut config.default_headers).into_iter().map(Into::into).collect();
    }

    fn next_subraph_id(&mut self) -> SubgraphId {
        let id = SubgraphId::from(self.next_subraph_id);
        self.next_subraph_id += 1;
        id
    }

    fn insert_graphql_datasource(&mut self, config: &mut Config) {
        self.schema.data_sources.graphql.endpoints = take(&mut config.graph.subgraphs)
            .into_iter()
            .enumerate()
            .map(|(index, subgraph)| {
                let subgraph_id = self.next_subraph_id();
                let name = subgraph.name.into();
                let url = self
                    .urls
                    .insert(url::Url::parse(&self.strings[subgraph.url.into()]).expect("valid url"));
                match config.subgraph_configs.remove(&federated_graph::SubgraphId(index)) {
                    Some(config::latest::SubgraphConfig { websocket_url, headers }) => {
                        sources::graphql::GraphqlEndpoint {
                            name,
                            subgraph_id,
                            url,
                            websocket_url: websocket_url
                                .map(|url| self.urls.insert(url::Url::parse(&config[url]).expect("valid url"))),
                            headers: headers.into_iter().map(Into::into).collect(),
                        }
                    }

                    None => sources::graphql::GraphqlEndpoint {
                        name,
                        subgraph_id,
                        url,
                        websocket_url: None,
                        headers: Vec::new(),
                    },
                }
            })
            .collect();
    }

    fn insert_enums(&mut self, config: &mut Config) {
        for (idx, enum_value) in take(&mut config.graph.enum_values).into_iter().enumerate() {
            if is_inaccessible(&config.graph, enum_value.composed_directives) {
                self.idmaps.enum_value.skip(federated_graph::EnumValueId(idx))
            } else {
                self.schema.enum_values.push(enum_value.into());
            }
        }
        let mut enums: Vec<Enum> = Vec::with_capacity(config.graph.enums.len());
        for federated_enum in take(&mut config.graph.enums) {
            let r#enum = Enum {
                name: federated_enum.name.into(),
                description: None,
                value_ids: {
                    let range = self.idmaps.enum_value.get_range(federated_enum.values);
                    self.schema[range].sort_unstable_by(|a, b| self.strings[a.name].cmp(&self.strings[b.name]));
                    // The range is still valid even if individual ids don't match anymore.
                    range
                },
                composed_directives: IdRange::from_start_and_length(federated_enum.composed_directives),
            };
            enums.push(r#enum);
        }
        self.schema.enums = enums;
    }

    fn insert_graphql_schema(&mut self, config: &mut Config) {
        let cache = take(&mut config.cache);
        let graph = &mut config.graph;
        let schema = &mut self.schema;
        let mut cache_configs = Interner::<config::latest::CacheConfig, CacheConfigId>::default();

        // -- OBJECTS --
        let mut entity_resolvers = HashMap::<ObjectId, Vec<(ResolverId, GraphqlEndpointId, ProvidableFieldSet)>>::new();
        let mut unresolvable_keys = HashMap::<ObjectId, HashMap<GraphqlEndpointId, ProvidableFieldSet>>::new();
        let mut field_id_to_maybe_object_id: Vec<Option<ObjectId>> = vec![None; graph.fields.len()];

        for object in take(&mut graph.objects) {
            let object_id = ObjectId::from(schema.objects.len());
            let cache_config = cache
                .rule(CacheConfigTarget::Object(federated_graph::ObjectId(object_id.into())))
                .map(|config| cache_configs.get_or_insert(config));

            let fields = self
                .idmaps
                .field
                .get_range((object.fields.start, object.fields.end.0 - object.fields.start.0));

            for field_id in fields {
                field_id_to_maybe_object_id[usize::from(field_id)] = Some(object_id);
            }

            schema.objects.push(Object {
                name: object.name.into(),
                description: None,
                interfaces: object.implements_interfaces.into_iter().map(Into::into).collect(),
                composed_directives: IdRange::from_start_and_length(object.composed_directives),
                cache_config,
                fields,
            });

            for key in object.keys {
                let endpoint_id = key.subgraph_id.into();
                // Some SDL are generated with empty keys, they're useless to us.
                if key.fields.is_empty() {
                    continue;
                }
                if key.resolvable {
                    let providable = self.idmaps.field.convert_providable_field_set(&key.fields);
                    let key = sources::graphql::FederationKey {
                        fields: self.required_field_sets_buffer.push(
                            SchemaLocation::Type {
                                name: object.name.into(),
                            },
                            key.fields,
                        ),
                    };

                    let resolver_id = ResolverId::from(schema.resolvers.len());
                    schema.resolvers.push(Resolver::GraphqlFederationEntity(
                        sources::graphql::FederationEntityResolver { endpoint_id, key },
                    ));
                    entity_resolvers
                        .entry(object_id)
                        .or_default()
                        .push((resolver_id, endpoint_id, providable));
                } else {
                    // We don't need to differentiate between keys here. We'll be using this to add
                    // those fields to `provides` in the relevant fields. It's the resolvable keys
                    // that will determine which fields to retrieve during planning. And composition
                    // ensures that keys between subgraphs are coherent.
                    let field_set: ProvidableFieldSet = self.idmaps.field.convert_providable_field_set(&key.fields);
                    unresolvable_keys
                        .entry(object_id)
                        .or_default()
                        .entry(endpoint_id)
                        .and_modify(|current| current.update(&field_set))
                        .or_insert(field_set);
                }
            }
        }

        // -- OBJECT FIELDS --
        let root_fields = {
            let mut root_fields = vec![];
            let walker = schema.walker();
            for field in walker.walk(schema.root_operation_types.query).fields() {
                root_fields.push(field.item);
            }
            if let Some(mutation) = schema.root_operation_types.mutation {
                for field in walker.walk(mutation).fields() {
                    root_fields.push(field.item);
                }
            }
            if let Some(subscription) = schema.root_operation_types.subscription {
                for field in walker.walk(subscription).fields() {
                    root_fields.push(field.item);
                }
            }
            root_fields.sort_unstable();
            root_fields
        };

        // Yeah it's ugly, conversion should be cleaned up once we got it working I guess.
        // -- FIELDS & RESOLVERS --
        // 1. The federated graph uses "resolvable_in" whenever a field is present in a subgraph.
        //    But for resolvers we only want the "entrypoints", so root fields and later the `@key`
        //    for federation entities.
        // 2. Field arguments are converted to input values. That's how the GraphQL spec defines
        //    them and having an id allows data sources to rename those more easily.
        let mut root_field_resolvers = HashMap::<GraphqlEndpointId, ResolverId>::new();
        for (i, field) in take(&mut graph.fields).into_iter().enumerate() {
            let Some(field_id) = self.idmaps.field.get(federated_graph::FieldId(i)) else {
                continue;
            };
            let mut resolvers = vec![];
            let mut only_resolvable_in = field.resolvable_in.into_iter().map(Into::into).collect::<HashSet<_>>();

            if root_fields.binary_search(&field_id).is_ok() {
                for &endpoint_id in &only_resolvable_in {
                    let resolver_id = *root_field_resolvers.entry(endpoint_id).or_insert_with(|| {
                        let resolver_id = ResolverId::from(schema.resolvers.len());
                        schema
                            .resolvers
                            .push(Resolver::GraphqlRootField(sources::graphql::RootFieldResolver {
                                endpoint_id,
                            }));
                        resolver_id
                    });
                    resolvers.push(resolver_id);
                }
            } else if let Some(parent_object_id) = field_id_to_maybe_object_id[usize::from(field_id)] {
                if let Some(entity_resolvers) = entity_resolvers.get(&parent_object_id) {
                    // FederatedGraph does not include key fields in resolvable_in.
                    for (_, endpoint_id, key_field_set) in entity_resolvers {
                        if key_field_set.contains(field_id) {
                            only_resolvable_in.insert(*endpoint_id);
                        }
                    }
                    // if resolvable within a federation subgraph and not part of the keys
                    // (requirements), we can use the resolver to retrieve this field.
                    for (resolver_id, endpoint_id, key_field_set) in entity_resolvers {
                        if !key_field_set.contains(field_id) && only_resolvable_in.contains(endpoint_id) {
                            resolvers.push(*resolver_id);
                        }
                    }
                }

                // if unresolvable within this subgraph, it means we can't provide the entity
                // directly but are able to provide the necessary key fields.
                if let Some(keys) = unresolvable_keys.get(&parent_object_id) {
                    for (endpoint_id, field_set) in keys {
                        if field_set.contains(field_id) {
                            only_resolvable_in.insert(*endpoint_id);
                        }
                    }
                }
            }

            let field = FieldDefinition {
                name: field.name.into(),
                description: None,
                ty: field.r#type.into(),
                only_resolvable_in: only_resolvable_in
                    .into_iter()
                    .map(|endpoint_id| schema.data_sources.graphql[endpoint_id].subgraph_id)
                    .collect(),
                resolvers,
                provides: field
                    .provides
                    .into_iter()
                    .filter(|provides| !provides.fields.is_empty())
                    .map(|federated_graph::FieldProvides { subgraph_id, fields }| FieldProvides {
                        subgraph_id: schema.data_sources.graphql[GraphqlEndpointId::from(subgraph_id)].subgraph_id,
                        field_set: self.idmaps.field.convert_providable_field_set(&fields),
                    })
                    .collect(),
                requires: field
                    .requires
                    .into_iter()
                    .filter(|requires| !requires.fields.is_empty())
                    .map(|federated_graph::FieldRequires { subgraph_id, fields }| {
                        let parent_object_id = field_id_to_maybe_object_id[usize::from(field_id)];
                        let field_set_id = self.required_field_sets_buffer.push(
                            SchemaLocation::Field {
                                ty: parent_object_id.map(|id| schema[id].name).unwrap_or(field.name.into()),
                                name: field.name.into(),
                            },
                            fields,
                        );
                        FieldRequires {
                            subgraph_id: schema.data_sources.graphql[GraphqlEndpointId::from(subgraph_id)].subgraph_id,
                            field_set_id,
                        }
                    })
                    .collect(),
                argument_ids: self.idmaps.input_value.get_range(field.arguments),
                composed_directives: IdRange::from_start_and_length(field.composed_directives),
                cache_config: cache
                    .rule(CacheConfigTarget::Field(federated_graph::FieldId(field_id.into())))
                    .map(|config| cache_configs.get_or_insert(config)),
            };
            schema.field_definitions.push(field);
        }

        // -- INPUT OBJECTS --
        // Separating the input fields into a separate input_value Vec with an id. This additional
        // indirection allows data sources to rename fields more easily.
        for input_object in take(&mut graph.input_objects) {
            let input_object = InputObject {
                name: input_object.name.into(),
                description: None,
                input_field_ids: self.idmaps.input_value.get_range(input_object.fields),
                composed_directives: IdRange::from_start_and_length(input_object.composed_directives),
            };
            schema.input_objects.push(input_object);
        }

        // -- INTERFACES --
        for interface in take(&mut graph.interfaces) {
            schema.interfaces.push(Interface {
                name: interface.name.into(),
                description: None,
                interfaces: Vec::new(),
                possible_types: Vec::new(),
                composed_directives: IdRange::from_start_and_length(interface.composed_directives),
                fields: self.idmaps.field.get_range((
                    interface.fields.start,
                    interface.fields.end.0 - interface.fields.start.0,
                )),
            })
        }

        // Adding all implementations of an interface, used during introspection.
        for object_id in (0..schema.objects.len()).map(ObjectId::from) {
            for interface_id in schema[object_id].interfaces.clone() {
                schema[interface_id].possible_types.push(object_id);
            }
        }

        // -- SCALARS --
        schema.scalars = take(&mut graph.scalars)
            .into_iter()
            .map(|scalar| {
                let name = StringId::from(scalar.name);
                Scalar {
                    name,
                    ty: ScalarType::from_scalar_name(&self.strings[name]),
                    description: None,
                    specified_by_url: None,
                    composed_directives: IdRange::from_start_and_length(scalar.composed_directives),
                }
            })
            .collect();

        // -- CACHE CONFIG --
        schema.cache_configs = cache_configs.into_iter().map(Into::into).collect();
    }

    fn build(self) -> Result<Schema, BuildError> {
        let SchemaBuilder { mut schema, .. } = self;
        schema.strings = self.strings.into();
        schema.urls = self.urls.into();

        schema.definitions = Vec::with_capacity(
            schema.scalars.len()
                + schema.objects.len()
                + schema.interfaces.len()
                + schema.unions.len()
                + schema.enums.len()
                + schema.input_objects.len(),
        );

        // Adding all definitions for introspection & query binding
        schema
            .definitions
            .extend((0..schema.scalars.len()).map(|id| Definition::Scalar(ScalarId::from(id))));
        schema
            .definitions
            .extend((0..schema.objects.len()).map(|id| Definition::Object(ObjectId::from(id))));
        schema
            .definitions
            .extend((0..schema.interfaces.len()).map(|id| Definition::Interface(InterfaceId::from(id))));
        schema
            .definitions
            .extend((0..schema.unions.len()).map(|id| Definition::Union(UnionId::from(id))));
        schema
            .definitions
            .extend((0..schema.enums.len()).map(|id| Definition::Enum(EnumId::from(id))));
        schema
            .definitions
            .extend((0..schema.input_objects.len()).map(|id| Definition::InputObject(InputObjectId::from(id))));

        let mut definitions = take(&mut schema.definitions);
        definitions.sort_unstable_by_key(|definition| schema.definition_name(*definition));
        schema.definitions = definitions;

        for interface in &mut schema.interfaces {
            interface.possible_types.sort_unstable();
        }
        for union in &mut schema.unions {
            union.possible_types.sort_unstable();
        }

        self.required_field_sets_buffer
            .try_insert_into(&mut schema, &self.idmaps)?;

        Ok(schema)
    }

    fn convert_input_object(&self, value: federated_graph::InputObject) -> InputObject {
        InputObject {
            name: value.name.into(),
            description: value.description.map(Into::into),
            input_field_ids: self.idmaps.input_value.get_range(value.fields),
            composed_directives: IdRange::from_start_and_length(value.composed_directives),
        }
    }
}

impl ids::IdMap<federated_graph::FieldId, FieldDefinitionId> {
    fn convert_providable_field_set(&self, field_set: &federated_graph::FieldSet) -> ProvidableFieldSet {
        field_set
            .iter()
            .filter_map(|item| self.convert_providable_field_set_item(item))
            .collect()
    }

    fn convert_providable_field_set_item(&self, item: &federated_graph::FieldSetItem) -> Option<ProvidableField> {
        Some(ProvidableField {
            id: self.get(item.field)?,
            subselection: self.convert_providable_field_set(&item.subselection),
        })
    }
}

fn is_inaccessible(graph: &federated_graph::FederatedGraphV3, directives: federated_graph::Directives) -> bool {
    graph[directives]
        .iter()
        .any(|directive| matches!(directive, federated_graph::Directive::Inaccessible))
}

impl From<federated_graph::Definition> for Definition {
    fn from(definition: federated_graph::Definition) -> Self {
        match definition {
            federated_graph::Definition::Scalar(id) => Definition::Scalar(id.into()),
            federated_graph::Definition::Object(id) => Definition::Object(id.into()),
            federated_graph::Definition::Interface(id) => Definition::Interface(id.into()),
            federated_graph::Definition::Union(id) => Definition::Union(id.into()),
            federated_graph::Definition::Enum(id) => Definition::Enum(id.into()),
            federated_graph::Definition::InputObject(id) => Definition::InputObject(id.into()),
        }
    }
}

impl From<federated_graph::Type> for Type {
    fn from(field_type: federated_graph::Type) -> Self {
        Type {
            inner: field_type.definition.into(),
            wrapping: field_type.wrapping,
        }
    }
}

impl From<federated_graph::InputValueDefinition> for InputValueDefinition {
    fn from(value: federated_graph::InputValueDefinition) -> Self {
        InputValueDefinition {
            name: value.name.into(),
            description: value.description.map(Into::into),
            ty: value.r#type.into(),
            default_value: None,
        }
    }
}

impl From<federated_graph::EnumValue> for EnumValue {
    fn from(enum_value: federated_graph::EnumValue) -> Self {
        EnumValue {
            name: enum_value.value.into(),
            description: None,
            composed_directives: IdRange::from_start_and_length(enum_value.composed_directives),
        }
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
    federated_graph::DirectiveId => DirectiveId,
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
