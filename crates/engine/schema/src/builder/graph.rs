use std::{
    collections::{BTreeSet, HashMap},
    mem::take,
};

use builder::coerce::InputValueCoercer;
use config::Config;
use federated_graph::{JoinFieldDirective, JoinImplementsDirective, JoinTypeDirective, JoinUnionMemberDirective};
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
                cost_directives: Vec::new(),
                list_size_directives: Vec::new(),
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

        self.ingest_fields_after_input_values_before_objects_and_interfaces(config);

        self.ingest_objects(config);
        self.ingest_interfaces_after_objects(config);
        self.ingest_unions_after_objects(config);

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
                    // FIXME: better input value schema location...
                    SchemaLocation::Definition {
                        name: definition.name.into(),
                    },
                    &definition.directives,
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
                            SchemaLocation::Definition {
                                name: definition.name.into(),
                            },
                            &definition.directives,
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
                SchemaLocation::Definition {
                    name: union.name.into(),
                },
                &union.directives,
            );

            let mut join_member_records: Vec<_> = union
                .directives
                .iter()
                .filter_map(|dir| dir.as_join_union_member())
                .map(
                    |&JoinUnionMemberDirective { subgraph_id, object_id }| JoinMemberDefinitionRecord {
                        subgraph_id: SubgraphId::GraphqlEndpoint(subgraph_id.into()),
                        member_id: object_id.into(),
                    },
                )
                .collect();

            join_member_records.sort_by_key(|record| (record.subgraph_id, record.member_id));
            let mut not_fully_implemented_in_ids = BTreeSet::new();
            for object_id in &possible_type_ids {
                let object = &self.graph[*object_id];

                // Check in which subgraphs these are resolved.
                for subgraph_id in &object.exists_in_subgraph_ids {
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
                description_id: union.description.map(Into::into),
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
                if is_inaccessible(&config.graph, &enum_value.directives) {
                    self.ctx
                        .idmaps
                        .enum_values
                        .skip(federated_graph::EnumValueId::from(idx));
                    None
                } else {
                    Some(EnumValueRecord {
                        name_id: enum_value.value.into(),
                        description_id: enum_value.description.map(Into::into),
                        directive_ids: self.push_directives(
                            config,
                            // FIXME: better schema location for enum values...
                            SchemaLocation::Definition {
                                name: enum_value.value.into(),
                            },
                            &enum_value.directives,
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
                description_id: federated_enum.description.map(Into::into),
                value_ids: self
                    .ctx
                    .idmaps
                    .enum_values
                    .get_range(config.graph.enum_value_range(federated_enum.id())),
                directive_ids: self.push_directives(
                    config,
                    SchemaLocation::Definition {
                        name: federated_enum.name.into(),
                    },
                    &federated_enum.directives,
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
                    description_id: scalar.description.map(Into::into),
                    specified_by_url_id: None,
                    directive_ids: self.push_directives(
                        config,
                        SchemaLocation::Definition { name },
                        &scalar.directives,
                    ),
                }
            })
            .collect();
    }

    fn ingest_objects(&mut self, config: &mut Config) {
        self.graph.object_definitions = Vec::with_capacity(config.graph.objects.len());
        for object in take(&mut config.graph.objects).into_iter() {
            let definition = config.graph.at(object.type_definition_id);

            let fields = self.ctx.idmaps.field.get_range((
                object.fields.start,
                usize::from(object.fields.end) - usize::from(object.fields.start),
            ));

            let schema_location = SchemaLocation::Definition {
                name: config.graph.view(object.type_definition_id).name.into(),
            };

            let directives = self.push_directives(
                config,
                schema_location,
                &config.graph[object.type_definition_id].directives,
            );

            let mut join_implement_records: Vec<_> = config.graph[object.type_definition_id]
                .directives
                .iter()
                .filter_map(|dir| dir.as_join_implements())
                .map(
                    |&JoinImplementsDirective {
                         subgraph_id,
                         interface_id,
                     }| {
                        JoinImplementsDefinitionRecord {
                            subgraph_id: SubgraphId::GraphqlEndpoint(subgraph_id.into()),
                            interface_id: interface_id.into(),
                        }
                    },
                )
                .collect();

            join_implement_records.sort_by_key(|record| (record.subgraph_id, record.interface_id));

            let mut exists_in_subgraph_ids = config.graph[object.type_definition_id]
                .directives
                .iter()
                .filter_map(|dir| dir.as_join_type())
                .map(|dir| SubgraphId::GraphqlEndpoint(dir.subgraph_id.into()))
                .collect::<Vec<_>>();

            exists_in_subgraph_ids.sort_unstable();

            self.graph.object_definitions.push(ObjectDefinitionRecord {
                name_id: config.graph.view(object.type_definition_id).name.into(),
                description_id: definition.description.map(Into::into),
                interface_ids: object.implements_interfaces.into_iter().map(Into::into).collect(),
                directive_ids: directives,
                field_ids: fields,
                join_implement_records,
                exists_in_subgraph_ids,
            });
        }
    }

    fn ingest_interfaces_after_objects(&mut self, config: &mut Config) {
        self.graph.interface_definitions = Vec::with_capacity(config.graph.interfaces.len());
        for interface in take(&mut config.graph.interfaces) {
            let name_id = config.graph.view(interface.type_definition_id).name.into();
            let definition = config.graph.at(interface.type_definition_id);

            let fields = self.ctx.idmaps.field.get_range((
                interface.fields.start,
                usize::from(interface.fields.end) - usize::from(interface.fields.start),
            ));

            let directives = self.push_directives(
                config,
                SchemaLocation::Definition { name: name_id },
                &config.graph[interface.type_definition_id].directives,
            );

            self.graph.interface_definitions.push(InterfaceDefinitionRecord {
                name_id,
                description_id: definition.description.map(Into::into),
                interface_ids: interface.implements_interfaces.into_iter().map(Into::into).collect(),
                possible_type_ids: Vec::new(),
                // Added at the end.
                possible_types_ordered_by_typename_ids: Vec::new(),
                directive_ids: directives,
                field_ids: fields,
                // Added at the end.
                not_fully_implemented_in_ids: Vec::new(),
            });
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
                for subgraph_id in &object.exists_in_subgraph_ids {
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
    }

    fn ingest_fields_after_input_values_before_objects_and_interfaces(&mut self, config: &mut Config) {
        let root_entities = [
            Some(EntityDefinitionId::from(ObjectDefinitionId::from(
                config.graph.root_operation_types.query,
            ))),
            config
                .graph
                .root_operation_types
                .mutation
                .map(|id| EntityDefinitionId::from(ObjectDefinitionId::from(id))),
            config
                .graph
                .root_operation_types
                .subscription
                .map(|id| EntityDefinitionId::from(ObjectDefinitionId::from(id))),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        #[derive(Clone)]
        enum FieldResovler {
            Root(ResolverDefinitionId),
            Entity {
                key: federated_graph::SelectionSet,
                id: ResolverDefinitionId,
            },
        }

        impl FieldResovler {
            fn id(&self) -> ResolverDefinitionId {
                match self {
                    FieldResovler::Root(id) | FieldResovler::Entity { id, .. } => *id,
                }
            }
        }

        let mut field_resolvers = HashMap::<(EntityDefinitionId, GraphqlEndpointId), Vec<FieldResovler>>::new();
        for (federated_id, field) in take(&mut config.graph.fields).into_iter().enumerate() {
            let federated_id = federated_graph::FieldId::from(federated_id);
            let Some(_) = self.ctx.idmaps.field.get(federated_id) else {
                continue;
            };

            let parent_entity_id = field.parent_entity_id.into();
            let parent_entity = config.graph.entity(field.parent_entity_id);
            let type_schema_location = SchemaLocation::Definition {
                name: parent_entity.name(&config.graph).into(),
            };
            let field_schema_location = SchemaLocation::Field {
                ty: parent_entity.name(&config.graph).into(),
                name: field.name.into(),
            };

            let mut distinct_type_in_ids = Vec::new();
            let mut requires_records = Vec::new();
            let mut provides_records = Vec::new();
            // BTreeSet to ensures consistent ordering of resolvers.
            let mut only_resolvable_in = BTreeSet::new();
            let mut has_join_field = false;

            for JoinFieldDirective {
                subgraph_id: federated_subgraph_id,
                requires,
                provides,
                r#type,
                ..
            } in field.directives.iter().filter_map(|dir| dir.as_join_field())
            {
                has_join_field = true;
                let subgraph_id = SubgraphId::GraphqlEndpoint((*federated_subgraph_id).into());
                if r#type.as_ref().is_some_and(|ty| ty != &field.r#type) {
                    distinct_type_in_ids.push(subgraph_id);
                }
                if let Some(provides) = provides.as_ref().filter(|provides| !provides.is_empty()) {
                    let field_set_id = self.field_sets.push(field_schema_location, provides.clone());
                    provides_records.push(FieldProvidesRecord {
                        subgraph_id,
                        field_set_id,
                    });
                }
                if let Some(requires) = requires.as_ref().filter(|requires| !requires.is_empty()) {
                    let field_set_id = self.field_sets.push(field_schema_location, requires.clone());
                    requires_records.push(FieldRequiresRecord {
                        subgraph_id,
                        field_set_id,
                    });
                }
                only_resolvable_in.insert((*federated_subgraph_id).into());
            }

            for JoinTypeDirective {
                subgraph_id,
                key,
                resolvable,
                ..
            } in parent_entity
                .directives(&config.graph)
                .filter_map(|dir| dir.as_join_type())
            {
                // If present in the keys as a subgraph must always be able to provide those at least.
                if key.as_ref().and_then(|key| key.find_field(federated_id)).is_some() {
                    only_resolvable_in.insert((*subgraph_id).into());
                } else if !has_join_field && *resolvable {
                    // If there is no @join__field we rely solely @join__type to define the subgraphs
                    // in which this field is resolvable in.
                    only_resolvable_in.insert((*subgraph_id).into());
                }
            }

            // Remove any overridden subgraphs
            for directive in field.directives.iter().filter_map(|dir| dir.as_join_field()) {
                if let Some(r#override) = &directive.r#override {
                    match r#override {
                        federated_graph::OverrideSource::Subgraph(subgraph_id) => {
                            only_resolvable_in.remove(&(*subgraph_id).into());
                        }
                        federated_graph::OverrideSource::Missing(_) => (),
                    };
                }
            }

            let mut resolver_ids = vec![];
            if root_entities.contains(&parent_entity_id) {
                for &endpoint_id in &only_resolvable_in {
                    resolver_ids.extend(
                        field_resolvers
                            .entry((parent_entity_id, endpoint_id))
                            .or_insert_with(|| {
                                vec![FieldResovler::Root(self.push_resolver(
                                    ResolverDefinitionRecord::GraphqlRootField(
                                        GraphqlRootFieldResolverDefinitionRecord { endpoint_id },
                                    ),
                                ))]
                            })
                            .iter()
                            .map(|res| res.id()),
                    );
                }
            } else {
                for &endpoint_id in &only_resolvable_in {
                    let endpoint_resolvers =
                        field_resolvers
                            .entry((parent_entity_id, endpoint_id))
                            .or_insert_with(|| {
                                parent_entity
                                    .directives(&config.graph)
                                    .filter_map(|dir| dir.as_join_type())
                                    .filter_map(|dir| {
                                        dir.key.as_ref().filter(|key| {
                                            !key.is_empty()
                                                && GraphqlEndpointId::from(dir.subgraph_id) == endpoint_id
                                                && dir.resolvable
                                        })
                                    })
                                    .map(|key| {
                                        let key_fields_id = self.field_sets.push(type_schema_location, key.clone());
                                        let id = self.push_resolver(ResolverDefinitionRecord::GraphqlFederationEntity(
                                            GraphqlFederationEntityResolverDefinitionRecord {
                                                key_fields_id,
                                                endpoint_id,
                                            },
                                        ));
                                        FieldResovler::Entity { key: key.clone(), id }
                                    })
                                    .collect::<Vec<_>>()
                            });
                    for res in endpoint_resolvers {
                        let FieldResovler::Entity { id, key } = res else {
                            continue;
                        };
                        // If part of the key we can't be provided by this resolver.
                        if key.find_field(federated_id).is_none() {
                            resolver_ids.push(*id);
                        }
                    }
                }
            }

            // If resolvable in all subgraphs, there is no need for `only_resolvable_in` from this
            // point on.
            if parent_entity
                .directives(&config.graph)
                .filter_map(|dir| dir.as_join_type())
                .all(|dir| only_resolvable_in.contains(&dir.subgraph_id.into()))
            {
                only_resolvable_in.clear();
            }

            let directive_ids = self.push_directives(config, field_schema_location, &field.directives);

            self.graph.field_definitions.push(FieldDefinitionRecord {
                name_id: field.name.into(),
                description_id: field.description.map(Into::into),
                parent_entity_id,
                distinct_type_in_ids,
                ty_record: self.ctx.convert_type(field.r#type),
                only_resolvable_in_ids: only_resolvable_in
                    .into_iter()
                    .map(SubgraphId::GraphqlEndpoint)
                    .collect(),
                resolver_ids,
                provides_records,
                requires_records,
                argument_ids: self.ctx.idmaps.input_value.get_range(field.arguments),
                directive_ids,
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

    fn push_resolver(&mut self, resolver: ResolverDefinitionRecord) -> ResolverDefinitionId {
        let resolver_id = ResolverDefinitionId::from(self.graph.resolver_definitions.len());
        self.graph.resolver_definitions.push(resolver);
        resolver_id
    }

    fn push_directives<'d>(
        &mut self,
        _config: &Config,
        schema_location: SchemaLocation,
        directives: impl IntoIterator<Item = &'d federated_graph::Directive>,
    ) -> Vec<TypeSystemDirectiveId> {
        let mut directive_ids = Vec::new();

        for directive in directives {
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
                federated_graph::Directive::Authorized(authorized) => {
                    self.graph.authorized_directives.push(AuthorizedDirectiveRecord {
                        arguments: authorized
                            .arguments
                            .as_ref()
                            .map(|args| self.convert_input_value_set(args))
                            .unwrap_or_default(),
                        fields_id: authorized
                            .fields
                            .as_ref()
                            .map(|field_set| self.field_sets.push(schema_location, field_set.clone())),
                        node_id: authorized
                            .node
                            .as_ref()
                            .map(|field_set| self.field_sets.push(schema_location, field_set.clone())),
                        metadata_id: authorized.metadata.clone().and_then(|value| {
                            let value = self.graph.input_values.ingest_as_json(self.ctx, value).ok()?;

                            Some(self.graph.input_values.push_value(value))
                        }),
                    });

                    let authorized_id = (self.graph.authorized_directives.len() - 1).into();
                    TypeSystemDirectiveId::Authorized(authorized_id)
                }
                federated_graph::Directive::Cost { weight } => {
                    let cost_id = self.graph.cost_directives.len().into();
                    self.graph.cost_directives.push(CostDirectiveRecord { weight: *weight });
                    TypeSystemDirectiveId::Cost(cost_id)
                }
                federated_graph::Directive::ListSize(federated_graph::ListSize {
                    assumed_size,
                    slicing_arguments,
                    sized_fields,
                    require_one_slicing_argument,
                }) => {
                    let list_size_id = self.graph.list_size_directives.len().into();
                    self.graph.list_size_directives.push(ListSizeDirectiveRecord {
                        assumed_size: *assumed_size,
                        slicing_argument_ids: slicing_arguments
                            .iter()
                            .filter_map(|id| self.ctx.idmaps.input_value.get(*id))
                            .collect(),
                        sized_field_ids: sized_fields
                            .iter()
                            .filter_map(|id| self.ctx.idmaps.field.get(*id))
                            .collect(),
                        require_one_slicing_argument: *require_one_slicing_argument,
                    });
                    TypeSystemDirectiveId::ListSize(list_size_id)
                }
                federated_graph::Directive::Other { .. }
                | federated_graph::Directive::Inaccessible
                | federated_graph::Directive::Policy(_)
                | federated_graph::Directive::JoinField(_)
                | federated_graph::Directive::JoinType(_)
                | federated_graph::Directive::JoinUnionMember(_)
                | federated_graph::Directive::JoinImplements(_) => continue,
            };

            directive_ids.push(id);
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

pub(super) fn is_inaccessible<'a>(
    _graph: &federated_graph::FederatedGraph,
    directives: impl IntoIterator<Item = &'a federated_graph::Directive>,
) -> bool {
    directives
        .into_iter()
        .any(|directive| matches!(directive, federated_graph::Directive::Inaccessible))
}
