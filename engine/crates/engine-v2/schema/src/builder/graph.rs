use std::{
    collections::{HashMap, HashSet},
    mem::take,
    ops::Range,
};

use config::latest::{CacheConfigTarget, Config};
use id_newtypes::IdRange;

use crate::{
    sources::{self, graphql::GraphqlEndpointId, introspection::IntrospectionBuilder, IntrospectionMetadata},
    CacheConfigId, Definition, Directive, EnumDefinition, EnumDefinitionId, EnumValueDefinition, EnumValueDefinitionId,
    FieldDefinition, FieldDefinitionId, FieldProvides, FieldRequires, Graph, InputObjectDefinition,
    InputObjectDefinitionId, InputValueDefinition, InterfaceDefinition, InterfaceDefinitionId, ObjectDefinition,
    ObjectDefinitionId, ProvidableField, ProvidableFieldSet, Resolver, ResolverId, RootOperationTypes,
    ScalarDefinition, ScalarDefinitionId, ScalarType, StringId, Type, UnionDefinition, UnionDefinitionId,
};

use super::{
    ids::IdMap, interner::Interner, BuildContext, BuildError, ExternalDataSources, RequiredFieldSetBuffer,
    SchemaLocation,
};

pub(crate) struct GraphBuilder<'a> {
    ctx: &'a mut BuildContext,
    sources: &'a ExternalDataSources,
    required_field_sets_buffer: RequiredFieldSetBuffer,
    graph: Graph,
}

impl<'a> GraphBuilder<'a> {
    pub fn build(
        ctx: &'a mut BuildContext,
        sources: &ExternalDataSources,
        config: &mut Config,
    ) -> Result<(Graph, IntrospectionMetadata), BuildError> {
        let mut builder = GraphBuilder {
            ctx,
            sources,
            required_field_sets_buffer: Default::default(),
            graph: Graph {
                description: None,
                root_operation_types: RootOperationTypes {
                    query: config.graph.root_operation_types.query.into(),
                    mutation: config.graph.root_operation_types.mutation.map(Into::into),
                    subscription: config.graph.root_operation_types.subscription.map(Into::into),
                },
                object_definitions: Vec::new(),
                interface_definitions: Vec::new(),
                union_definitions: Vec::new(),
                scalar_definitions: Vec::new(),
                enum_definitions: Vec::new(),
                enum_value_definitions: Vec::new(),
                input_object_definitions: Vec::new(),
                input_value_definitions: Vec::new(),
                field_definitions: Vec::new(),
                resolvers: Vec::new(),
                definitions: Vec::new(),
                directive_definitions: Vec::new(),
                cache_configs: Vec::new(),
                required_field_sets: Vec::new(),
                required_fields_arguments: Vec::new(),
                input_values: Default::default(),
            },
        };
        builder.ingest_config(config);
        builder.finalize()
    }

    fn ingest_config(&mut self, config: &mut Config) {
        self.ingest_input_values(config);
        self.ingest_input_objects(config);
        self.ingest_unions(config);
        self.ingest_enums(config);
        self.ingest_scalars(config);
        self.ingest_object_and_fields(config);
        self.ingest_interfaces_after_objects(config);
        self.ingest_directives_after_all(config);
    }

    fn ingest_input_values(&mut self, config: &mut Config) {
        self.graph.input_value_definitions = take(&mut config.graph.input_value_definitions)
            .into_iter()
            .enumerate()
            .filter_map(|(idx, definition)| {
                if self.ctx.idmaps.input_value.contains(idx) {
                    Some(definition.into())
                } else {
                    None
                }
            })
            .collect();
    }

    fn ingest_input_objects(&mut self, config: &mut Config) {
        self.graph.input_object_definitions = take(&mut config.graph.input_objects)
            .into_iter()
            .enumerate()
            .filter_map(|(idx, definition)| {
                if self.ctx.idmaps.input_value.contains(idx) {
                    Some(InputObjectDefinition {
                        name: definition.name.into(),
                        description: definition.description.map(Into::into),
                        input_field_ids: self.ctx.idmaps.input_value.get_range(definition.fields),
                        composed_directives: IdRange::from_start_and_length(definition.composed_directives),
                    })
                } else {
                    None
                }
            })
            .collect();
    }

    fn ingest_unions(&mut self, config: &mut Config) {
        self.graph.union_definitions = take(&mut config.graph.unions)
            .into_iter()
            .map(|union| UnionDefinition {
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
    }

    fn ingest_enums(&mut self, config: &mut Config) {
        let mut idmap = IdMap::<federated_graph::EnumValueId, EnumValueDefinitionId>::default();
        self.graph.enum_value_definitions = take(&mut config.graph.enum_values)
            .into_iter()
            .enumerate()
            .filter_map(|(idx, enum_value)| {
                if is_inaccessible(&config.graph, enum_value.composed_directives) {
                    idmap.skip(federated_graph::EnumValueId(idx));
                    None
                } else {
                    Some(EnumValueDefinition {
                        name: enum_value.value.into(),
                        description: None,
                        composed_directives: IdRange::from_start_and_length(enum_value.composed_directives),
                    })
                }
            })
            .collect();

        self.graph.enum_definitions = take(&mut config.graph.enums)
            .into_iter()
            .map(|federated_enum| EnumDefinition {
                name: federated_enum.name.into(),
                description: None,
                value_ids: {
                    let range = idmap.get_range(federated_enum.values);
                    self.graph.enum_value_definitions[Range::<usize>::from(range)]
                        .sort_unstable_by(|a, b| self.ctx.strings[a.name].cmp(&self.ctx.strings[b.name]));
                    // The range is still valid even if individual ids don't match anymore.
                    range
                },
                composed_directives: IdRange::from_start_and_length(federated_enum.composed_directives),
            })
            .collect();
    }

    fn ingest_scalars(&mut self, config: &mut Config) {
        self.graph.scalar_definitions = take(&mut config.graph.scalars)
            .into_iter()
            .map(|scalar| {
                let name = StringId::from(scalar.name);
                ScalarDefinition {
                    name,
                    ty: ScalarType::from_scalar_name(&self.ctx.strings[name]),
                    description: None,
                    specified_by_url: None,
                    composed_directives: IdRange::from_start_and_length(scalar.composed_directives),
                }
            })
            .collect();
    }

    fn ingest_object_and_fields(&mut self, config: &mut Config) {
        let schema = &mut self.graph;
        let cache = take(&mut config.cache);
        let graph = &mut config.graph;
        let mut cache_configs = Interner::<config::latest::CacheConfig, CacheConfigId>::default();

        // -- OBJECTS --
        let mut entity_resolvers =
            HashMap::<ObjectDefinitionId, Vec<(ResolverId, GraphqlEndpointId, ProvidableFieldSet)>>::new();
        let mut unresolvable_keys =
            HashMap::<ObjectDefinitionId, HashMap<GraphqlEndpointId, ProvidableFieldSet>>::new();
        let mut field_id_to_maybe_object_id: Vec<Option<ObjectDefinitionId>> = vec![None; graph.fields.len()];

        for object in take(&mut graph.objects) {
            let object_id = ObjectDefinitionId::from(schema.object_definitions.len());
            let cache_config = cache
                .rule(CacheConfigTarget::Object(federated_graph::ObjectId(object_id.into())))
                .map(|config| cache_configs.get_or_insert(config));

            let fields = self
                .ctx
                .idmaps
                .field
                .get_range((object.fields.start, object.fields.end.0 - object.fields.start.0));

            for field_id in fields {
                field_id_to_maybe_object_id[usize::from(field_id)] = Some(object_id);
            }

            schema.object_definitions.push(ObjectDefinition {
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
                    let providable = self.ctx.idmaps.field.convert_providable_field_set(&key.fields);
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
                    let field_set: ProvidableFieldSet = self.ctx.idmaps.field.convert_providable_field_set(&key.fields);
                    unresolvable_keys
                        .entry(object_id)
                        .or_default()
                        .entry(endpoint_id)
                        .and_modify(|current| current.update(&field_set))
                        .or_insert(field_set);
                }
            }
        }

        // -- ROOT FIELDS --
        let root_fields = {
            let mut root_fields = vec![];
            root_fields.extend(schema[schema.root_operation_types.query].fields);

            if let Some(mutation) = schema.root_operation_types.mutation {
                root_fields.extend(schema[mutation].fields);
            }
            if let Some(subscription) = schema.root_operation_types.subscription {
                root_fields.extend(schema[subscription].fields);
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
            let Some(field_id) = self.ctx.idmaps.field.get(federated_graph::FieldId(i)) else {
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
                    .map(|endpoint_id| self.sources.graphql[endpoint_id].subgraph_id)
                    .collect(),
                resolvers,
                provides: field
                    .provides
                    .into_iter()
                    .filter(|provides| !provides.fields.is_empty())
                    .map(|federated_graph::FieldProvides { subgraph_id, fields }| FieldProvides {
                        subgraph_id: self.sources.graphql[GraphqlEndpointId::from(subgraph_id)].subgraph_id,
                        field_set: self.ctx.idmaps.field.convert_providable_field_set(&fields),
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
                            subgraph_id: self.sources.graphql[GraphqlEndpointId::from(subgraph_id)].subgraph_id,
                            field_set_id,
                        }
                    })
                    .collect(),
                argument_ids: self.ctx.idmaps.input_value.get_range(field.arguments),
                composed_directives: IdRange::from_start_and_length(field.composed_directives),
                cache_config: cache
                    .rule(CacheConfigTarget::Field(federated_graph::FieldId(field_id.into())))
                    .map(|config| cache_configs.get_or_insert(config)),
            };
            schema.field_definitions.push(field);
        }

        // -- CACHE CONFIG --
        schema.cache_configs = cache_configs.into_iter().map(Into::into).collect();
    }

    fn ingest_interfaces_after_objects(&mut self, config: &mut Config) {
        self.graph.interface_definitions = take(&mut config.graph.interfaces)
            .into_iter()
            .map(|interface| InterfaceDefinition {
                name: interface.name.into(),
                description: None,
                interfaces: Vec::new(),
                possible_types: Vec::new(),
                composed_directives: IdRange::from_start_and_length(interface.composed_directives),
                fields: self.ctx.idmaps.field.get_range((
                    interface.fields.start,
                    interface.fields.end.0 - interface.fields.start.0,
                )),
            })
            .collect();

        // Adding all implementations of an interface, used during introspection.
        for object_id in (0..self.graph.object_definitions.len()).map(ObjectDefinitionId::from) {
            for interface_id in self.graph[object_id].interfaces.clone() {
                self.graph[interface_id].possible_types.push(object_id);
            }
        }
    }

    fn ingest_directives_after_all(&mut self, config: &mut Config) {
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
        self.graph.directive_definitions = directives;
    }

    fn finalize(self) -> Result<(Graph, IntrospectionMetadata), BuildError> {
        let Self {
            ctx,
            required_field_sets_buffer,
            mut graph,
            ..
        } = self;

        required_field_sets_buffer.try_insert_into(ctx, &mut graph)?;

        let introspection = IntrospectionBuilder::create_data_source_and_insert_fields(ctx, &mut graph);

        let mut definitions = Vec::with_capacity(
            graph.scalar_definitions.len()
                + graph.object_definitions.len()
                + graph.interface_definitions.len()
                + graph.union_definitions.len()
                + graph.enum_definitions.len()
                + graph.input_object_definitions.len(),
        );

        // Adding all definitions for introspection & query binding
        definitions
            .extend((0..graph.scalar_definitions.len()).map(|id| Definition::Scalar(ScalarDefinitionId::from(id))));
        definitions
            .extend((0..graph.object_definitions.len()).map(|id| Definition::Object(ObjectDefinitionId::from(id))));
        definitions.extend(
            (0..graph.interface_definitions.len()).map(|id| Definition::Interface(InterfaceDefinitionId::from(id))),
        );
        definitions.extend((0..graph.union_definitions.len()).map(|id| Definition::Union(UnionDefinitionId::from(id))));
        definitions.extend((0..graph.enum_definitions.len()).map(|id| Definition::Enum(EnumDefinitionId::from(id))));
        definitions.extend(
            (0..graph.input_object_definitions.len())
                .map(|id| Definition::InputObject(InputObjectDefinitionId::from(id))),
        );
        definitions.sort_unstable_by_key(|definition| match *definition {
            Definition::Scalar(id) => &ctx.strings[graph[id].name],
            Definition::Object(id) => &ctx.strings[graph[id].name],
            Definition::Interface(id) => &ctx.strings[graph[id].name],
            Definition::Union(id) => &ctx.strings[graph[id].name],
            Definition::Enum(id) => &ctx.strings[graph[id].name],
            Definition::InputObject(id) => &ctx.strings[graph[id].name],
        });
        graph.definitions = definitions;

        for interface in &mut graph.interface_definitions {
            interface.possible_types.sort_unstable();
        }
        for union in &mut graph.union_definitions {
            union.possible_types.sort_unstable();
        }

        Ok((graph, introspection))
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

impl From<federated_graph::EnumValue> for EnumValueDefinition {
    fn from(enum_value: federated_graph::EnumValue) -> Self {
        EnumValueDefinition {
            name: enum_value.value.into(),
            description: None,
            composed_directives: IdRange::from_start_and_length(enum_value.composed_directives),
        }
    }
}

impl IdMap<federated_graph::FieldId, FieldDefinitionId> {
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
