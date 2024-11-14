use std::{
    collections::{BTreeSet, HashMap},
    mem::take,
};

use builder::coerce::InputValueCoercer;
use config::Config;
use introspection::{IntrospectionBuilder, IntrospectionMetadata};

use crate::*;

use super::{interner::Interner, BuildContext, BuildError, FieldSetsBuilder, SchemaLocation};

pub(crate) struct GraphBuilder<'a> {
    ctx: &'a mut BuildContext,
    field_sets: FieldSetsBuilder,
    required_scopes: Interner<RequiresScopesDirectiveRecord, RequiresScopesDirectiveId>,
    graph: Graph,
}

impl<'a> GraphBuilder<'a> {
    pub fn build(ctx: &'a mut BuildContext, config: &mut Config) -> Result<(Graph, IntrospectionMetadata), BuildError> {
        let mut builder = GraphBuilder {
            ctx,
            field_sets: Default::default(),
            required_scopes: Default::default(),
            graph: Graph {
                description_id: None,
                root_operation_types_record: RootOperationTypesRecord {
                    query_id: config.graph.root_operation_types.query.into(),
                    mutation_id: config.graph.root_operation_types.mutation.map(Into::into),
                    subscription_id: config.graph.root_operation_types.subscription.map(Into::into),
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
                type_definitions_ordered_by_name: Vec::new(),
                fields: Vec::new(),
                input_values: Default::default(),
                required_scopes: Vec::new(),
                authorized_directives: Vec::new(),
                field_sets: Vec::new(),
                field_arguments: Vec::new(),
            },
        };
        builder.ingest_config(config)?;
        builder.finalize()
    }

    fn ingest_config(&mut self, config: &mut Config) -> Result<(), BuildError> {
        self.ingest_enums_before_input_values(config);

        self.ingest_scalars(config);
        self.ingest_input_objects(config);
        self.ingest_input_values_after_scalars_and_input_objects_and_enums(config)?;

        // Not guaranteed to be sorted and rely on binary search to find the directives for a
        // field.
        config
            .graph
            .object_authorized_directives
            .sort_unstable_by_key(|(id, _)| *id);

        let object_metadata = self.ingest_objects(config);
        let interface_metadata = self.ingest_interfaces_after_objects(config);
        self.ingest_unions_after_objects(config);

        // Not guaranteed to be sorted and rely on binary search to find the directives for a
        // field.
        config
            .graph
            .field_authorized_directives
            .sort_unstable_by_key(|(id, _)| *id);

        self.ingest_fields_after_input_values(config, object_metadata, interface_metadata);

        Ok(())
    }

    fn ingest_input_values_after_scalars_and_input_objects_and_enums(
        &mut self,
        config: &mut Config,
    ) -> Result<(), BuildError> {
        // Arbitrary initial capacity, to make it at least proportional to the input_values count.
        let mut default_values = Vec::with_capacity(config.graph.input_value_definitions.len() / 20);
        let mut input_value_definitions = Vec::new();
        for (idx, definition) in take(&mut config.graph.input_value_definitions).into_iter().enumerate() {
            if !self.ctx.idmaps.input_value.contains(idx) {
                continue;
            }
            if let Some(value) = definition.default {
                default_values.push((input_value_definitions.len(), value));
            }
            input_value_definitions.push(InputValueDefinitionRecord {
                name_id: definition.name.into(),
                description_id: definition.description.map(Into::into),
                ty_record: self.ctx.convert_type(definition.r#type),
                // Adding after ingesting all input values as input object fields are input values.
                // So we need them for coercion.
                default_value_id: None,
                directive_ids: self.push_directives(
                    config,
                    Directives {
                        federated: definition.directives,
                        ..Default::default()
                    },
                ),
            });
        }
        self.graph.input_value_definitions = input_value_definitions;

        let mut input_values = take(&mut self.graph.input_values);
        let mut coercer = InputValueCoercer::new(self.ctx, &self.graph, &mut input_values);

        let default_values = default_values
            .into_iter()
            .map(|(idx, value)| {
                let input_value_definition = &self.graph.input_value_definitions[idx];
                let value = coercer.coerce(input_value_definition.ty_record, value).map_err(|err| {
                    BuildError::DefaultValueCoercionError {
                        err,
                        name: self.ctx.strings[input_value_definition.name_id].to_string(),
                    }
                })?;
                Ok((idx, value))
            })
            .collect::<Result<Vec<_>, BuildError>>()?;

        for (idx, value_id) in default_values {
            self.graph.input_value_definitions[idx].default_value_id = Some(value_id);
        }

        self.graph.input_values = input_values;

        Ok(())
    }

    fn ingest_input_objects(&mut self, config: &mut Config) {
        self.graph.input_object_definitions = take(&mut config.graph.input_objects)
            .into_iter()
            .enumerate()
            .filter_map(|(idx, definition)| {
                if self.ctx.idmaps.input_value.contains(idx) {
                    Some(InputObjectDefinitionRecord {
                        name_id: definition.name.into(),
                        description_id: definition.description.map(Into::into),
                        input_field_ids: self.ctx.idmaps.input_value.get_range(definition.fields),
                        directive_ids: self.push_directives(
                            config,
                            Directives {
                                federated: definition.composed_directives,
                                ..Default::default()
                            },
                        ),
                    })
                } else {
                    None
                }
            })
            .collect();
    }

    fn ingest_unions_after_objects(&mut self, config: &mut Config) {
        for union in take(&mut config.graph.unions) {
            let possible_type_ids = union
                .members
                .into_iter()
                // FIXME: fix inaccessible union
                // .filter(|object_id| {
                //     let composed_directives = config
                //         .graph
                //         .at(*object_id)
                //         .then(|obj| obj.type_definition_id)
                //         .directives;
                //
                //     !is_inaccessible(&config.graph, composed_directives)
                // })
                .map(ObjectDefinitionId::from)
                .collect::<Vec<_>>();

            let directive_ids = self.push_directives(
                config,
                Directives {
                    federated: union.composed_directives,
                    ..Default::default()
                },
            );

            let mut join_member_records: Vec<_> = union
                .join_members
                .into_iter()
                .map(|(subgraph_id, member_id)| JoinMemberDefinitionRecord {
                    subgraph_id: SubgraphId::GraphqlEndpoint(subgraph_id.into()),
                    member_id: member_id.into(),
                })
                .collect();

            join_member_records.sort_by_key(|record| (record.subgraph_id, record.member_id));
            let mut not_fully_implemented_in_ids = BTreeSet::new();
            for object_id in &possible_type_ids {
                let object = &self.graph[*object_id];

                // Check in which subgraphs these are resolved.
                for subgraph_id in &object.only_resolvable_in_ids {
                    // The object implements the interface if it defines az `@join__implements`
                    // corresponding to the interface and to the subgraph.
                    if join_member_records
                        .binary_search_by(|probe| {
                            probe.subgraph_id.cmp(subgraph_id).then(probe.member_id.cmp(object_id))
                        })
                        .is_err()
                    {
                        not_fully_implemented_in_ids.insert(*subgraph_id);
                    }
                }
            }

            let union_definition = UnionDefinitionRecord {
                name_id: union.name.into(),
                description_id: None,
                possible_type_ids,
                // Added at the end.
                possible_types_ordered_by_typename_ids: Vec::new(),
                directive_ids,
                join_member_records,
                not_fully_implemented_in_ids: not_fully_implemented_in_ids.into_iter().collect(),
            };

            self.graph.union_definitions.push(union_definition);
        }
    }

    fn ingest_enums_before_input_values(&mut self, config: &mut Config) {
        self.graph.enum_value_definitions = config
            .graph
            .enum_values
            .iter()
            .enumerate()
            .filter_map(|(idx, enum_value)| {
                if is_inaccessible(&config.graph, enum_value.composed_directives) {
                    self.ctx
                        .idmaps
                        .enum_values
                        .skip(federated_graph::EnumValueId::from(idx));
                    None
                } else {
                    Some(EnumValueRecord {
                        name_id: enum_value.value.into(),
                        description_id: None,
                        directive_ids: self.push_directives(
                            config,
                            Directives {
                                federated: enum_value.composed_directives,
                                ..Default::default()
                            },
                        ),
                    })
                }
            })
            .collect();

        self.graph.enum_definitions = config
            .graph
            .iter_enums()
            .map(|federated_enum| EnumDefinitionRecord {
                name_id: federated_enum.name.into(),
                description_id: None,
                value_ids: self
                    .ctx
                    .idmaps
                    .enum_values
                    .get_range(config.graph.enum_value_range(federated_enum.id())),
                directive_ids: self.push_directives(
                    config,
                    Directives {
                        federated: federated_enum.directives,
                        ..Default::default()
                    },
                ),
            })
            .collect();
    }

    fn ingest_scalars(&mut self, config: &mut Config) {
        self.graph.scalar_definitions = config
            .graph
            .iter_scalars()
            .map(|scalar| {
                let name = StringId::from(scalar.name);
                ScalarDefinitionRecord {
                    name_id: name,
                    ty: ScalarType::from_scalar_name(&self.ctx.strings[name]),
                    description_id: None,
                    specified_by_url_id: None,
                    directive_ids: self.push_directives(
                        config,
                        Directives {
                            federated: scalar.directives,
                            ..Default::default()
                        },
                    ),
                }
            })
            .collect();
    }

    fn ingest_objects(&mut self, config: &mut Config) -> ObjectMetadata {
        let mut entities_metadata = ObjectMetadata {
            entities: Default::default(),
            // At most we have as many field as the FederatedGraph
            field_id_to_maybe_object_id: vec![None; config.graph.fields.len()],
        };

        self.graph.object_definitions = Vec::with_capacity(config.graph.objects.len());
        for (federated_id, object) in take(&mut config.graph.objects).into_iter().enumerate() {
            let federated_id = federated_graph::ObjectId::from(federated_id);
            let definition = config.graph.at(object.type_definition_id);
            let object_id = ObjectDefinitionId::from(self.graph.object_definitions.len());

            let fields = self.ctx.idmaps.field.get_range((
                object.fields.start,
                usize::from(object.fields.end) - usize::from(object.fields.start),
            ));

            for field_id in fields {
                entities_metadata.field_id_to_maybe_object_id[usize::from(field_id)] = Some(object_id);
            }

            let schema_location = SchemaLocation::Type {
                name: config.graph.view(object.type_definition_id).name.into(),
            };

            let directives = self.push_directives(
                config,
                Directives {
                    federated: definition.directives,
                    authorized_directives: {
                        let mapping = &config.graph.object_authorized_directives;
                        let mut i = mapping.partition_point(|(id, _)| *id < federated_id);
                        let mut ids = Vec::new();
                        while i < mapping.len() && mapping[i].0 == federated_id {
                            ids.push(mapping[i].1);
                            i += 1
                        }
                        Some((schema_location, ids))
                    },
                },
            );

            let mut join_implement_records: Vec<_> = object
                .join_implements
                .into_iter()
                .map(|(subgraph_id, interface_id)| JoinImplementsDefinitionRecord {
                    subgraph_id: SubgraphId::GraphqlEndpoint(subgraph_id.into()),
                    interface_id: interface_id.into(),
                })
                .collect();

            join_implement_records.sort_by_key(|record| (record.subgraph_id, record.interface_id));

            let mut only_resolvable_in_ids = object
                .keys
                .iter()
                .map(|key| SubgraphId::GraphqlEndpoint(key.subgraph_id.into()))
                .collect::<Vec<_>>();

            only_resolvable_in_ids.sort();

            self.graph.object_definitions.push(ObjectDefinitionRecord {
                name_id: config.graph.view(object.type_definition_id).name.into(),
                description_id: None,
                interface_ids: object.implements_interfaces.into_iter().map(Into::into).collect(),
                directive_ids: directives,
                field_ids: fields,
                join_implement_records,
                only_resolvable_in_ids,
            });

            if let Some(entity) = self.generate_federation_entity_from_keys(schema_location, object.keys) {
                entities_metadata.entities.insert(object_id, entity);
            }
        }

        entities_metadata
    }

    fn ingest_interfaces_after_objects(&mut self, config: &mut Config) -> InterfaceMetadata {
        let mut entities_metadata = InterfaceMetadata {
            entities: Default::default(),
            // At most we have as many field as the FederatedGraph
            field_id_to_maybe_interface_id: vec![None; config.graph.fields.len()],
        };

        self.graph.interface_definitions = Vec::with_capacity(config.graph.interfaces.len());
        for interface in take(&mut config.graph.interfaces) {
            let interface_id = InterfaceDefinitionId::from(self.graph.interface_definitions.len());
            let name_id = config.graph.view(interface.type_definition_id).name.into();
            let definition = config.graph.at(interface.type_definition_id);

            let fields = self.ctx.idmaps.field.get_range((
                interface.fields.start,
                usize::from(interface.fields.end) - usize::from(interface.fields.start),
            ));

            for field_id in fields {
                entities_metadata.field_id_to_maybe_interface_id[usize::from(field_id)] = Some(interface_id);
            }

            let directives = self.push_directives(
                config,
                Directives {
                    federated: definition.directives,
                    ..Default::default()
                },
            );

            self.graph.interface_definitions.push(InterfaceDefinitionRecord {
                name_id,
                description_id: None,
                interface_ids: interface.implements_interfaces.into_iter().map(Into::into).collect(),
                possible_type_ids: Vec::new(),
                // Added at the end.
                possible_types_ordered_by_typename_ids: Vec::new(),
                directive_ids: directives,
                field_ids: fields,
                // Added at the end.
                not_fully_implemented_in_ids: Vec::new(),
            });

            if let Some(entity) =
                self.generate_federation_entity_from_keys(SchemaLocation::Type { name: name_id }, interface.keys)
            {
                entities_metadata.entities.insert(interface_id, entity);
            }
        }

        // Adding all implementations of an interface, used during introspection.
        for object_id in (0..self.graph.object_definitions.len()).map(ObjectDefinitionId::from) {
            for interface_id in self.graph[object_id].interface_ids.clone() {
                self.graph[interface_id].possible_type_ids.push(object_id);
            }
        }

        // Adding all not fully implemented interfaces per subgraph.
        for interface_id in (0..self.graph.interface_definitions.len()).map(InterfaceDefinitionId::from) {
            let mut not_fully_implemented_in = BTreeSet::<SubgraphId>::new();

            // For every possible type implementing this interface.
            for object_id in &self.graph[interface_id].possible_type_ids {
                let object = &self.graph[*object_id];

                // Check in which subgraphs these are resolved.
                for subgraph_id in &object.only_resolvable_in_ids {
                    // The object implements the interface if it defines az `@join__implements`
                    // corresponding to the interface and to the subgraph.
                    if object.implements_interface_in_subgraph(subgraph_id, &interface_id) {
                        continue;
                    }

                    not_fully_implemented_in.insert(*subgraph_id);
                }
            }

            // Sorted by the subgraph id, hence the btree.
            self.graph[interface_id].not_fully_implemented_in_ids = not_fully_implemented_in.into_iter().collect();
        }

        entities_metadata
    }

    fn ingest_fields_after_input_values(
        &mut self,
        config: &mut Config,
        object_metadata: ObjectMetadata,
        interface_metadata: InterfaceMetadata,
    ) {
        let root_fields = {
            let mut root_fields = vec![];
            root_fields.extend(self.graph[self.graph.root_operation_types_record.query_id].field_ids);

            if let Some(mutation) = self.graph.root_operation_types_record.mutation_id {
                root_fields.extend(self.graph[mutation].field_ids);
            }
            if let Some(subscription) = self.graph.root_operation_types_record.subscription_id {
                root_fields.extend(self.graph[subscription].field_ids);
            }
            root_fields.sort_unstable();
            root_fields
        };

        let mut root_field_resolvers = HashMap::<GraphqlEndpointId, ResolverDefinitionId>::new();
        for (federated_id, field) in take(&mut config.graph.fields).into_iter().enumerate() {
            let federated_id = federated_graph::FieldId::from(federated_id);
            let Some(field_id) = self.ctx.idmaps.field.get(federated_id) else {
                continue;
            };
            let mut resolver_ids = vec![];
            let mut only_resolvable_in = field
                .resolvable_in
                .into_iter()
                .map(Into::into)
                .collect::<BTreeSet<GraphqlEndpointId>>();

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
                        self.push_resolver(ResolverDefinitionRecord::GraphqlRootField(
                            GraphqlRootFieldResolverDefinitionRecord { endpoint_id },
                        ))
                    });
                    resolver_ids.push(resolver_id);
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
                    if field_set_contains(key_field_set, federated_id) {
                        only_resolvable_in.insert(*endpoint_id);
                    }
                }
                // if resolvable within a federation subgraph and not part of the keys
                // (requirements), we can use the resolver to retrieve this field.
                for (endpoint_id, resolver_id, key_field_set) in keys {
                    if !field_set_contains(key_field_set, federated_id) && only_resolvable_in.contains(endpoint_id) {
                        resolver_ids.push(*resolver_id);
                    }
                }

                // if unresolvable within this subgraph, it means we can't provide the entity
                // directly but are able to provide the necessary key fields.
                for (endpoint_id, key_field_sets) in unresolvable_keys {
                    if key_field_sets
                        .iter()
                        .any(|key_field_set| field_set_contains(key_field_set, federated_id))
                    {
                        only_resolvable_in.insert(*endpoint_id);
                    }
                }
            }
            let parent_entity_id = if let Some(object_id) =
                object_metadata.field_id_to_maybe_object_id[usize::from(field_id)]
            {
                EntityDefinitionId::Object(object_id)
            } else if let Some(interface_id) = interface_metadata.field_id_to_maybe_interface_id[usize::from(field_id)]
            {
                EntityDefinitionId::Interface(interface_id)
            } else {
                // TODO: better guarantee this never fails.
                unreachable!()
            };
            let schema_location = SchemaLocation::Field {
                ty: match parent_entity_id {
                    EntityDefinitionId::Object(id) => self.graph[id].name_id,
                    EntityDefinitionId::Interface(id) => self.graph[id].name_id,
                },
                name: field.name.into(),
            };

            let directives = self.push_directives(
                config,
                Directives {
                    federated: field.composed_directives,
                    authorized_directives: {
                        let mapping = &config.graph.field_authorized_directives;
                        let mut i = mapping.partition_point(|(id, _)| *id < federated_id);
                        let mut ids = Vec::new();
                        while i < mapping.len() && mapping[i].0 == federated_id {
                            ids.push(mapping[i].1);
                            i += 1
                        }
                        Some((schema_location, ids))
                    },
                },
            );

            self.graph.field_definitions.push(FieldDefinitionRecord {
                name_id: field.name.into(),
                description_id: None,
                parent_entity_id,
                ty_record: self.ctx.convert_type(field.r#type),
                only_resolvable_in_ids: only_resolvable_in
                    .into_iter()
                    .map(SubgraphId::GraphqlEndpoint)
                    .collect(),
                resolver_ids,
                provides_records: field
                    .provides
                    .into_iter()
                    .filter(|provides| !provides.fields.is_empty())
                    .map(|federated_graph::FieldProvides { subgraph_id, fields }| {
                        let field_set_id = self.field_sets.push(schema_location, fields);
                        FieldProvidesRecord {
                            subgraph_id: SubgraphId::GraphqlEndpoint(GraphqlEndpointId::from(subgraph_id)),
                            field_set_id,
                        }
                    })
                    .collect(),
                requires_records: field
                    .requires
                    .into_iter()
                    .filter(|requires| !requires.fields.is_empty())
                    .map(|federated_graph::FieldRequires { subgraph_id, fields }| {
                        let field_set_id = self.field_sets.push(schema_location, fields);
                        FieldRequiresRecord {
                            subgraph_id: SubgraphId::GraphqlEndpoint(GraphqlEndpointId::from(subgraph_id)),
                            field_set_id,
                        }
                    })
                    .collect(),
                argument_ids: self.ctx.idmaps.input_value.get_range(field.arguments),
                directive_ids: directives,
            })
        }
    }

    fn finalize(self) -> Result<(Graph, IntrospectionMetadata), BuildError> {
        let Self {
            ctx,
            field_sets,
            required_scopes,
            mut graph,
        } = self;

        graph.required_scopes = required_scopes.into();
        field_sets.try_insert_into(ctx, &mut graph)?;

        let introspection = IntrospectionBuilder::create_data_source_and_insert_fields(ctx, &mut graph);

        graph.type_definitions_ordered_by_name = {
            let mut definitions = Vec::with_capacity(
                graph.scalar_definitions.len()
                    + graph.object_definitions.len()
                    + graph.interface_definitions.len()
                    + graph.union_definitions.len()
                    + graph.enum_definitions.len()
                    + graph.input_object_definitions.len(),
            );

            // Adding all definitions for introspection & query binding
            definitions.extend(
                (0..graph.scalar_definitions.len()).map(|id| DefinitionId::Scalar(ScalarDefinitionId::from(id))),
            );
            definitions.extend(
                (0..graph.object_definitions.len()).map(|id| DefinitionId::Object(ObjectDefinitionId::from(id))),
            );
            definitions.extend(
                (0..graph.interface_definitions.len())
                    .map(|id| DefinitionId::Interface(InterfaceDefinitionId::from(id))),
            );
            definitions
                .extend((0..graph.union_definitions.len()).map(|id| DefinitionId::Union(UnionDefinitionId::from(id))));
            definitions
                .extend((0..graph.enum_definitions.len()).map(|id| DefinitionId::Enum(EnumDefinitionId::from(id))));
            definitions.extend(
                (0..graph.input_object_definitions.len())
                    .map(|id| DefinitionId::InputObject(InputObjectDefinitionId::from(id))),
            );
            definitions.sort_unstable_by_key(|definition| match *definition {
                DefinitionId::Scalar(id) => &ctx.strings[graph[id].name_id],
                DefinitionId::Object(id) => &ctx.strings[graph[id].name_id],
                DefinitionId::Interface(id) => &ctx.strings[graph[id].name_id],
                DefinitionId::Union(id) => &ctx.strings[graph[id].name_id],
                DefinitionId::Enum(id) => &ctx.strings[graph[id].name_id],
                DefinitionId::InputObject(id) => &ctx.strings[graph[id].name_id],
            });
            definitions
        };

        let mut interface_definitions = std::mem::take(&mut graph.interface_definitions);
        for interface in &mut interface_definitions {
            interface.possible_type_ids.sort_unstable();
            interface
                .possible_types_ordered_by_typename_ids
                .clone_from(&interface.possible_type_ids);
            interface
                .possible_types_ordered_by_typename_ids
                .sort_unstable_by_key(|id| &ctx.strings[graph[*id].name_id]);
        }
        graph.interface_definitions = interface_definitions;

        let mut union_definitions = std::mem::take(&mut graph.union_definitions);
        for union in &mut union_definitions {
            union.possible_type_ids.sort_unstable();
            union
                .possible_types_ordered_by_typename_ids
                .clone_from(&union.possible_type_ids);
            union
                .possible_types_ordered_by_typename_ids
                .sort_unstable_by_key(|id| &ctx.strings[graph[*id].name_id]);
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
                let key_fields_id = self.field_sets.push(location, key.fields.clone());
                let resolver_id = self.push_resolver(ResolverDefinitionRecord::GraphqlFederationEntity(
                    GraphqlFederationEntityResolverDefinitionRecord {
                        endpoint_id,
                        key_fields_id,
                    },
                ));
                entity.keys.push((endpoint_id, resolver_id, key.fields));
            } else {
                // We don't need to differentiate between keys here. We'll be using this to add
                // those fields to `provides` in the relevant fields. It's the resolvable keys
                // that will determine which fields to retrieve during planning. And composition
                // ensures that keys between subgraphs are coherent.
                entity
                    .unresolvable_keys
                    .entry(endpoint_id)
                    .or_default()
                    .push(key.fields.clone());
            }
        }

        if entity.keys.is_empty() && entity.unresolvable_keys.is_empty() {
            None
        } else {
            Some(entity)
        }
    }

    fn push_resolver(&mut self, resolver: ResolverDefinitionRecord) -> ResolverDefinitionId {
        let resolver_id = ResolverDefinitionId::from(self.graph.resolver_definitions.len());
        self.graph.resolver_definitions.push(resolver);
        resolver_id
    }

    fn push_directives(&mut self, config: &Config, directives: Directives) -> Vec<TypeSystemDirectiveId> {
        let mut directive_ids = Vec::new();

        for directive in &config.graph[directives.federated] {
            let id = match directive {
                federated_graph::Directive::Authenticated => TypeSystemDirectiveId::Authenticated,
                federated_graph::Directive::RequiresScopes(federated_scopes) => {
                    let id = self.required_scopes.get_or_insert(RequiresScopesDirectiveRecord::new(
                        federated_scopes
                            .iter()
                            .map(|scopes| scopes.iter().copied().map(Into::into).collect())
                            .collect(),
                    ));
                    TypeSystemDirectiveId::RequiresScopes(id)
                }
                federated_graph::Directive::Deprecated { reason } => {
                    TypeSystemDirectiveId::Deprecated(DeprecatedDirectiveRecord {
                        reason_id: reason.map(Into::into),
                    })
                }
                federated_graph::Directive::Cost { .. } => {
                    // TODO: Implement this
                    continue;
                }
                federated_graph::Directive::Other { .. }
                | federated_graph::Directive::Inaccessible
                | federated_graph::Directive::Policy(_) => continue,
            };
            directive_ids.push(id);
        }

        if let Some((schema_location, directives)) = directives.authorized_directives {
            for id in directives {
                let federated_graph::AuthorizedDirective {
                    fields,
                    arguments,
                    metadata,
                    node,
                } = &config.graph[id];

                self.graph.authorized_directives.push(AuthorizedDirectiveRecord {
                    arguments: arguments
                        .as_ref()
                        .map(|args| self.convert_input_value_set(args))
                        .unwrap_or_default(),
                    fields_id: fields
                        .as_ref()
                        .map(|field_set| self.field_sets.push(schema_location, field_set.clone())),
                    node_id: node
                        .as_ref()
                        .map(|field_set| self.field_sets.push(schema_location, field_set.clone())),
                    metadata_id: metadata.clone().and_then(|value| {
                        let value = self.graph.input_values.ingest_as_json(self.ctx, value).ok()?;

                        Some(self.graph.input_values.push_value(value))
                    }),
                });

                let authorized_id = (self.graph.authorized_directives.len() - 1).into();
                directive_ids.push(TypeSystemDirectiveId::Authorized(authorized_id));
            }
        }

        directive_ids
    }

    fn convert_input_value_set(&self, input_value_set: &federated_graph::InputValueDefinitionSet) -> InputValueSet {
        input_value_set
            .iter()
            .filter_map(|item| {
                self.ctx
                    .idmaps
                    .input_value
                    .get(item.input_value_definition)
                    .map(|id| InputValueSetSelection {
                        id,
                        subselection: self.convert_input_value_set(&item.subselection),
                    })
            })
            .collect()
    }
}

struct Directives {
    federated: federated_graph::Directives,
    authorized_directives: Option<(SchemaLocation, Vec<federated_graph::AuthorizedDirectiveId>)>,
}

impl Default for Directives {
    fn default() -> Self {
        Self {
            federated: (federated_graph::DirectiveId::from(0), 0),
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
    keys: Vec<(GraphqlEndpointId, ResolverDefinitionId, federated_graph::SelectionSet)>,
    unresolvable_keys: HashMap<GraphqlEndpointId, Vec<federated_graph::SelectionSet>>,
}

fn field_set_contains(field_set: &federated_graph::SelectionSet, field_id: federated_graph::FieldId) -> bool {
    field_set.iter().any(|item| match item {
        federated_graph::Selection::Field { field, .. } => *field == field_id,
        federated_graph::Selection::InlineFragment { subselection, .. } => field_set_contains(subselection, field_id),
    })
}

pub(super) fn is_inaccessible(
    graph: &federated_graph::FederatedGraph,
    directives: federated_graph::Directives,
) -> bool {
    graph[directives]
        .iter()
        .any(|directive| matches!(directive, federated_graph::Directive::Inaccessible))
}
