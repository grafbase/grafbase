use std::collections::HashMap;

use config::latest::{CacheConfigTarget, Config};

use super::*;
// All of that should be in federated_graph actually.
use super::sources::*;

#[allow(clippy::panic)]
impl From<Config> for Schema {
    fn from(config: Config) -> Self {
        let graph = config.graph;
        let mut schema = Schema {
            description: None,
            root_operation_types: RootOperationTypes {
                query: graph.root_operation_types.query.into(),
                mutation: graph.root_operation_types.mutation.map(Into::into),
                subscription: graph.root_operation_types.subscription.map(Into::into),
            },
            objects: Vec::with_capacity(graph.objects.len()),
            object_fields: Vec::with_capacity(graph.object_fields.len()),
            fields: Vec::with_capacity(graph.fields.len()),
            types: graph.field_types.into_iter().map(Into::into).collect(),
            interfaces: graph.interfaces.into_iter().map(Into::into).collect(),
            interface_fields: graph.interface_fields.into_iter().map(Into::into).collect(),
            enums: graph.enums.into_iter().map(Into::into).collect(),
            unions: graph.unions.into_iter().map(Into::into).collect(),
            scalars: Vec::with_capacity(graph.scalars.len()),
            input_objects: Vec::with_capacity(graph.input_objects.len()),
            headers: convert_headers(config.headers, graph.strings.len()),
            strings: graph.strings,
            resolvers: vec![],
            definitions: vec![],
            input_values: vec![],
            data_sources: DataSources {
                federation: federation::DataSource {
                    subgraphs: graph.subgraphs.into_iter().map(Into::into).collect(),
                },
                ..Default::default()
            },
            default_headers: config.default_headers.into_iter().map(Into::into).collect(),
            cache_configs: vec![],
            auth_config: config.auth,
        };

        schema.strings.extend(config.strings);
        for (id, config) in config.subgraph_configs {
            schema.update_subgraph_config(id, config);
        }

        // -- OBJECTS --
        let mut entity_resolvers = HashMap::<ObjectId, Vec<(ResolverId, SubgraphId)>>::new();
        for object in graph.objects {
            let object_id = ObjectId::from(schema.objects.len());
            let keys = object.resolvable_keys;
            let cache_config = config
                .cache
                .rule(CacheConfigTarget::Object(federated_graph::ObjectId(object_id.into())))
                .map(|config| schema.insert_cache_config(config));

            schema.objects.push(Object {
                name: object.name.into(),
                description: None,
                interfaces: object.implements_interfaces.into_iter().map(Into::into).collect(),
                composed_directives: object.composed_directives.into_iter().map(Into::into).collect(),
                cache_config,
            });

            for key in keys {
                let resolver_id = ResolverId::from(schema.resolvers.len());
                let subgraph_id = key.subgraph_id.into();
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
            }
        }

        // -- OBJECT FIELDS --
        let mut field_entity_resolvers = HashMap::<FieldId, Vec<(ResolverId, SubgraphId)>>::new();
        for object_field in graph.object_fields {
            if let Some(resolvers) = entity_resolvers.get(&object_field.object_id.into()) {
                field_entity_resolvers
                    .entry(object_field.field_id.into())
                    .or_default()
                    .extend(resolvers);
            }
            schema.object_fields.push(object_field.into());
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
        for (i, field) in graph.fields.into_iter().enumerate() {
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
            if let Some(subgraph_id) = field.resolvable_in {
                let subgraph_id = subgraph_id.into();
                if root_fields.binary_search(&field_id).is_ok() {
                    let resolver_id = *root_field_resolvers.entry(subgraph_id).or_insert_with(|| {
                        let resolver_id = ResolverId::from(schema.resolvers.len());
                        schema
                            .resolvers
                            .push(Resolver::FederationRootField(federation::RootFieldResolver {
                                subgraph_id,
                            }));
                        resolver_id
                    });
                    resolvers.push(FieldResolver {
                        resolver_id,
                        requires: FieldSet::default(),
                    });
                } else if let Some(entity_resolvers) = field_entity_resolvers.remove(&field_id) {
                    for (resolver_id, entity_subgraph_id) in entity_resolvers {
                        if entity_subgraph_id == subgraph_id {
                            resolvers.push(FieldResolver {
                                resolver_id,
                                requires: subgraph_requires.get(&entity_subgraph_id).cloned().unwrap_or_default(),
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
                provides: field
                    .provides
                    .into_iter()
                    .find(|provides| Some(provides.subgraph_id) == field.resolvable_in)
                    .map(|provides| provides.fields.into_iter().map(Into::into).collect())
                    .unwrap_or_default(),
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
                cache_config: config
                    .cache
                    .rule(CacheConfigTarget::Field(federated_graph::FieldId(field_id.into())))
                    .map(|config| schema.insert_cache_config(config)),
            };
            schema.fields.push(field);
        }

        // -- INPUT OBJECTS --
        // Separating the input fields into a separate input_value vec with an id. This additional
        // indirection allows data sources to rename fields more easily.
        for input_object in graph.input_objects {
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
        schema.scalars = graph
            .scalars
            .into_iter()
            .map(|scalar| {
                let name = StringId::from(scalar.name);
                Scalar {
                    name,
                    data_type: DataType::from_scalar_name(&schema[name]),
                    description: None,
                    specified_by_url: None,
                    composed_directives: scalar.composed_directives.into_iter().map(Into::into).collect(),
                }
            })
            .collect();

        // -- INTROSPECTION --
        introspection::Introspection::finalize_schema(schema)
    }
}

impl From<federated_graph::Subgraph> for federation::Subgraph {
    fn from(subgraph: federated_graph::Subgraph) -> Self {
        federation::Subgraph {
            name: subgraph.name.into(),
            url: subgraph.url.into(),
            headers: vec![],
        }
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

fn convert_headers(headers: Vec<config::latest::Header>, base_string_index: usize) -> Vec<Header> {
    headers
        .into_iter()
        .map(|header| Header {
            name: (base_string_index + header.name.0).into(),
            value: match header.value {
                config::latest::HeaderValue::Forward(id) => HeaderValue::Forward((base_string_index + id.0).into()),
                config::latest::HeaderValue::Static(id) => HeaderValue::Static((base_string_index + id.0).into()),
            },
        })
        .collect()
}

impl Schema {
    fn update_subgraph_config(&mut self, id: federated_graph::SubgraphId, config: config::latest::SubgraphConfig) {
        let subgraph = &mut self.data_sources.federation[id.into()];
        subgraph.headers = config.headers.into_iter().map(Into::into).collect()
    }

    fn insert_cache_config(&mut self, cache_config: &config::latest::CacheConfig) -> CacheConfigId {
        let new_config: CacheConfig = cache_config.into();

        for (i, existing_config) in self.cache_configs.iter().enumerate() {
            if *existing_config == new_config {
                return CacheConfigId::from(i);
            }
        }

        self.cache_configs.push(new_config);

        CacheConfigId::from(self.cache_configs.len() - 1)
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
