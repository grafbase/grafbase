use builder::SchemaLocation;

use crate::BuildError;

use super::*;

impl<'a> Context<'a> {
    pub(super) fn into_graph_context(
        self,
    ) -> Result<(GraphContext<'a>, Vec<SchemaLocation>, IntrospectionMetadata), BuildError> {
        let federated_graph = self.federated_graph;
        let graph = Graph {
            description_id: None,
            root_operation_types_record: RootOperationTypesRecord {
                query_id: federated_graph.root_operation_types.query.into(),
                mutation_id: federated_graph.root_operation_types.mutation.map(Into::into),
                subscription_id: federated_graph.root_operation_types.subscription.map(Into::into),
            },
            object_definitions: Vec::with_capacity(federated_graph.objects.len()),
            inaccessible_object_definitions: BitSet::with_capacity(federated_graph.objects.len()),
            interface_definitions: Vec::with_capacity(federated_graph.interfaces.len()),
            inaccessible_interface_definitions: BitSet::with_capacity(federated_graph.interfaces.len()),
            interface_has_inaccessible_implementor: BitSet::with_capacity(federated_graph.interfaces.len()),
            union_definitions: Vec::with_capacity(federated_graph.unions.len()),
            inaccessible_union_definitions: BitSet::with_capacity(federated_graph.unions.len()),
            union_has_inaccessible_member: BitSet::with_capacity(federated_graph.unions.len()),
            scalar_definitions: Vec::with_capacity(federated_graph.scalar_definitions.len()),
            inaccessible_scalar_definitions: BitSet::with_capacity(federated_graph.scalar_definitions.len()),
            enum_definitions: Vec::with_capacity(federated_graph.enum_definitions.len()),
            inaccessible_enum_definitions: BitSet::with_capacity(federated_graph.enum_definitions.len()),
            enum_values: Vec::with_capacity(federated_graph.enum_values.len()),
            inaccessible_enum_values: BitSet::with_capacity(federated_graph.enum_values.len()),
            input_object_definitions: Vec::with_capacity(federated_graph.input_objects.len()),
            inaccessible_input_object_definitions: BitSet::with_capacity(federated_graph.input_objects.len()),
            field_definitions: Vec::with_capacity(federated_graph.fields.len()),
            inaccessible_field_definitions: BitSet::with_capacity(federated_graph.fields.len()),
            input_value_definitions: Vec::with_capacity(federated_graph.input_value_definitions.len()),
            inaccessible_input_value_definitions: BitSet::with_capacity(federated_graph.input_value_definitions.len()),
            // Initialized in the relevant functions as there is no obvious default.
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
            extension_directive_arguments: Vec::new(),
        };
        let scalar_mapping =
            FxHashMap::with_capacity_and_hasher(federated_graph.scalar_definitions.len(), Default::default());
        let enum_mapping =
            FxHashMap::with_capacity_and_hasher(federated_graph.scalar_definitions.len(), Default::default());
        let input_value_mapping =
            FxHashMap::with_capacity_and_hasher(federated_graph.input_value_definitions.len(), Default::default());

        let mut ctx = GraphContext {
            graph,
            scalar_mapping,
            enum_mapping,
            input_value_mapping,
            deduplicated_fields: Default::default(),
            field_arguments: Default::default(),
            required_scopes: Default::default(),
            graphql_federated_entity_resolvers: Default::default(),
            value_path: Default::default(),
            input_fields_buffer_pool: Default::default(),
            virtual_subgraph_to_selection_set_resolver: vec![None; self.subgraphs.virtual_subgraphs.len()],
            ctx: self,
        };

        let mut schema_locations = Vec::with_capacity(
            federated_graph.fields.len()
                + federated_graph.objects.len()
                + federated_graph.interfaces.len()
                + federated_graph.unions.len()
                + federated_graph.scalar_definitions.len()
                + federated_graph.enum_definitions.len()
                + federated_graph.enum_values.len()
                + federated_graph.input_objects.len()
                + federated_graph.input_value_definitions.len(),
        );
        let mut federated_default_values = Vec::with_capacity(federated_graph.input_value_definitions.len() >> 3);

        for scalar in federated_graph.iter_scalar_definitions() {
            if scalar.namespace.is_some() {
                continue;
            }

            let id = ScalarDefinitionId::from(ctx.graph.scalar_definitions.len());
            ctx.scalar_mapping.insert(scalar.id(), id);
            schema_locations.push(SchemaLocation::Scalar(id, scalar.id()));

            let name_id = ctx.get_or_insert_str(scalar.name);
            let description_id = scalar.description.map(|id| ctx.get_or_insert_str(id));
            ctx.graph.scalar_definitions.push(ScalarDefinitionRecord {
                name_id,
                ty: ScalarType::from_scalar_name(&ctx.strings[name_id]),
                description_id,
                specified_by_url_id: None,
                directive_ids: Default::default(),
                // Added afterwards
                exists_in_subgraph_ids: Vec::new(),
            })
        }

        for enm in federated_graph.iter_enum_definitions() {
            if enm.namespace.is_some() {
                continue;
            }

            let enum_id = EnumDefinitionId::from(ctx.graph.enum_definitions.len());
            let start = ctx.graph.enum_values.len();
            for enum_value in federated_graph.iter_enum_values(enm.id()) {
                let id = EnumValueId::from(ctx.graph.enum_values.len());
                schema_locations.push(SchemaLocation::EnumValue(enum_id, id, enum_value.id()));

                let name_id = ctx.get_or_insert_str(enum_value.value);
                let description_id = enum_value.description.map(|id| ctx.get_or_insert_str(id));
                ctx.graph.enum_values.push(EnumValueRecord {
                    name_id,
                    description_id,
                    parent_enum_id: enum_id,
                    // Added afterwards
                    directive_ids: Default::default(),
                });
            }
            let value_ids = (start..ctx.graph.enum_values.len()).into();

            ctx.enum_mapping.insert(enm.id(), enum_id);
            schema_locations.push(SchemaLocation::Enum(enum_id, enm.id()));

            let name_id = ctx.get_or_insert_str(enm.name);
            let description_id = enm.description.map(|id| ctx.get_or_insert_str(id));
            ctx.graph.enum_definitions.push(EnumDefinitionRecord {
                name_id,
                description_id,
                value_ids,
                // Added afterwards
                directive_ids: Default::default(),
                exists_in_subgraph_ids: Vec::new(),
            });
        }

        for (ix, input_object) in federated_graph.input_objects.iter().enumerate() {
            let input_object_id = InputObjectDefinitionId::from(ctx.graph.input_object_definitions.len());

            let start = ctx.graph.input_value_definitions.len();
            let (federated_id_start, length) = input_object.fields;
            for offset in 0..length {
                let federated_id =
                    federated_graph::InputValueDefinitionId::from(usize::from(federated_id_start) + offset);
                let input_value = &federated_graph[federated_id];
                let id = InputValueDefinitionId::from(ctx.graph.input_value_definitions.len());
                schema_locations.push(SchemaLocation::InputFieldDefinition(input_object_id, id, federated_id));
                ctx.input_value_mapping.insert(federated_id, id);

                let name_id = ctx.get_or_insert_str(input_value.name);
                let description_id = input_value.description.map(|id| ctx.get_or_insert_str(id));
                if let Some(value) = &input_value.default {
                    federated_default_values.push((id, value));
                }
                ctx.graph.input_value_definitions.push(InputValueDefinitionRecord {
                    name_id,
                    description_id,
                    ty_record: ctx.convert_type(input_value.r#type),
                    // Added afterwards
                    default_value_id: None,
                    directive_ids: Default::default(),
                });
            }
            let input_field_ids = (start..ctx.graph.input_value_definitions.len()).into();

            schema_locations.push(SchemaLocation::InputObject(input_object_id, ix.into()));
            let name_id = ctx.get_or_insert_str(input_object.name);
            let description_id = input_object.description.map(|id| ctx.get_or_insert_str(id));
            ctx.graph.input_object_definitions.push(InputObjectDefinitionRecord {
                name_id,
                description_id,
                input_field_ids,
                // Added afterwards
                directive_ids: Default::default(),
                exists_in_subgraph_ids: Vec::new(),
            });
        }

        for (ix, object) in federated_graph.objects.iter().enumerate() {
            let id = ObjectDefinitionId::from(ctx.graph.object_definitions.len());
            schema_locations.push(SchemaLocation::Object(id, ix.into()));

            let name_id = ctx.get_or_insert_str(object.name);
            let description_id = object.description.map(|id| ctx.get_or_insert_str(id));
            ctx.graph.object_definitions.push(ObjectDefinitionRecord {
                name_id,
                description_id,
                interface_ids: object.implements_interfaces.iter().copied().map(Into::into).collect(),
                field_ids: IdRange::from(object.fields.clone()),
                directive_ids: Default::default(),
                join_implement_records: Default::default(),
                exists_in_subgraph_ids: Default::default(),
            });
        }

        for (ix, union) in federated_graph.unions.iter().enumerate() {
            let id = UnionDefinitionId::from(ctx.graph.union_definitions.len());
            schema_locations.push(SchemaLocation::Union(id, ix.into()));

            let possible_type_ids = union
                .members
                .iter()
                .copied()
                .map(ObjectDefinitionId::from)
                .collect::<Vec<_>>();

            let name_id = ctx.get_or_insert_str(union.name);
            let description_id = union.description.map(|id| ctx.get_or_insert_str(id));
            ctx.graph.union_definitions.push(UnionDefinitionRecord {
                name_id,
                description_id,
                possible_type_ids,
                // Added at the end.
                possible_types_ordered_by_typename_ids: Vec::new(),
                directive_ids: Default::default(),
                join_member_records: Vec::new(),
                not_fully_implemented_in_ids: Vec::new(),
                exists_in_subgraph_ids: Vec::new(),
            });
        }

        for (ix, interface) in federated_graph.interfaces.iter().enumerate() {
            let id = InterfaceDefinitionId::from(ctx.graph.interface_definitions.len());

            schema_locations.push(SchemaLocation::Interface(id, ix.into()));

            let name_id = ctx.get_or_insert_str(interface.name);
            let description_id = interface.description.map(|id| ctx.get_or_insert_str(id));
            ctx.graph.interface_definitions.push(InterfaceDefinitionRecord {
                name_id,
                description_id,
                interface_ids: interface
                    .implements_interfaces
                    .iter()
                    .copied()
                    .map(Into::into)
                    .collect(),
                field_ids: IdRange::from(interface.fields.clone()),
                // Added at the end.
                possible_type_ids: Vec::new(),
                possible_types_ordered_by_typename_ids: Vec::new(),
                not_fully_implemented_in_ids: Vec::new(),
                directive_ids: Default::default(),
                exists_in_subgraph_ids: Default::default(),
                is_interface_object_in_ids: Default::default(),
            });
        }

        // Adding all implementations of an interface, used during introspection.
        for object_id in (0..ctx.graph.object_definitions.len()).map(ObjectDefinitionId::from) {
            for interface_id in ctx.graph[object_id].interface_ids.clone() {
                ctx.graph[interface_id].possible_type_ids.push(object_id);
                if ctx.graph.inaccessible_object_definitions[object_id] {
                    ctx.graph.interface_has_inaccessible_implementor.set(interface_id, true);
                }
            }
        }

        for (ix, field) in federated_graph.fields.iter().enumerate() {
            let field_id = FieldDefinitionId::from(ctx.graph.field_definitions.len());

            let start = ctx.graph.input_value_definitions.len();
            let (federated_id_start, length) = field.arguments;
            for offset in 0..length {
                let federated_id =
                    federated_graph::InputValueDefinitionId::from(usize::from(federated_id_start) + offset);
                let input_value = &federated_graph[federated_id];
                let id = InputValueDefinitionId::from(ctx.graph.input_value_definitions.len());
                schema_locations.push(SchemaLocation::ArgumentDefinition(field_id, id, federated_id));
                ctx.input_value_mapping.insert(federated_id, id);

                let name_id = ctx.get_or_insert_str(input_value.name);
                let description_id = input_value.description.map(|id| ctx.get_or_insert_str(id));
                if let Some(value) = &input_value.default {
                    federated_default_values.push((id, value));
                }
                ctx.graph.input_value_definitions.push(InputValueDefinitionRecord {
                    name_id,
                    description_id,
                    ty_record: ctx.convert_type(input_value.r#type),
                    // Added afterwards
                    default_value_id: None,
                    directive_ids: Default::default(),
                });
            }
            let argument_ids = (start..ctx.graph.input_value_definitions.len()).into();

            schema_locations.push(SchemaLocation::FieldDefinition(field_id, ix.into()));
            let name_id = ctx.get_or_insert_str(field.name);
            let description_id = field.description.map(|id| ctx.get_or_insert_str(id));
            ctx.graph.field_definitions.push(FieldDefinitionRecord {
                name_id,
                description_id,
                parent_entity_id: field.parent_entity_id.into(),
                ty_record: ctx.convert_type(field.r#type),
                argument_ids,
                // Added at the end.
                subgraph_type_records: Default::default(),
                exists_in_subgraph_ids: Default::default(),
                resolver_ids: Default::default(),
                provides_records: Default::default(),
                requires_records: Default::default(),
                directive_ids: Default::default(),
            });
        }

        let introspection = ctx.create_introspection_metadata();
        ingest_all_default_values(&mut ctx, federated_default_values)?;
        add_extra_vecs_for_definitions_with_different_ordering(&mut ctx);

        Ok((ctx, schema_locations, introspection))
    }
}

fn add_extra_vecs_for_definitions_with_different_ordering(GraphContext { ctx, graph, .. }: &mut GraphContext<'_>) {
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
            (0..graph.scalar_definitions.len()).map(|id| TypeDefinitionId::Scalar(ScalarDefinitionId::from(id))),
        );
        definitions.extend(
            (0..graph.object_definitions.len()).map(|id| TypeDefinitionId::Object(ObjectDefinitionId::from(id))),
        );
        definitions.extend(
            (0..graph.interface_definitions.len())
                .map(|id| TypeDefinitionId::Interface(InterfaceDefinitionId::from(id))),
        );
        definitions
            .extend((0..graph.union_definitions.len()).map(|id| TypeDefinitionId::Union(UnionDefinitionId::from(id))));
        definitions
            .extend((0..graph.enum_definitions.len()).map(|id| TypeDefinitionId::Enum(EnumDefinitionId::from(id))));
        definitions.extend(
            (0..graph.input_object_definitions.len())
                .map(|id| TypeDefinitionId::InputObject(InputObjectDefinitionId::from(id))),
        );
        definitions.sort_unstable_by_key(|definition| match *definition {
            TypeDefinitionId::Scalar(id) => &ctx.strings[graph[id].name_id],
            TypeDefinitionId::Object(id) => &ctx.strings[graph[id].name_id],
            TypeDefinitionId::Interface(id) => &ctx.strings[graph[id].name_id],
            TypeDefinitionId::Union(id) => &ctx.strings[graph[id].name_id],
            TypeDefinitionId::Enum(id) => &ctx.strings[graph[id].name_id],
            TypeDefinitionId::InputObject(id) => &ctx.strings[graph[id].name_id],
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
}

fn ingest_all_default_values(
    ctx: &mut GraphContext<'_>,
    federated_default_values: Vec<(InputValueDefinitionId, &federated_graph::Value)>,
) -> Result<(), BuildError> {
    for (id, default_value) in federated_default_values {
        ctx.graph[id].default_value_id = Some(ctx.coerce_fed_value(id, default_value.clone()).map_err(|err| {
            BuildError::DefaultValueCoercionError {
                err,
                name: ctx.strings[ctx.graph[id].name_id].to_string(),
            }
        })?);
    }

    Ok(())
}
