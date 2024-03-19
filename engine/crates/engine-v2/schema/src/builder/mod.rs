mod ids;
mod interner;

use std::collections::{HashMap, HashSet};
use std::mem::take;

use config::latest::{CacheConfigTarget, Config};
use url::Url;

use crate::sources::introspection::IntrospectionSchemaBuilder;

use self::interner::Interner;

use super::sources::*;
use super::*;

impl From<Config> for Schema {
    fn from(config: Config) -> Self {
        SchemaBuilder::build_schema(config)
    }
}

pub(crate) struct SchemaBuilder {
    pub schema: Schema,
    pub strings: Interner<String, StringId>,
    pub urls: Interner<Url, UrlId>,
    field_id_mapper: ids::IdMapper<federated_graph::FieldId, FieldId>,
    input_value_id_mapper: ids::IdMapper<federated_graph::InputValueDefinitionId, InputValueDefinitionId>,
    enum_value_id_mapper: ids::IdMapper<federated_graph::EnumValueId, EnumValueId>,
}

impl SchemaBuilder {
    fn build_schema(mut config: Config) -> Schema {
        let mut builder = Self::initialize(&mut config);
        builder.insert_headers(&mut config);
        builder.insert_federation_datasource(&mut config);
        builder.insert_enums(&mut config);
        builder.insert_graphql_schema(&mut config);
        // has to be last for easier @inaccessible removal
        builder.insert_directives(&mut config);
        IntrospectionSchemaBuilder::insert_introspection_fields(&mut builder);
        builder.build()
    }

    fn initialize(config: &mut Config) -> Self {
        let mut builder = Self {
            field_id_mapper: ids::IdMapper::default(),
            enum_value_id_mapper: ids::IdMapper::default(),
            input_value_id_mapper: ids::IdMapper::default(),
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
                fields: Vec::with_capacity(config.graph.fields.len()),
                interfaces: take(&mut config.graph.interfaces).into_iter().map(Into::into).collect(),
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
            },
        };

        for (idx, input_value_definition) in take(&mut config.graph.input_value_definitions).into_iter().enumerate() {
            if is_inaccessible(&config.graph, input_value_definition.directives) {
                builder
                    .input_value_id_mapper
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
                builder.field_id_mapper.skip(federated_graph::FieldId(i))
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
                federated_graph::Directive::Other { name, arguments } => Directive::Other {
                    name: name.into(),
                    arguments: {
                        let ids = self.schema.input_values.reserve_map(StringId::from(0), arguments.len());
                        for ((key, value), id) in arguments.into_iter().zip(ids) {
                            self.schema.input_values[id] = (key.into(), self.insert_value(value));
                        }
                        ids
                    },
                },
            };
            directives.push(directive);
        }
        self.schema.directives = directives;
    }

    fn insert_value(&mut self, value: federated_graph::Value) -> SchemaInputValue {
        match value {
            federated_graph::Value::String(s) => SchemaInputValue::String(s.into()),
            federated_graph::Value::Int(i) => SchemaInputValue::BigInt(i),
            federated_graph::Value::Float(f) => SchemaInputValue::Float(f),
            federated_graph::Value::Boolean(b) => SchemaInputValue::Boolean(b),
            federated_graph::Value::EnumValue(id) => SchemaInputValue::UnknownEnumValue(id.into()),
            federated_graph::Value::Object(fields) => {
                let ids = self.schema.input_values.reserve_map(StringId::from(0), fields.len());
                for ((key, value), id) in fields.into_vec().into_iter().zip(ids) {
                    self.schema.input_values[id] = (key.into(), self.insert_value(value));
                }
                SchemaInputValue::Map(ids)
            }
            federated_graph::Value::List(l) => {
                let ids = self.schema.input_values.reserve_list(l.len());
                for (value, id) in l.into_vec().into_iter().zip(ids) {
                    self.schema.input_values[id] = self.insert_value(value);
                }
                SchemaInputValue::List(ids)
            }
        }
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

    fn insert_federation_datasource(&mut self, config: &mut Config) {
        self.schema.data_sources.federation.subgraphs = take(&mut config.graph.subgraphs)
            .into_iter()
            .enumerate()
            .map(|(index, subgraph)| {
                let name = subgraph.name.into();
                let url = self
                    .urls
                    .insert(url::Url::parse(&self.strings[subgraph.url.into()]).expect("valid url"));
                match config.subgraph_configs.remove(&federated_graph::SubgraphId(index)) {
                    Some(config::latest::SubgraphConfig { websocket_url, headers }) => federation::Subgraph {
                        name,
                        url,
                        websocket_url: websocket_url
                            .map(|url| self.urls.insert(url::Url::parse(&config[url]).expect("valid url"))),
                        headers: headers.into_iter().map(Into::into).collect(),
                    },

                    None => federation::Subgraph {
                        name,
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
                self.enum_value_id_mapper.skip(federated_graph::EnumValueId(idx))
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
                    let range = self.enum_value_id_mapper.map_range(federated_enum.values);
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
        let input_value_mapper = &mut self.input_value_id_mapper;
        let field_id_mapper = &mut self.field_id_mapper;
        let mut cache_configs = Interner::<config::latest::CacheConfig, CacheConfigId>::default();

        // -- OBJECTS --
        let mut entity_resolvers = HashMap::<ObjectId, Vec<(ResolverId, SubgraphId)>>::new();
        let mut unresolvable_keys = HashMap::<ObjectId, HashMap<SubgraphId, FieldSet>>::new();
        let mut field_id_to_maybe_object_id: Vec<Option<ObjectId>> = vec![None; graph.fields.len()];

        for object in take(&mut graph.objects) {
            let object_id = ObjectId::from(schema.objects.len());
            let cache_config = cache
                .rule(CacheConfigTarget::Object(federated_graph::ObjectId(object_id.into())))
                .map(|config| cache_configs.get_or_insert(config));

            let fields = field_id_mapper.map_range((object.fields.start, object.fields.end.0 - object.fields.start.0));

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
                let subgraph_id = key.subgraph_id.into();
                // Some SDL are generated with empty keys, they're useless to us.
                if key.fields.is_empty() {
                    continue;
                }
                if key.resolvable {
                    let key = federation::Key {
                        fields: key
                            .fields
                            .into_iter()
                            .filter_map(|item| field_id_mapper.convert_field_set_item(item))
                            .collect(),
                    };

                    let resolver_id = ResolverId::from(schema.resolvers.len());
                    schema
                        .resolvers
                        .push(Resolver::FederationEntity(federation::EntityResolver {
                            subgraph_id,
                            key,
                        }));
                    entity_resolvers
                        .entry(object_id)
                        .or_default()
                        .push((resolver_id, subgraph_id));
                } else {
                    // We don't need to differentiate between keys here. We'll be using this to add
                    // those fields to `provides` in the relevant fields. It's the resolvable keys
                    // that will determine which fields to retrieve during planning. And composition
                    // ensures that keys between subgraphs are coherent.
                    let field_set: FieldSet = key
                        .fields
                        .into_iter()
                        .filter_map(|item| field_id_mapper.convert_field_set_item(item))
                        .collect();
                    unresolvable_keys
                        .entry(object_id)
                        .or_default()
                        .entry(subgraph_id)
                        .and_modify(|current| {
                            *current = FieldSet::merge(current, &field_set);
                        })
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
        let mut root_field_resolvers = HashMap::<SubgraphId, ResolverId>::new();
        for (i, field) in take(&mut graph.fields).into_iter().enumerate() {
            let Some(field_id) = field_id_mapper.map(federated_graph::FieldId(i)) else {
                continue;
            };
            let mut resolvers = vec![];
            let subgraph_requires = field
                .requires
                .into_iter()
                .map(|federated_graph::FieldRequires { subgraph_id, fields }| {
                    (
                        SubgraphId::from(subgraph_id),
                        FieldSet::from_iter(
                            fields
                                .into_iter()
                                .filter_map(|item| field_id_mapper.convert_field_set_item(item)),
                        ),
                    )
                })
                .collect::<HashMap<_, _>>();
            let mut resolvable_in = field.resolvable_in.into_iter().map(Into::into).collect::<HashSet<_>>();

            if root_fields.binary_search(&field_id).is_ok() {
                for subgraph_id in &resolvable_in {
                    let resolver_id = *root_field_resolvers.entry(*subgraph_id).or_insert_with(|| {
                        let resolver_id = ResolverId::from(schema.resolvers.len());
                        schema
                            .resolvers
                            .push(Resolver::FederationRootField(federation::RootFieldResolver {
                                subgraph_id: *subgraph_id,
                            }));
                        resolver_id
                    });
                    resolvers.push(FieldResolver {
                        resolver_id,
                        field_requires: FieldSet::default(),
                    });
                }
            }

            let mut provides: HashMap<SubgraphId, FieldSet> = field.provides.into_iter().fold(
                HashMap::new(),
                |mut provides, federated_graph::FieldProvides { subgraph_id, fields }| {
                    let field_set: FieldSet = fields
                        .into_iter()
                        .filter_map(|item| field_id_mapper.convert_field_set_item(item))
                        .collect();
                    provides
                        .entry(subgraph_id.into())
                        .and_modify(|current| {
                            *current = FieldSet::merge(current, &field_set);
                        })
                        .or_insert(field_set);

                    provides
                },
            );
            // Whether the field returns an object
            if let Definition::Object(object_id) = &field.r#type.definition.into() {
                if let Some(keys) = unresolvable_keys.get(object_id) {
                    for (subgraph_id, field_set) in keys {
                        provides
                            .entry(*subgraph_id)
                            .and_modify(|current| {
                                *current = FieldSet::merge(current, field_set);
                            })
                            .or_insert_with(|| field_set.clone());
                    }
                }
            }
            // Whether the field is attached to an object (rather than an interface)
            if let Some(object_id) = field_id_to_maybe_object_id[usize::from(field_id)] {
                if let Some(entity_resolvers) = entity_resolvers.get(&object_id) {
                    for (resolver_id, entity_subgraph_id) in entity_resolvers {
                        // Keys aren't in 'resolvable_in', so adding them
                        if let Resolver::FederationEntity(resolver) = &schema[*resolver_id] {
                            if let Some(item) = resolver.key.fields.get(field_id) {
                                resolvable_in.insert(*entity_subgraph_id);
                                provides
                                    .entry(*entity_subgraph_id)
                                    .and_modify(|current| {
                                        *current = FieldSet::merge(current, &item.subselection);
                                    })
                                    .or_insert_with(|| item.subselection.clone());
                            }
                        }
                    }
                    for (resolver_id, entity_subgraph_id) in entity_resolvers {
                        if resolvable_in.contains(entity_subgraph_id) {
                            resolvers.push(FieldResolver {
                                resolver_id: *resolver_id,
                                field_requires: subgraph_requires.get(entity_subgraph_id).cloned().unwrap_or_default(),
                            });
                        }
                    }
                }
            }

            let field = Field {
                name: field.name.into(),
                description: None,
                type_id,
                resolvers,
                provides: provides
                    .into_iter()
                    .map(|(subgraph_id, field_set)| FieldProvides::IfResolverGroup {
                        group: ResolverGroup::FederationSubgraph(subgraph_id),
                        field_set,
                    })
                    .collect(),
                argument_ids: input_value_mapper.map_range(field.arguments),
                composed_directives: IdRange::from_start_and_length(field.composed_directives),
                cache_config: cache
                    .rule(CacheConfigTarget::Field(federated_graph::FieldId(field_id.into())))
                    .map(|config| cache_configs.get_or_insert(config)),
            };
            schema.fields.push(field);
        }

        // -- INPUT OBJECTS --
        // Separating the input fields into a separate input_value vec with an id. This additional
        // indirection allows data sources to rename fields more easily.
        for input_object in take(&mut graph.input_objects) {
            let input_object = InputObject {
                name: input_object.name.into(),
                description: None,
                input_field_ids: input_value_mapper.map_range(input_object.fields),
                composed_directives: IdRange::from_start_and_length(input_object.composed_directives),
            };
            schema.input_objects.push(input_object);
        }

        // -- INTERFACES --
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

    fn build(self) -> Schema {
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

        assert!(matches!(schema.resolvers.last(), Some(Resolver::Introspection(_))));
        schema
    }

    fn convert_input_object(&self, value: federated_graph::InputObject) -> InputObject {
        InputObject {
            name: value.name.into(),
            description: value.description.map(Into::into),
            input_field_ids: self.input_value_id_mapper.map_range(value.fields),
            composed_directives: IdRange::from_start_and_length(value.composed_directives),
        }
    }
}

impl ids::IdMapper<federated_graph::FieldId, FieldId> {
    fn convert_field_set_item(&self, selection: federated_graph::FieldSetItem) -> Option<FieldSetItem> {
        Some(FieldSetItem {
            field_id: self.map(selection.field)?,
            subselection: selection
                .subselection
                .into_iter()
                .filter_map(|item| self.convert_field_set_item(item))
                .collect(),
        })
    }
}

fn is_inaccessible(graph: &federated_graph::FederatedGraphV3, directives: federated_graph::Directives) -> bool {
    graph[directives]
        .iter()
        .any(|directive| matches!(directive, federated_graph::Directive::Inaccessible))
}

impl From<federated_graph::Object> for Object {
    fn from(object: federated_graph::Object) -> Self {
        Object {
            name: object.name.into(),
            description: None,
            interfaces: object.implements_interfaces.into_iter().map(Into::into).collect(),
            composed_directives: IdRange::from_start_and_length(object.composed_directives),
            cache_config: Default::default(),
        }
    }
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

impl From<federated_graph::Interface> for Interface {
    fn from(interface: federated_graph::Interface) -> Self {
        Interface {
            name: interface.name.into(),
            description: None,
            interfaces: vec![],
            possible_types: vec![],
            composed_directives: IdRange::from_start_and_length(interface.composed_directives),
        }
    }
}

impl From<federated_graph::InputValueDefinition> for InputValueDefinition {
    fn from(value: federated_graph::InputValueDefinition) -> Self {
        InputValueDefinition {
            name: value.name.into(),
            description: value.description.map(Into::into),
            type_id: value.type_id.into(),
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
    federated_graph::SubgraphId => SubgraphId,
    federated_graph::UnionId => UnionId,
    config::latest::HeaderId => HeaderId,
}
