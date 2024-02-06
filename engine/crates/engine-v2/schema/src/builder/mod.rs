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
}

impl SchemaBuilder {
    fn build_schema(mut config: Config) -> Schema {
        let mut builder = Self::initialize(&mut config);
        builder.insert_headers(&mut config);
        builder.insert_federation_datasource(&mut config);
        builder.insert_graphql_schema(&mut config);
        IntrospectionSchemaBuilder::insert_introspection_fields(&mut builder);
        builder.build()
    }

    fn initialize(config: &mut Config) -> Self {
        Self {
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
                object_fields: Vec::with_capacity(config.graph.object_fields.len()),
                fields: Vec::with_capacity(config.graph.fields.len()),
                types: take(&mut config.graph.field_types)
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                interfaces: take(&mut config.graph.interfaces).into_iter().map(Into::into).collect(),
                interface_fields: take(&mut config.graph.interface_fields)
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                enums: take(&mut config.graph.enums).into_iter().map(Into::into).collect(),
                unions: take(&mut config.graph.unions).into_iter().map(Into::into).collect(),
                scalars: Vec::with_capacity(config.graph.scalars.len()),
                input_objects: Vec::with_capacity(config.graph.input_objects.len()),
                headers: Vec::new(),
                strings: Vec::new(),
                resolvers: vec![],
                definitions: vec![],
                input_values: vec![],
                data_sources: DataSources::default(),
                default_headers: Vec::new(),
                cache_configs: vec![],
                auth_config: take(&mut config.auth),
                operation_limits: take(&mut config.operation_limits),
                urls: Vec::new(),
            },
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

    fn insert_graphql_schema(&mut self, config: &mut Config) {
        let cache = take(&mut config.cache);
        let graph = &mut config.graph;
        let schema = &mut self.schema;
        let mut cache_configs = Interner::<config::latest::CacheConfig, CacheConfigId>::default();

        // -- OBJECTS --
        let mut entity_resolvers = HashMap::<ObjectId, Vec<(ResolverId, SubgraphId)>>::new();
        let mut unresolvable_keys = HashMap::<ObjectId, HashMap<SubgraphId, FieldSet>>::new();
        for object in take(&mut graph.objects) {
            let object_id = ObjectId::from(schema.objects.len());
            let cache_config = cache
                .rule(CacheConfigTarget::Object(federated_graph::ObjectId(object_id.into())))
                .map(|config| cache_configs.get_or_insert(config));

            schema.objects.push(Object {
                name: object.name.into(),
                description: None,
                interfaces: object.implements_interfaces.into_iter().map(Into::into).collect(),
                composed_directives: object.composed_directives.into_iter().map(Into::into).collect(),
                cache_config,
            });

            for key in object.keys {
                let subgraph_id = key.subgraph_id.into();
                // Some SDL are generated with empty keys, they're useless to us.
                if key.fields.is_empty() {
                    continue;
                }
                if key.resolvable {
                    let resolver_id = ResolverId::from(schema.resolvers.len());
                    schema
                        .resolvers
                        .push(Resolver::FederationEntity(federation::EntityResolver {
                            subgraph_id,
                            key: federation::Key {
                                fields: key.fields.into_iter().map(Into::into).collect(),
                            },
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
                    let field_set: FieldSet = key.fields.into_iter().map(Into::into).collect();
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
        let mut field_id_to_maybe_object_id: Vec<Option<ObjectId>> = vec![None; graph.fields.len()];
        for object_field in take(&mut graph.object_fields) {
            let object_field: ObjectField = object_field.into();
            field_id_to_maybe_object_id[usize::from(object_field.field_id)] = Some(object_field.object_id);
            schema.object_fields.push(object_field);
        }

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
            let field_id = FieldId::from(i);
            let mut resolvers = vec![];
            let subgraph_requires = field
                .requires
                .into_iter()
                .map(|federated_graph::FieldRequires { subgraph_id, fields }| {
                    (
                        SubgraphId::from(subgraph_id),
                        FieldSet::from_iter(fields.into_iter().map(Into::into)),
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
                    let field_set: FieldSet = fields.into_iter().map(Into::into).collect();
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
            if let Definition::Object(object_id) = &schema[TypeId::from(field.field_type_id)].inner {
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
                type_id: field.field_type_id.into(),
                resolvers,
                provides: provides
                    .into_iter()
                    .map(|(subgraph_id, field_set)| FieldProvides::IfResolverGroup {
                        group: ResolverGroup::FederationSubgraph(subgraph_id),
                        field_set,
                    })
                    .collect(),
                arguments: {
                    field
                        .arguments
                        .into_iter()
                        .map(|argument| {
                            let input_value = InputValue {
                                name: argument.name.into(),
                                description: None,
                                type_id: argument.type_id.into(),
                                default_value: None,
                            };
                            schema.input_values.push(input_value);
                            InputValueId::from(schema.input_values.len() - 1)
                        })
                        .collect()
                },
                composed_directives: field.composed_directives.into_iter().map(Into::into).collect(),
                is_deprecated: false,
                deprecation_reason: None,
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
                input_fields: {
                    input_object
                        .fields
                        .into_iter()
                        .map(|field| {
                            let input_value = InputValue {
                                name: field.name.into(),
                                description: None,
                                type_id: field.field_type_id.into(),
                                default_value: None,
                            };
                            schema.input_values.push(input_value);
                            InputValueId::from(schema.input_values.len() - 1)
                        })
                        .collect()
                },
                composed_directives: input_object.composed_directives.into_iter().map(Into::into).collect(),
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
                    data_type: DataType::from_scalar_name(&self.strings[name]),
                    description: None,
                    specified_by_url: None,
                    composed_directives: scalar.composed_directives.into_iter().map(Into::into).collect(),
                }
            })
            .collect();

        // -- CACHE CONFIG --
        schema.cache_configs = cache_configs.into_iter().map(Into::into).collect();
    }

    fn build(self) -> Schema {
        let mut schema = self.schema;
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

        let mut object_fields = take(&mut schema.object_fields);
        object_fields
            .sort_unstable_by_key(|ObjectField { object_id, field_id }| (*object_id, &schema[schema[*field_id].name]));
        schema.object_fields = object_fields;

        let mut interface_fields = take(&mut schema.interface_fields);
        interface_fields.sort_unstable_by_key(|InterfaceField { interface_id, field_id }| {
            (*interface_id, &schema[schema[*field_id].name])
        });
        schema.interface_fields = interface_fields;

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
}

impl From<federated_graph::Object> for Object {
    fn from(object: federated_graph::Object) -> Self {
        Object {
            name: object.name.into(),
            description: None,
            interfaces: object.implements_interfaces.into_iter().map(Into::into).collect(),
            composed_directives: object.composed_directives.into_iter().map(Into::into).collect(),
            cache_config: Default::default(),
        }
    }
}

impl From<federated_graph::Directive> for Directive {
    fn from(directive: federated_graph::Directive) -> Self {
        Directive {
            name: directive.name.into(),
            arguments: directive
                .arguments
                .into_iter()
                .map(|(id, value)| (id.into(), value.into()))
                .collect(),
        }
    }
}

impl From<federated_graph::ObjectField> for ObjectField {
    fn from(object_field: federated_graph::ObjectField) -> Self {
        ObjectField {
            object_id: object_field.object_id.into(),
            field_id: object_field.field_id.into(),
        }
    }
}

impl From<federated_graph::Value> for Value {
    fn from(value: federated_graph::Value) -> Self {
        match value {
            federated_graph::Value::String(s) => Value::String(s.into()),
            federated_graph::Value::Int(i) => Value::Int(i),
            federated_graph::Value::Float(f) => Value::Float(f.into()),
            federated_graph::Value::Boolean(b) => Value::Boolean(b),
            federated_graph::Value::EnumValue(s) => Value::EnumValue(s.into()),
            federated_graph::Value::Object(fields) => Value::Object(
                fields
                    .into_iter()
                    .map(|(id, value)| (id.into(), value.into()))
                    .collect(),
            ),
            federated_graph::Value::List(l) => Value::List(l.into_iter().map(Into::into).collect()),
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

impl From<federated_graph::FieldType> for Type {
    fn from(field_type: federated_graph::FieldType) -> Self {
        Type {
            inner: field_type.kind.into(),
            wrapping: Wrapping {
                inner_is_required: field_type.inner_is_required,
                list_wrapping: field_type.list_wrappers.into_iter().rev().map(Into::into).collect(),
            },
        }
    }
}

impl From<federated_graph::ListWrapper> for ListWrapping {
    fn from(wrapper: federated_graph::ListWrapper) -> Self {
        match wrapper {
            federated_graph::ListWrapper::RequiredList => ListWrapping::RequiredList,
            federated_graph::ListWrapper::NullableList => ListWrapping::NullableList,
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
            composed_directives: interface.composed_directives.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<federated_graph::InterfaceField> for InterfaceField {
    fn from(interface_field: federated_graph::InterfaceField) -> Self {
        InterfaceField {
            interface_id: interface_field.interface_id.into(),
            field_id: interface_field.field_id.into(),
        }
    }
}

impl From<federated_graph::Enum> for Enum {
    fn from(value: federated_graph::Enum) -> Self {
        Enum {
            name: value.name.into(),
            description: None,
            values: value.values.into_iter().map(Into::into).collect(),
            composed_directives: value.composed_directives.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<federated_graph::Union> for Union {
    fn from(union: federated_graph::Union) -> Self {
        Union {
            name: union.name.into(),
            description: None,
            possible_types: union.members.into_iter().map(Into::into).collect(),
            composed_directives: union.composed_directives.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<federated_graph::FieldSetItem> for FieldSetItem {
    fn from(selection: federated_graph::FieldSetItem) -> Self {
        FieldSetItem {
            field_id: selection.field.into(),
            subselection: selection.subselection.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<federated_graph::EnumValue> for EnumValue {
    fn from(enum_value: federated_graph::EnumValue) -> Self {
        EnumValue {
            name: enum_value.value.into(),
            description: None,
            deprecation_reason: None,
            is_deprecated: false,
            composed_directives: enum_value.composed_directives.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<federated_graph::InputObjectField> for InputValue {
    fn from(field: federated_graph::InputObjectField) -> Self {
        InputValue {
            name: field.name.into(),
            description: None,
            type_id: field.field_type_id.into(),
            default_value: None,
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

from_id_newtypes! {
    federated_graph::EnumId => EnumId,
    federated_graph::FieldId => FieldId,
    federated_graph::FieldTypeId => TypeId,
    federated_graph::InputObjectId => InputObjectId,
    federated_graph::InterfaceId => InterfaceId,
    federated_graph::ObjectId => ObjectId,
    federated_graph::ScalarId => ScalarId,
    federated_graph::StringId => StringId,
    federated_graph::SubgraphId => SubgraphId,
    federated_graph::UnionId => UnionId,
    config::latest::HeaderId => HeaderId,
}
