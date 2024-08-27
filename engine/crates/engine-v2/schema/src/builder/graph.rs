use std::{
    collections::{HashMap, HashSet},
    mem::take,
};

use config::latest::{CacheConfigTarget, Config};
use federated_graph::FederatedGraph;
use id_newtypes::IdRange;
use sources::{
    graphql::{FederationEntityResolverDefinition, FederationKey, GraphqlEndpointId, RootFieldResolverDefinition},
    introspection::IntrospectionBuilder,
    IntrospectionMetadata,
};

use crate::*;

use super::{
    ids::IdMap, interner::Interner, BuildContext, BuildError, ExternalDataSources, RequiredFieldSetBuffer,
    SchemaLocation,
};

pub(crate) struct GraphBuilder<'a> {
    ctx: &'a mut BuildContext,
    sources: &'a ExternalDataSources,
    required_field_sets_buffer: RequiredFieldSetBuffer,
    cache_control: Interner<CacheControl, CacheControlId>,
    required_scopes: Interner<RequiredScopes, RequiredScopesId>,
    graph: Graph,
}

impl<'a> GraphBuilder<'a> {
    pub fn build(
        ctx: &'a mut BuildContext,
        sources: &ExternalDataSources,
        config: &mut Config,
        graph: &mut FederatedGraph,
    ) -> Result<(Graph, IntrospectionMetadata), BuildError> {
        let mut builder = GraphBuilder {
            ctx,
            sources,
            required_field_sets_buffer: Default::default(),
            cache_control: Default::default(),
            required_scopes: Default::default(),
            graph: Graph {
                description: None,
                root_operation_types: RootOperationTypes {
                    query: graph.root_operation_types.query.into(),
                    mutation: graph.root_operation_types.mutation.map(Into::into),
                    subscription: graph.root_operation_types.subscription.map(Into::into),
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
                resolver_definitions: Vec::new(),
                type_definitions: Vec::new(),
                type_system_directives: Vec::new(),
                required_field_sets: Vec::new(),
                required_fields: Vec::new(),
                cache_control: Vec::new(),
                input_values: Default::default(),
                required_scopes: Vec::new(),
                authorized_directives: Vec::new(),
            },
        };
        builder.ingest_config(config, graph);
        builder.finalize()
    }

    fn ingest_config(&mut self, config: &mut Config, graph: &mut FederatedGraph) {
        self.ingest_enums_before_input_values(config, graph);

        self.ingest_input_values(config, graph);
        self.ingest_input_objects(config, graph);
        self.ingest_unions(config, graph);
        self.ingest_scalars(config, graph);
        // Not guaranteed to be sorted and rely on binary search to find the directives for a
        // field.
        graph.object_authorized_directives.sort_unstable_by_key(|(id, _)| *id);
        let object_metadata = self.ingest_objects(config, graph);
        let interface_metadata = self.ingest_interfaces_after_objects(config, graph);
        // Not guaranteed to be sorted and rely on binary search to find the directives for a
        // field.
        graph.field_authorized_directives.sort_unstable_by_key(|(id, _)| *id);
        self.ingest_fields_after_input_values(config, object_metadata, interface_metadata, graph);
    }

    fn ingest_input_values(&mut self, config: &mut Config, graph: &mut FederatedGraph) {
        self.graph.input_value_definitions = take(&mut graph.input_value_definitions)
            .into_iter()
            .enumerate()
            .filter_map(|(idx, definition)| {
                if self.ctx.idmaps.input_value.contains(idx) {
                    Some(InputValueDefinition {
                        name: definition.name.into(),
                        description: definition.description.map(Into::into),
                        ty: definition.r#type.into(),
                        default_value: definition.default.and_then(|default| {
                            let value = self
                                .graph
                                .input_values
                                .ingest_arbitrary_federated_value(self.ctx, default)
                                .ok()?;

                            Some(self.graph.input_values.push_value(value))
                        }),
                        directives: self.push_directives(
                            config,
                            Directives {
                                federated: definition.directives,
                                ..Default::default()
                            },
                            graph,
                        ),
                    })
                } else {
                    None
                }
            })
            .collect();
    }

    fn ingest_input_objects(&mut self, config: &mut Config, graph: &mut FederatedGraph) {
        self.graph.input_object_definitions = take(&mut graph.input_objects)
            .into_iter()
            .enumerate()
            .filter_map(|(idx, definition)| {
                if self.ctx.idmaps.input_value.contains(idx) {
                    Some(InputObjectDefinition {
                        name: definition.name.into(),
                        description: definition.description.map(Into::into),
                        input_field_ids: self.ctx.idmaps.input_value.get_range(definition.fields),
                        directives: self.push_directives(
                            config,
                            Directives {
                                federated: definition.composed_directives,
                                ..Default::default()
                            },
                            graph,
                        ),
                    })
                } else {
                    None
                }
            })
            .collect();
    }

    fn ingest_unions(&mut self, config: &mut Config, graph: &mut FederatedGraph) {
        self.graph.union_definitions = take(&mut graph.unions)
            .into_iter()
            .map(|union| UnionDefinition {
                name: union.name.into(),
                description: None,
                possible_types: union
                    .members
                    .into_iter()
                    .filter(|object_id| !is_inaccessible(graph, graph[*object_id].composed_directives))
                    .map(Into::into)
                    .collect(),
                // Added at the end.
                possible_types_ordered_by_typename: Vec::new(),
                directives: self.push_directives(
                    config,
                    Directives {
                        federated: union.composed_directives,
                        ..Default::default()
                    },
                    graph,
                ),
            })
            .collect();
    }

    fn ingest_enums_before_input_values(&mut self, config: &mut Config, graph: &mut FederatedGraph) {
        self.graph.enum_value_definitions = take(&mut graph.enum_values)
            .into_iter()
            .enumerate()
            .filter_map(|(idx, enum_value)| {
                if is_inaccessible(graph, enum_value.composed_directives) {
                    self.ctx.idmaps.enum_values.skip(federated_graph::EnumValueId(idx));
                    None
                } else {
                    Some(EnumValue {
                        name: enum_value.value.into(),
                        description: None,
                        directives: self.push_directives(
                            config,
                            Directives {
                                federated: enum_value.composed_directives,
                                ..Default::default()
                            },
                            graph,
                        ),
                    })
                }
            })
            .collect();

        self.graph.enum_definitions = take(&mut graph.enums)
            .into_iter()
            .map(|federated_enum| EnumDefinition {
                name: federated_enum.name.into(),
                description: None,
                value_ids: self.ctx.idmaps.enum_values.get_range(federated_enum.values),
                directives: self.push_directives(
                    config,
                    Directives {
                        federated: federated_enum.composed_directives,
                        ..Default::default()
                    },
                    graph,
                ),
            })
            .collect();
    }

    fn ingest_scalars(&mut self, config: &mut Config, graph: &mut FederatedGraph) {
        self.graph.scalar_definitions = take(&mut graph.scalars)
            .into_iter()
            .map(|scalar| {
                let name = StringId::from(scalar.name);
                ScalarDefinition {
                    name,
                    ty: ScalarType::from_scalar_name(&self.ctx.strings[name]),
                    description: None,
                    specified_by_url: None,
                    directives: self.push_directives(
                        config,
                        Directives {
                            federated: scalar.composed_directives,
                            ..Default::default()
                        },
                        graph,
                    ),
                }
            })
            .collect();
    }

    fn ingest_objects(&mut self, config: &mut Config, graph: &mut FederatedGraph) -> ObjectMetadata {
        let mut entities_metadata = ObjectMetadata {
            entities: Default::default(),
            // At most we have as many field as the FederatedGraph
            field_id_to_maybe_object_id: vec![None; graph.fields.len()],
        };

        self.graph.object_definitions = Vec::with_capacity(graph.objects.len());
        for (federated_id, object) in take(&mut graph.objects).into_iter().enumerate() {
            let federated_id = federated_graph::ObjectId(federated_id);
            let object_id = ObjectDefinitionId::from(self.graph.object_definitions.len());

            let fields = self
                .ctx
                .idmaps
                .field
                .get_range((object.fields.start, object.fields.end.0 - object.fields.start.0));

            for field_id in fields {
                entities_metadata.field_id_to_maybe_object_id[usize::from(field_id)] = Some(object_id);
            }

            let schema_location = SchemaLocation::Type {
                name: object.name.into(),
            };
            let directives = self.push_directives(
                config,
                Directives {
                    federated: object.composed_directives,
                    cache_config_target: Some(CacheConfigTarget::Object(federated_id)),
                    authorized_directives: {
                        let mapping = &graph.object_authorized_directives;
                        let mut i = mapping.partition_point(|(id, _)| *id < federated_id);
                        let mut ids = Vec::new();
                        while i < mapping.len() && mapping[i].0 == federated_id {
                            ids.push(mapping[i].1);
                            i += 1
                        }
                        Some((schema_location, ids))
                    },
                },
                graph,
            );
            self.graph.object_definitions.push(ObjectDefinition {
                name: object.name.into(),
                description: None,
                interfaces: object.implements_interfaces.into_iter().map(Into::into).collect(),
                directives,
                fields,
            });

            if let Some(entity) = self.generate_federation_entity_from_keys(schema_location, object.keys) {
                entities_metadata.entities.insert(object_id, entity);
            }
        }

        entities_metadata
    }

    fn ingest_interfaces_after_objects(
        &mut self,
        config: &mut Config,
        graph: &mut FederatedGraph,
    ) -> InterfaceMetadata {
        let mut entities_metadata = InterfaceMetadata {
            entities: Default::default(),
            // At most we have as many field as the FederatedGraph
            field_id_to_maybe_interface_id: vec![None; graph.fields.len()],
        };

        self.graph.interface_definitions = Vec::with_capacity(graph.interfaces.len());
        for interface in take(&mut graph.interfaces) {
            let interface_id = InterfaceDefinitionId::from(self.graph.interface_definitions.len());
            let fields = self.ctx.idmaps.field.get_range((
                interface.fields.start,
                interface.fields.end.0 - interface.fields.start.0,
            ));
            for field_id in fields {
                entities_metadata.field_id_to_maybe_interface_id[usize::from(field_id)] = Some(interface_id);
            }
            let directives = self.push_directives(
                config,
                Directives {
                    federated: interface.composed_directives,
                    ..Default::default()
                },
                graph,
            );
            self.graph.interface_definitions.push(InterfaceDefinition {
                name: interface.name.into(),
                description: None,
                interfaces: interface.implements_interfaces.into_iter().map(Into::into).collect(),
                possible_types: Vec::new(),
                // Added at the end.
                possible_types_ordered_by_typename: Vec::new(),
                directives,
                fields,
            });

            if let Some(entity) = self.generate_federation_entity_from_keys(
                SchemaLocation::Type {
                    name: interface.name.into(),
                },
                interface.keys,
            ) {
                entities_metadata.entities.insert(interface_id, entity);
            }
        }

        // Adding all implementations of an interface, used during introspection.
        for object_id in (0..self.graph.object_definitions.len()).map(ObjectDefinitionId::from) {
            for interface_id in self.graph[object_id].interfaces.clone() {
                self.graph[interface_id].possible_types.push(object_id);
            }
        }

        entities_metadata
    }

    fn ingest_fields_after_input_values(
        &mut self,
        config: &mut Config,
        object_metadata: ObjectMetadata,
        interface_metadata: InterfaceMetadata,
        graph: &mut FederatedGraph,
    ) {
        let root_fields = {
            let mut root_fields = vec![];
            root_fields.extend(self.graph[self.graph.root_operation_types.query].fields);

            if let Some(mutation) = self.graph.root_operation_types.mutation {
                root_fields.extend(self.graph[mutation].fields);
            }
            if let Some(subscription) = self.graph.root_operation_types.subscription {
                root_fields.extend(self.graph[subscription].fields);
            }
            root_fields.sort_unstable();
            root_fields
        };

        let mut root_field_resolvers = HashMap::<GraphqlEndpointId, ResolverDefinitionId>::new();
        for (federated_id, field) in take(&mut graph.fields).into_iter().enumerate() {
            let federated_id = federated_graph::FieldId(federated_id);
            let Some(field_id) = self.ctx.idmaps.field.get(federated_id) else {
                continue;
            };
            let mut resolvers = vec![];
            let mut only_resolvable_in = field
                .resolvable_in
                .into_iter()
                .map(Into::into)
                .collect::<HashSet<GraphqlEndpointId>>();

            // two loops as we can't rely on the ordering of the overrides.
            for r#override in &field.overrides {
                only_resolvable_in.insert(r#override.graph.into());
            }
            for r#override in field.overrides {
                match r#override.from {
                    federated_graph::OverrideSource::Subgraph(id) => {
                        only_resolvable_in.remove(&id.into());
                    }
                    federated_graph::OverrideSource::Missing(_) => (),
                };
            }

            if root_fields.binary_search(&field_id).is_ok() {
                for &endpoint_id in &only_resolvable_in {
                    let resolver_id = *root_field_resolvers.entry(endpoint_id).or_insert_with(|| {
                        self.push_resolver(ResolverDefinition::GraphqlRootField(RootFieldResolverDefinition {
                            endpoint_id,
                        }))
                    });
                    resolvers.push(resolver_id);
                }
            } else if let Some(FederationEntity {
                keys,
                unresolvable_keys,
            }) = object_metadata
                .get_parent_entity(field_id)
                .or_else(|| interface_metadata.get_parent_entity(field_id))
            {
                // FederatedGraph does not include key fields in resolvable_in.
                for (endpoint_id, _, key_field_set) in keys {
                    if key_field_set.contains(field_id) {
                        only_resolvable_in.insert(*endpoint_id);
                    }
                }
                // if resolvable within a federation subgraph and not part of the keys
                // (requirements), we can use the resolver to retrieve this field.
                for (endpoint_id, resolver_id, key_field_set) in keys {
                    if !key_field_set.contains(field_id) && only_resolvable_in.contains(endpoint_id) {
                        resolvers.push(*resolver_id);
                    }
                }

                // if unresolvable within this subgraph, it means we can't provide the entity
                // directly but are able to provide the necessary key fields.
                for (endpoint_id, field_set) in unresolvable_keys {
                    if field_set.contains(field_id) {
                        only_resolvable_in.insert(*endpoint_id);
                    }
                }
            }
            let parent_entity_id = if let Some(object_id) =
                object_metadata.field_id_to_maybe_object_id[usize::from(field_id)]
            {
                EntityId::Object(object_id)
            } else if let Some(interface_id) = interface_metadata.field_id_to_maybe_interface_id[usize::from(field_id)]
            {
                EntityId::Interface(interface_id)
            } else {
                // TODO: better guarantee this never fails.
                unreachable!()
            };
            let schema_location = SchemaLocation::Field {
                ty: match parent_entity_id {
                    EntityId::Object(id) => self.graph[id].name,
                    EntityId::Interface(id) => self.graph[id].name,
                },
                name: field.name.into(),
            };

            let directives = self.push_directives(
                config,
                Directives {
                    federated: field.composed_directives,
                    cache_config_target: Some(CacheConfigTarget::Field(federated_id)),
                    authorized_directives: {
                        let mapping = &graph.field_authorized_directives;
                        let mut i = mapping.partition_point(|(id, _)| *id < federated_id);
                        let mut ids = Vec::new();
                        while i < mapping.len() && mapping[i].0 == federated_id {
                            ids.push(mapping[i].1);
                            i += 1
                        }
                        Some((schema_location, ids))
                    },
                },
                graph,
            );

            self.graph.field_definitions.push(FieldDefinition {
                name: field.name.into(),
                description: None,
                parent_entity: parent_entity_id,
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
                        let field_set_id = self.required_field_sets_buffer.push(schema_location, fields);
                        FieldRequires {
                            subgraph_id: self.sources.graphql[GraphqlEndpointId::from(subgraph_id)].subgraph_id,
                            field_set_id,
                        }
                    })
                    .collect(),
                argument_ids: self.ctx.idmaps.input_value.get_range(field.arguments),
                directives,
            })
        }
    }

    fn finalize(self) -> Result<(Graph, IntrospectionMetadata), BuildError> {
        let Self {
            ctx,
            required_field_sets_buffer,
            cache_control,
            required_scopes,
            mut graph,
            sources: _,
        } = self;

        graph.cache_control = cache_control.into();
        graph.required_scopes = required_scopes.into();
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
        graph.type_definitions = definitions;

        let mut interface_definitions = std::mem::take(&mut graph.interface_definitions);
        for interface in &mut interface_definitions {
            interface.possible_types.sort_unstable();
            interface
                .possible_types_ordered_by_typename
                .clone_from(&interface.possible_types);
            interface
                .possible_types_ordered_by_typename
                .sort_unstable_by_key(|id| &ctx.strings[graph[*id].name]);
        }
        graph.interface_definitions = interface_definitions;

        let mut union_definitions = std::mem::take(&mut graph.union_definitions);
        for union in &mut union_definitions {
            union.possible_types.sort_unstable();
            union
                .possible_types_ordered_by_typename
                .clone_from(&union.possible_types);
            union
                .possible_types_ordered_by_typename
                .sort_unstable_by_key(|id| &ctx.strings[graph[*id].name]);
        }
        graph.union_definitions = union_definitions;

        Ok((graph, introspection))
    }

    fn generate_federation_entity_from_keys(
        &mut self,
        location: SchemaLocation,
        keys: Vec<federated_graph::Key>,
    ) -> Option<FederationEntity> {
        if keys.is_empty() {
            return None;
        }

        let mut entity = FederationEntity::default();

        for key in keys {
            // Some SDL are generated with empty keys, they're useless to us.
            if key.fields.is_empty() {
                continue;
            }

            let endpoint_id = key.subgraph_id.into();
            if key.resolvable {
                let providable = self.ctx.idmaps.field.convert_providable_field_set(&key.fields);
                let key = FederationKey {
                    fields: self.required_field_sets_buffer.push(location, key.fields),
                };

                let resolver_id = self.push_resolver(ResolverDefinition::GraphqlFederationEntity(
                    FederationEntityResolverDefinition { endpoint_id, key },
                ));
                entity.keys.push((endpoint_id, resolver_id, providable));
            } else {
                // We don't need to differentiate between keys here. We'll be using this to add
                // those fields to `provides` in the relevant fields. It's the resolvable keys
                // that will determine which fields to retrieve during planning. And composition
                // ensures that keys between subgraphs are coherent.
                let field_set: ProvidableFieldSet = self.ctx.idmaps.field.convert_providable_field_set(&key.fields);
                entity
                    .unresolvable_keys
                    .entry(endpoint_id)
                    .and_modify(|current| current.update(&field_set))
                    .or_insert(field_set);
            }
        }

        if entity.keys.is_empty() && entity.unresolvable_keys.is_empty() {
            None
        } else {
            Some(entity)
        }
    }

    fn push_resolver(&mut self, resolver: ResolverDefinition) -> ResolverDefinitionId {
        let resolver_id = ResolverDefinitionId::from(self.graph.resolver_definitions.len());
        self.graph.resolver_definitions.push(resolver);
        resolver_id
    }

    fn push_directives(
        &mut self,
        config: &Config,
        directives: Directives,
        graph: &FederatedGraph,
    ) -> IdRange<TypeSystemDirectiveId> {
        let start = self.graph.type_system_directives.len();

        for directive in &graph[directives.federated] {
            let directive = match directive {
                federated_graph::Directive::Authenticated => TypeSystemDirective::Authenticated,
                federated_graph::Directive::RequiresScopes(federated_scopes) => {
                    let id = self.required_scopes.get_or_insert(RequiredScopes::new(
                        federated_scopes
                            .iter()
                            .map(|scopes| scopes.iter().copied().map(Into::into).collect())
                            .collect(),
                    ));
                    TypeSystemDirective::RequiresScopes(id)
                }
                federated_graph::Directive::Deprecated { reason } => {
                    TypeSystemDirective::Deprecated(crate::Deprecated {
                        reason: reason.map(Into::into),
                    })
                }
                federated_graph::Directive::Other { .. }
                | federated_graph::Directive::Inaccessible
                | federated_graph::Directive::Policy(_) => continue,
            };
            self.graph.type_system_directives.push(directive);
        }

        if let Some(config) = directives
            .cache_config_target
            .and_then(|target| config.cache.rule(target))
        {
            let cache_control_id = self.cache_control.get_or_insert(CacheControl {
                max_age: config.max_age,
                stale_while_revalidate: config.stale_while_revalidate,
            });
            self.graph
                .type_system_directives
                .push(TypeSystemDirective::CacheControl(cache_control_id));
        }

        if let Some((schema_location, directives)) = directives.authorized_directives {
            for id in directives {
                let federated_graph::AuthorizedDirective {
                    fields,
                    arguments,
                    metadata,
                    node,
                } = &graph[id];

                self.graph.authorized_directives.push(AuthorizedDirective {
                    arguments: arguments
                        .as_ref()
                        .map(|args| self.convert_input_value_set(args))
                        .unwrap_or_default(),
                    fields: fields
                        .as_ref()
                        .map(|field_set| self.required_field_sets_buffer.push(schema_location, field_set.clone())),
                    node: node
                        .as_ref()
                        .map(|field_set| self.required_field_sets_buffer.push(schema_location, field_set.clone())),
                    metadata: metadata.clone().and_then(|value| {
                        let value = self
                            .graph
                            .input_values
                            .ingest_arbitrary_federated_value(self.ctx, value)
                            .ok()?;

                        Some(self.graph.input_values.push_value(value))
                    }),
                });

                let authorized_id = (self.graph.authorized_directives.len() - 1).into();
                self.graph
                    .type_system_directives
                    .push(TypeSystemDirective::Authorized(authorized_id));
            }
        }

        let end = self.graph.type_system_directives.len();
        (start..end).into()
    }

    fn convert_input_value_set(&self, input_value_set: &federated_graph::InputValueDefinitionSet) -> InputValueSet {
        input_value_set
            .iter()
            .filter_map(|item| {
                self.ctx
                    .idmaps
                    .input_value
                    .get(item.input_value_definition)
                    .map(|id| InputValueSetItem {
                        id,
                        subselection: self.convert_input_value_set(&item.subselection),
                    })
            })
            .collect()
    }
}

struct Directives {
    federated: federated_graph::Directives,
    cache_config_target: Option<CacheConfigTarget>,
    authorized_directives: Option<(SchemaLocation, Vec<federated_graph::AuthorizedDirectiveId>)>,
}

impl Default for Directives {
    fn default() -> Self {
        Self {
            federated: (federated_graph::DirectiveId(0), 0),
            cache_config_target: None,
            authorized_directives: None,
        }
    }
}

struct ObjectMetadata {
    entities: HashMap<ObjectDefinitionId, FederationEntity>,
    field_id_to_maybe_object_id: Vec<Option<ObjectDefinitionId>>,
}

impl ObjectMetadata {
    fn get_parent_entity(&self, id: FieldDefinitionId) -> Option<&FederationEntity> {
        self.field_id_to_maybe_object_id[usize::from(id)].and_then(|id| self.entities.get(&id))
    }
}

struct InterfaceMetadata {
    entities: HashMap<InterfaceDefinitionId, FederationEntity>,
    field_id_to_maybe_interface_id: Vec<Option<InterfaceDefinitionId>>,
}

impl InterfaceMetadata {
    fn get_parent_entity(&self, id: FieldDefinitionId) -> Option<&FederationEntity> {
        self.field_id_to_maybe_interface_id[usize::from(id)].and_then(|id| self.entities.get(&id))
    }
}

#[derive(Default)]
struct FederationEntity {
    keys: Vec<(GraphqlEndpointId, ResolverDefinitionId, ProvidableFieldSet)>,
    unresolvable_keys: HashMap<GraphqlEndpointId, ProvidableFieldSet>,
}

pub(super) fn is_inaccessible(
    graph: &federated_graph::FederatedGraph,
    directives: federated_graph::Directives,
) -> bool {
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
