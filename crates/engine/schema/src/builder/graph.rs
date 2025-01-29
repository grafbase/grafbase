use std::{collections::BTreeSet, mem::take};

use builder::{coerce::InputValueCoercer, external_sources::ExternalDataSources};
use config::Config;
use federated_graph::{JoinFieldDirective, JoinImplementsDirective, JoinTypeDirective, JoinUnionMemberDirective};
use fxhash::FxHashMap;
use introspection::{IntrospectionBuilder, IntrospectionMetadata};
use runtime::extension::ExtensionCatalog;

use crate::*;

use super::{interner::Interner, BuildContext, BuildError, FieldSetsBuilder, SchemaLocation};

pub(crate) struct GraphBuilder<'a, EC> {
    ctx: &'a mut BuildContext<EC>,
    sources: &'a ExternalDataSources,
    field_sets: FieldSetsBuilder,
    all_subgraphs: Vec<SubgraphId>,
    required_scopes: Interner<RequiresScopesDirectiveRecord, RequiresScopesDirectiveId>,
    graph: Graph,
    graphql_federated_entity_resolvers: FxHashMap<(EntityDefinitionId, GraphqlEndpointId), Vec<EntityResovler>>,
}

#[derive(Clone)]
enum EntityResovler {
    Root(ResolverDefinitionId),
    Entity {
        key: federated_graph::SelectionSet,
        id: ResolverDefinitionId,
    },
}

impl EntityResovler {
    fn id(&self) -> ResolverDefinitionId {
        match self {
            EntityResovler::Root(id) | EntityResovler::Entity { id, .. } => *id,
        }
    }
}

impl<'a, EC: ExtensionCatalog> GraphBuilder<'a, EC> {
    pub fn build(
        ctx: &'a mut BuildContext<EC>,
        sources: &'a ExternalDataSources,
        config: &mut Config,
    ) -> Result<(Graph, IntrospectionMetadata), BuildError> {
        let mut all_subgraphs = sources.iter().collect::<Vec<_>>();
        all_subgraphs.sort_unstable();

        let mut builder = GraphBuilder {
            ctx,
            sources,
            field_sets: Default::default(),
            all_subgraphs,
            required_scopes: Default::default(),
            graph: Graph {
                description_id: None,
                root_operation_types_record: RootOperationTypesRecord {
                    query_id: config.graph.root_operation_types.query.into(),
                    mutation_id: config.graph.root_operation_types.mutation.map(Into::into),
                    subscription_id: config.graph.root_operation_types.subscription.map(Into::into),
                },
                object_definitions: Vec::new(),
                inaccessible_object_definitions: BitSet::new(),
                interface_definitions: Vec::new(),
                inaccessible_interface_definitions: BitSet::new(),
                interface_has_inaccessible_implementor: BitSet::new(),
                union_definitions: Vec::new(),
                inaccessible_union_definitions: BitSet::new(),
                union_has_inaccessible_member: BitSet::new(),
                scalar_definitions: Vec::new(),
                inaccessible_scalar_definitions: BitSet::new(),
                enum_definitions: Vec::new(),
                inaccessible_enum_definitions: BitSet::new(),
                enum_values: Vec::new(),
                inaccessible_enum_values: BitSet::new(),
                input_object_definitions: Vec::new(),
                inaccessible_input_object_definitions: BitSet::new(),
                input_value_definitions: Vec::new(),
                inaccessible_input_value_definitions: BitSet::new(),
                field_definitions: Vec::new(),
                inaccessible_field_definitions: BitSet::new(),
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
                extension_directives: Vec::new(),
            },
            graphql_federated_entity_resolvers: Default::default(),
        };
        builder.ingest_config(config)?;
        builder.finalize()
    }

    fn ingest_config(&mut self, config: &mut Config) -> Result<(), BuildError> {
        self.ingest_enums(config)?;
        self.ingest_scalars(config)?;
        self.ingest_input_objects(config)?;
        self.ingest_input_values_after_scalars_and_input_objects_and_enums(config)?;
        self.ingest_fields_after_input_values(config)?;
        self.ingest_objects(config)?;
        self.ingest_interfaces_after_objects_and_fields(config)?;
        self.ingest_unions_after_objects(config)?;

        Ok(())
    }

    fn ingest_input_values_after_scalars_and_input_objects_and_enums(
        &mut self,
        config: &mut Config,
    ) -> Result<(), BuildError> {
        // Arbitrary initial capacity, to make it at least proportional to the input_values count.
        let mut default_values = Vec::with_capacity(config.graph.input_value_definitions.len() / 20);
        self.graph.input_value_definitions = Vec::with_capacity(config.graph.input_value_definitions.len());
        self.graph.inaccessible_input_value_definitions =
            BitSet::with_capacity(config.graph.input_value_definitions.len());
        for (ix, definition) in take(&mut config.graph.input_value_definitions).into_iter().enumerate() {
            let id = InputValueDefinitionId::from(ix);
            if let Some(value) = definition.default {
                default_values.push((id, value));
            }
            if has_inaccessible(&definition.directives) {
                self.graph.inaccessible_input_value_definitions.set(id, true);
            }
            let directive_ids = self.push_directives(
                config,
                // FIXME: better input value schema location...
                SchemaLocation::Definition {
                    name: definition.name.into(),
                },
                &definition.directives,
            )?;
            self.graph.input_value_definitions.push(InputValueDefinitionRecord {
                name_id: definition.name.into(),
                description_id: definition.description.map(Into::into),
                ty_record: self.ctx.convert_type(definition.r#type),
                // Adding after ingesting all input values as input object fields are input values.
                // So we need them for coercion.
                default_value_id: None,
                directive_ids,
            });
        }

        let mut input_values = take(&mut self.graph.input_values);
        let mut coercer = InputValueCoercer::new(self.ctx, &self.graph, &mut input_values);

        let default_values = default_values
            .into_iter()
            .map(|(id, value)| {
                let input_value_definition = &self.graph[id];
                let value = coercer.coerce(input_value_definition.ty_record, value).map_err(|err| {
                    BuildError::DefaultValueCoercionError {
                        err,
                        name: self.ctx.strings[input_value_definition.name_id].to_string(),
                    }
                })?;
                Ok((id, value))
            })
            .collect::<Result<Vec<_>, BuildError>>()?;

        for (id, value_id) in default_values {
            self.graph[id].default_value_id = Some(value_id);
        }

        self.graph.input_values = input_values;

        Ok(())
    }

    fn ingest_input_objects(&mut self, config: &mut Config) -> Result<(), BuildError> {
        self.graph.input_object_definitions = Vec::with_capacity(config.graph.input_objects.len());
        self.graph.inaccessible_input_object_definitions = BitSet::with_capacity(config.graph.input_objects.len());
        for (ix, definition) in take(&mut config.graph.input_objects).into_iter().enumerate() {
            if has_inaccessible(&definition.directives) {
                self.graph.inaccessible_input_object_definitions.set(ix.into(), true);
            }
            let directive_ids = self.push_directives(
                config,
                SchemaLocation::Definition {
                    name: definition.name.into(),
                },
                &definition.directives,
            )?;
            self.graph.input_object_definitions.push(InputObjectDefinitionRecord {
                name_id: definition.name.into(),
                description_id: definition.description.map(Into::into),
                input_field_ids: IdRange::from_start_and_length(definition.fields),
                directive_ids,
            });
        }

        Ok(())
    }

    fn ingest_unions_after_objects(&mut self, config: &mut Config) -> Result<(), BuildError> {
        self.graph.union_definitions = Vec::with_capacity(config.graph.unions.len());
        self.graph.inaccessible_union_definitions = BitSet::with_capacity(config.graph.unions.len());
        self.graph.union_has_inaccessible_member = BitSet::with_capacity(config.graph.unions.len());
        for (ix, union) in take(&mut config.graph.unions).into_iter().enumerate() {
            if has_inaccessible(&union.directives) {
                self.graph.inaccessible_union_definitions.set(ix.into(), true);
            }

            let possible_type_ids = union
                .members
                .into_iter()
                .map(ObjectDefinitionId::from)
                .collect::<Vec<_>>();

            for object_id in &possible_type_ids {
                if self.graph.inaccessible_object_definitions[*object_id] {
                    self.graph
                        .union_has_inaccessible_member
                        .set(UnionDefinitionId::from(ix), true);
                    break;
                }
            }

            let directive_ids = self.push_directives(
                config,
                SchemaLocation::Definition {
                    name: union.name.into(),
                },
                &union.directives,
            )?;

            let mut join_member_records: Vec<_> = union
                .directives
                .iter()
                .filter_map(|dir| dir.as_join_union_member())
                .map(
                    |&JoinUnionMemberDirective { subgraph_id, object_id }| JoinMemberDefinitionRecord {
                        subgraph_id: self.sources[subgraph_id],
                        member_id: object_id.into(),
                    },
                )
                .collect();
            let mut union_subgraph_ids = join_member_records
                .iter()
                .map(|join| join.subgraph_id)
                .collect::<Vec<_>>();
            union_subgraph_ids.sort_unstable();
            union_subgraph_ids.dedup();

            join_member_records.sort_by_key(|record| (record.subgraph_id, record.member_id));
            let mut not_fully_implemented_in_ids = BTreeSet::new();
            for object_id in &possible_type_ids {
                for subgraph_id in &union_subgraph_ids {
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

            self.graph.union_definitions.push(UnionDefinitionRecord {
                name_id: union.name.into(),
                description_id: union.description.map(Into::into),
                possible_type_ids,
                // Added at the end.
                possible_types_ordered_by_typename_ids: Vec::new(),
                directive_ids,
                join_member_records,
                not_fully_implemented_in_ids: not_fully_implemented_in_ids.into_iter().collect(),
            });
        }

        Ok(())
    }

    fn ingest_enums(&mut self, config: &mut Config) -> Result<(), BuildError> {
        for federated_enum in config.graph.iter_enum_definitions() {
            if federated_enum.namespace.is_some() {
                continue;
            }

            let id = EnumDefinitionId::from(self.graph.enum_definitions.len());
            self.ctx.enum_mapping.insert(federated_enum.id(), id);
            self.graph
                .inaccessible_enum_definitions
                .push(has_inaccessible(&federated_enum.directives));

            let directive_ids = self.push_directives(
                config,
                SchemaLocation::Definition {
                    name: federated_enum.name.into(),
                },
                &federated_enum.directives,
            )?;
            self.graph.enum_definitions.push(EnumDefinitionRecord {
                name_id: federated_enum.name.into(),
                description_id: federated_enum.description.map(Into::into),
                value_ids: IdRange::from_start_and_length(config.graph.enum_value_range(federated_enum.id())),
                directive_ids,
            })
        }

        // Enum values MUST be after enum definitions as otherwise enums will be empty.
        self.graph.enum_values = Vec::with_capacity(config.graph.enum_values.len());
        self.graph.inaccessible_enum_values = BitSet::with_capacity(config.graph.enum_values.len());
        for (ix, enum_value) in take(&mut config.graph.enum_values).into_iter().enumerate() {
            if has_inaccessible(&enum_value.directives) {
                self.graph.inaccessible_enum_values.set(ix.into(), true);
            }
            let directive_ids = self.push_directives(
                config,
                // FIXME: better schema location for enum values...
                SchemaLocation::Definition {
                    name: enum_value.value.into(),
                },
                &enum_value.directives,
            )?;
            self.graph.enum_values.push(EnumValueRecord {
                name_id: enum_value.value.into(),
                description_id: enum_value.description.map(Into::into),
                directive_ids,
            });
        }

        Ok(())
    }

    fn ingest_scalars(&mut self, config: &mut Config) -> Result<(), BuildError> {
        for scalar in config.graph.iter_scalar_definitions() {
            if scalar.namespace.is_some() {
                continue;
            }

            let id = ScalarDefinitionId::from(self.graph.scalar_definitions.len());
            self.ctx.scalar_mapping.insert(scalar.id(), id);
            self.graph
                .inaccessible_scalar_definitions
                .push(has_inaccessible(&scalar.directives));
            let name = StringId::from(scalar.name);
            let directive_ids =
                self.push_directives(config, SchemaLocation::Definition { name }, &scalar.directives)?;
            self.graph.scalar_definitions.push(ScalarDefinitionRecord {
                name_id: name,
                ty: ScalarType::from_scalar_name(&self.ctx.strings[name]),
                description_id: scalar.description.map(Into::into),
                specified_by_url_id: None,
                directive_ids,
            })
        }

        Ok(())
    }

    fn ingest_objects(&mut self, config: &mut Config) -> Result<(), BuildError> {
        self.graph.object_definitions = Vec::with_capacity(config.graph.objects.len());
        self.graph.inaccessible_object_definitions = BitSet::with_capacity(config.graph.objects.len());
        for (ix, object) in take(&mut config.graph.objects).into_iter().enumerate() {
            let name_id = object.name.into();
            let federated_directives = &object.directives;
            if has_inaccessible(federated_directives) {
                self.graph.inaccessible_object_definitions.set(ix.into(), true);
            }

            let directives = self.push_directives(
                config,
                SchemaLocation::Definition { name: name_id },
                federated_directives,
            )?;

            let mut join_implement_records: Vec<_> = object
                .directives
                .iter()
                .filter_map(|dir| dir.as_join_implements())
                .map(
                    |&JoinImplementsDirective {
                         subgraph_id,
                         interface_id,
                     }| {
                        JoinImplementsDefinitionRecord {
                            subgraph_id: self.sources[subgraph_id],
                            interface_id: interface_id.into(),
                        }
                    },
                )
                .collect();

            join_implement_records.sort_by_key(|record| (record.subgraph_id, record.interface_id));

            let mut exists_in_subgraph_ids = object
                .directives
                .iter()
                .filter_map(|dir| dir.as_join_type())
                .map(|dir| self.sources[dir.subgraph_id])
                .collect::<Vec<_>>();
            if exists_in_subgraph_ids.is_empty() {
                exists_in_subgraph_ids = self.all_subgraphs.clone()
            } else {
                exists_in_subgraph_ids.sort_unstable();
            }

            self.graph.object_definitions.push(ObjectDefinitionRecord {
                name_id,
                description_id: object.description.map(Into::into),
                interface_ids: object.implements_interfaces.into_iter().map(Into::into).collect(),
                directive_ids: directives,
                field_ids: IdRange::from(object.fields),
                join_implement_records,
                exists_in_subgraph_ids,
            });
        }

        Ok(())
    }

    fn ingest_interfaces_after_objects_and_fields(&mut self, config: &mut Config) -> Result<(), BuildError> {
        self.graph.interface_definitions = Vec::with_capacity(config.graph.interfaces.len());
        self.graph.inaccessible_interface_definitions = BitSet::with_capacity(config.graph.interfaces.len());
        self.graph.interface_has_inaccessible_implementor = BitSet::with_capacity(config.graph.interfaces.len());
        for (ix, interface) in take(&mut config.graph.interfaces).into_iter().enumerate() {
            let name_id = interface.name.into();
            let federated_directives = &interface.directives;

            if has_inaccessible(federated_directives) {
                self.graph.inaccessible_interface_definitions.set(ix.into(), true);
            }

            let directives = self.push_directives(
                config,
                SchemaLocation::Definition { name: name_id },
                federated_directives,
            )?;

            let mut exists_in_subgraph_ids = Vec::new();
            let mut is_interface_object_in_ids = Vec::new();

            for dir in interface.directives.iter().filter_map(|dir| dir.as_join_type()) {
                exists_in_subgraph_ids.push(self.sources[dir.subgraph_id]);
                if dir.is_interface_object {
                    is_interface_object_in_ids.push(self.sources[dir.subgraph_id]);
                }
            }

            self.graph.interface_definitions.push(InterfaceDefinitionRecord {
                name_id,
                description_id: interface.description.map(Into::into),
                interface_ids: interface.implements_interfaces.into_iter().map(Into::into).collect(),
                possible_type_ids: Vec::new(),
                // Added at the end.
                possible_types_ordered_by_typename_ids: Vec::new(),
                directive_ids: directives,
                field_ids: IdRange::from(interface.fields),
                // Added at the end.
                not_fully_implemented_in_ids: Vec::new(),
                exists_in_subgraph_ids,
                is_interface_object_in_ids,
            });
        }

        // Adding all implementations of an interface, used during introspection.
        for object_id in (0..self.graph.object_definitions.len()).map(ObjectDefinitionId::from) {
            for interface_id in self.graph[object_id].interface_ids.clone() {
                self.graph[interface_id].possible_type_ids.push(object_id);
                if self.graph.inaccessible_object_definitions[object_id] {
                    self.graph
                        .interface_has_inaccessible_implementor
                        .set(interface_id, true);
                }
            }
        }

        // Adding all not fully implemented interfaces per subgraph.
        for interface_id in (0..self.graph.interface_definitions.len()).map(InterfaceDefinitionId::from) {
            let mut not_fully_implemented_in = BTreeSet::<SubgraphId>::new();

            // For every possible type implementing this interface.
            for object_id in &self.graph[interface_id].possible_type_ids {
                let object = &self.graph[*object_id];

                // Check in which subgraphs these are resolved.
                for subgraph_id in &self.graph[interface_id].exists_in_subgraph_ids {
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

        Ok(())
    }

    fn ingest_fields_after_input_values(&mut self, config: &mut Config) -> Result<(), BuildError> {
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

        self.graph.field_definitions = Vec::with_capacity(config.graph.fields.len());
        self.graph.inaccessible_field_definitions = BitSet::with_capacity(config.graph.fields.len());
        let mut graphql_federated_entity_resolvers = std::mem::take(&mut self.graphql_federated_entity_resolvers);
        for (ix, field) in take(&mut config.graph.fields).into_iter().enumerate() {
            let federated_id = federated_graph::FieldId::from(ix);

            if has_inaccessible(&field.directives) {
                self.graph.inaccessible_field_definitions.set(ix.into(), true);
            }

            let parent_entity_id = field.parent_entity_id.into();
            let parent_entity = config.graph.entity(field.parent_entity_id);
            let type_schema_location = SchemaLocation::Definition {
                name: parent_entity.name().into(),
            };
            let field_schema_location = SchemaLocation::Field {
                ty: parent_entity.name().into(),
                name: field.name.into(),
            };

            let mut subgraph_type_records = Vec::new();
            let mut requires_records = Vec::new();
            let mut provides_records = Vec::new();
            // BTreeSet to ensures consistent ordering of resolvers.
            let mut resolvable_in = BTreeSet::new();
            let mut has_join_field = false;

            for JoinFieldDirective {
                subgraph_id: federated_subgraph_id,
                requires,
                provides,
                r#type,
                ..
            } in field.directives.iter().filter_map(|dir| dir.as_join_field())
            {
                // If there is a @join__field we rely solely on that to define the subgraphs in
                // which this field exists. It may not specify a subgraph at all, in that case it's
                // a interfaceObject field.
                has_join_field = true;
                if let Some(federated_subgraph_id) = *federated_subgraph_id {
                    let subgraph_id = self.sources[federated_subgraph_id];
                    if let Some(r#type) = r#type.filter(|ty| ty != &field.r#type) {
                        subgraph_type_records.push(SubgraphTypeRecord {
                            subgraph_id,
                            ty_record: self.ctx.convert_type(r#type),
                        });
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
                    resolvable_in.insert(subgraph_id);
                }
            }

            let mut parent_has_join_type = false;
            for JoinTypeDirective {
                subgraph_id,
                key,
                resolvable,
                ..
            } in parent_entity.directives().filter_map(|dir| dir.as_join_type())
            {
                parent_has_join_type = true;
                // If present in the keys as a subgraph must always be able to provide those at least.
                if key.as_ref().and_then(|key| key.find_field(federated_id)).is_some() {
                    resolvable_in.insert(self.sources[*subgraph_id]);
                } else if !has_join_field && *resolvable {
                    // If there is no @join__field we rely solely @join__type to define the subgraphs
                    // in which this field is resolvable in.
                    resolvable_in.insert(self.sources[*subgraph_id]);
                }
            }

            // Remove any overridden subgraphs
            for directive in field.directives.iter().filter_map(|dir| dir.as_join_field()) {
                if let Some(r#override) = &directive.r#override {
                    match r#override {
                        federated_graph::OverrideSource::Subgraph(subgraph_id) => {
                            resolvable_in.remove(&self.sources[*subgraph_id]);
                        }
                        federated_graph::OverrideSource::Missing(_) => (),
                    };
                }
            }

            // If there is no @join__field and no @join__type at all, we assume this field to be
            // available everywhere.
            let exists_in_subgraph_ids = if !has_join_field && !parent_has_join_type {
                self.all_subgraphs.clone()
            } else {
                resolvable_in.into_iter().collect::<Vec<_>>()
            };

            let mut resolver_ids: Vec<ResolverDefinitionId> = vec![];
            let is_root_entity = root_entities.contains(&parent_entity_id);
            for &subgraph_id in &exists_in_subgraph_ids {
                match subgraph_id {
                    SubgraphId::GraphqlEndpoint(endpoint_id) if is_root_entity => {
                        resolver_ids.extend(
                            graphql_federated_entity_resolvers
                                .entry((parent_entity_id, endpoint_id))
                                .or_insert_with(|| {
                                    vec![EntityResovler::Root(self.push_resolver(
                                        ResolverDefinitionRecord::GraphqlRootField(
                                            GraphqlRootFieldResolverDefinitionRecord { endpoint_id },
                                        ),
                                    ))]
                                })
                                .iter()
                                .map(|res| res.id()),
                        );
                    }
                    SubgraphId::GraphqlEndpoint(endpoint_id) => {
                        let endpoint_resolvers = graphql_federated_entity_resolvers
                            .entry((parent_entity_id, endpoint_id))
                            .or_insert_with(|| {
                                parent_entity
                                    .directives()
                                    .filter_map(|dir| dir.as_join_type())
                                    .filter_map(|dir| {
                                        dir.key.as_ref().filter(|key| {
                                            !key.is_empty()
                                                && self.sources[dir.subgraph_id] == subgraph_id
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
                                        EntityResovler::Entity { key: key.clone(), id }
                                    })
                                    .collect::<Vec<_>>()
                            });
                        for res in endpoint_resolvers {
                            let EntityResovler::Entity { id, key } = res else {
                                continue;
                            };
                            // If part of the key we can't be provided by this resolver.
                            if key.find_field(federated_id).is_none() {
                                resolver_ids.push(*id);
                            }
                        }
                    }
                    SubgraphId::Virtual(_) | SubgraphId::Introspection => (),
                }
            }

            let directive_ids = self.push_directives(config, field_schema_location, &field.directives)?;
            resolver_ids.extend(
                directive_ids
                    .iter()
                    .filter_map(|id| id.as_extension())
                    .filter_map(|id| {
                        if exists_in_subgraph_ids.contains(&self.graph[id].subgraph_id) {
                            self.graph
                                .resolver_definitions
                                .push(ResolverDefinitionRecord::FieldResolverExtension(
                                    FieldResolverExtensionDefinitionRecord { directive_id: id },
                                ));
                            Some(ResolverDefinitionId::from(self.graph.resolver_definitions.len() - 1))
                        } else {
                            None
                        }
                    }),
            );

            self.graph.field_definitions.push(FieldDefinitionRecord {
                name_id: field.name.into(),
                description_id: field.description.map(Into::into),
                parent_entity_id,
                subgraph_type_records,
                ty_record: self.ctx.convert_type(field.r#type),
                exists_in_subgraph_ids,
                resolver_ids,
                provides_records,
                requires_records,
                argument_ids: IdRange::from_start_and_length(field.arguments),
                directive_ids,
            })
        }

        self.graphql_federated_entity_resolvers = graphql_federated_entity_resolvers;
        Ok(())
    }

    fn finalize(self) -> Result<(Graph, IntrospectionMetadata), BuildError> {
        let Self {
            ctx,
            field_sets,
            required_scopes,
            mut graph,
            ..
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

        // Any field or input_value having an inaccessible type is marked as inaccessible.
        // Composition should ensure all of this is consistent, but we ensure it.
        fn is_definition_inaccessible(graph: &Graph, definition_id: DefinitionId) -> bool {
            match definition_id {
                DefinitionId::Scalar(id) => graph.inaccessible_scalar_definitions[id],
                DefinitionId::Object(id) => graph.inaccessible_object_definitions[id],
                DefinitionId::Interface(id) => graph.inaccessible_interface_definitions[id],
                DefinitionId::Union(id) => graph.inaccessible_union_definitions[id],
                DefinitionId::Enum(id) => graph.inaccessible_enum_definitions[id],
                DefinitionId::InputObject(id) => graph.inaccessible_input_object_definitions[id],
            }
        }

        for (ix, field) in graph.field_definitions.iter().enumerate() {
            if is_definition_inaccessible(&graph, field.ty_record.definition_id) {
                graph.inaccessible_field_definitions.set(ix.into(), true);
            }
        }

        for (ix, input_value) in graph.input_value_definitions.iter().enumerate() {
            if is_definition_inaccessible(&graph, input_value.ty_record.definition_id) {
                graph.inaccessible_input_value_definitions.set(ix.into(), true);
            }
        }

        Ok((graph, introspection))
    }

    fn push_resolver(&mut self, resolver: ResolverDefinitionRecord) -> ResolverDefinitionId {
        let resolver_id = ResolverDefinitionId::from(self.graph.resolver_definitions.len());
        self.graph.resolver_definitions.push(resolver);
        resolver_id
    }

    fn push_directives<'d>(
        &mut self,
        config: &Config,
        schema_location: SchemaLocation,
        directives: impl IntoIterator<Item = &'d federated_graph::Directive>,
    ) -> Result<Vec<TypeSystemDirectiveId>, BuildError> {
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
                            .map(convert_input_value_set)
                            .unwrap_or_default(),
                        fields_id: authorized
                            .fields
                            .as_ref()
                            .map(|field_set| self.field_sets.push(schema_location, field_set.clone())),
                        node_id: authorized
                            .node
                            .as_ref()
                            .map(|field_set| self.field_sets.push(schema_location, field_set.clone())),
                        metadata_id: authorized.metadata.clone().map(|value| {
                            let value = self.graph.input_values.ingest_arbitrary_value(self.ctx, value);
                            self.graph.input_values.push_value(value)
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
                        slicing_argument_ids: slicing_arguments.iter().copied().map(Into::into).collect(),
                        sized_field_ids: sized_fields.iter().copied().map(Into::into).collect(),
                        require_one_slicing_argument: *require_one_slicing_argument,
                    });
                    TypeSystemDirectiveId::ListSize(list_size_id)
                }
                federated_graph::Directive::ExtensionDirective(federated_graph::ExtensionDirective {
                    subgraph_id,
                    extension_id,
                    name,
                    arguments,
                }) => {
                    let extension_id = &config.graph[*extension_id].id;
                    let Some(id) = self.ctx.extension_catalog.find_compatible_extension(extension_id) else {
                        return Err(BuildError::UnknownDirectiveExtension {
                            name: self.ctx.strings[StringId::from(*name)].to_string(),
                            id: extension_id.clone(),
                        });
                    };

                    self.graph.extension_directives.push(ExtensionDirectiveRecord {
                        subgraph_id: self.sources[*subgraph_id],
                        extension_id: id,
                        name_id: (*name).into(),
                        arguments_id: arguments.as_ref().map(|arguments| {
                            let arguments = arguments.iter().map(|arg| (arg.name, arg.value.clone())).collect();
                            let value = self
                                .graph
                                .input_values
                                .ingest_arbitrary_value(self.ctx, federated_graph::Value::Object(arguments));
                            self.graph.input_values.push_value(value)
                        }),
                    });
                    let id = (self.graph.extension_directives.len() - 1).into();
                    TypeSystemDirectiveId::Extension(id)
                }
                federated_graph::Directive::Other { .. }
                | federated_graph::Directive::Inaccessible
                | federated_graph::Directive::Policy(_)
                | federated_graph::Directive::JoinField(_)
                | federated_graph::Directive::JoinGraph(_)
                | federated_graph::Directive::JoinType(_)
                | federated_graph::Directive::JoinUnionMember(_)
                | federated_graph::Directive::JoinImplements(_) => continue,
            };

            directive_ids.push(id);
        }

        Ok(directive_ids)
    }
}

fn convert_input_value_set(input_value_set: &federated_graph::InputValueDefinitionSet) -> InputValueSet {
    input_value_set
        .iter()
        .map(|item| InputValueSetSelection {
            id: item.input_value_definition.into(),
            subselection: convert_input_value_set(&item.subselection),
        })
        .collect()
}

fn has_inaccessible(directives: &[federated_graph::Directive]) -> bool {
    directives
        .iter()
        .any(|dir| matches!(dir, federated_graph::Directive::Inaccessible))
}
