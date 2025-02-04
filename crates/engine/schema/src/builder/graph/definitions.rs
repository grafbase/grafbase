use builder::SchemaLocation;

use crate::BuildError;

use super::*;

impl<'a> Context<'a> {
    pub(super) fn into_graph_context(mut self) -> Result<(GraphContext<'a>, Vec<SchemaLocation>), BuildError> {
        let federated_graph = self.federated_graph;
        let mut graph = Graph {
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

        for scalar in federated_graph.iter_scalar_definitions() {
            if scalar.namespace.is_some() {
                continue;
            }

            let id = ScalarDefinitionId::from(graph.scalar_definitions.len());
            self.scalar_mapping.insert(scalar.id(), id);
            schema_locations.push(SchemaLocation::Scalar(id, scalar.id()));

            let name_id = self.strings.get_or_new(&federated_graph[scalar.name]);
            let description_id = scalar.description.map(|id| self.get_or_insert_str(id));
            graph.scalar_definitions.push(ScalarDefinitionRecord {
                name_id,
                ty: ScalarType::from_scalar_name(&self.strings[name_id]),
                description_id,
                specified_by_url_id: None,
                directive_ids: Default::default(),
            })
        }

        for enm in federated_graph.iter_enum_definitions() {
            if enm.namespace.is_some() {
                continue;
            }

            let id = EnumDefinitionId::from(graph.enum_definitions.len());
            self.enum_mapping.insert(enm.id(), id);
            schema_locations.push(SchemaLocation::Enum(id, enm.id()));

            let name_id = self.strings.get_or_new(&federated_graph[enm.name]);
            let description_id = enm.description.map(|id| self.get_or_insert_str(id));
            graph.enum_definitions.push(EnumDefinitionRecord {
                name_id,
                description_id,
                value_ids: IdRange::from_start_and_length(federated_graph.enum_value_range(enm.id())),
                // Added afterwards
                directive_ids: Default::default(),
            })
        }

        // Enum values MUST be after enum definitions as otherwise enums will be empty.
        for (ix, enum_value) in federated_graph.enum_values.iter().enumerate() {
            let id = EnumValueId::from(graph.enum_values.len());
            schema_locations.push(SchemaLocation::EnumValue(id, ix.into()));

            let name_id = self.strings.get_or_new(&federated_graph[enum_value.value]);
            let description_id = enum_value.description.map(|id| self.get_or_insert_str(id));
            graph.enum_values.push(EnumValueRecord {
                name_id,
                description_id,
                // Added afterwards
                directive_ids: Default::default(),
            });
        }

        for (ix, input_object) in federated_graph.input_objects.iter().enumerate() {
            let id = InputObjectDefinitionId::from(graph.input_object_definitions.len());
            schema_locations.push(SchemaLocation::InputObject(id, ix.into()));

            let name_id = self.strings.get_or_new(&federated_graph[input_object.name]);
            let description_id = input_object.description.map(|id| self.get_or_insert_str(id));
            graph.input_object_definitions.push(InputObjectDefinitionRecord {
                name_id,
                description_id,
                input_field_ids: IdRange::from_start_and_length(input_object.fields),
                // Added afterwards
                directive_ids: Default::default(),
            });
        }

        for (ix, input_value) in federated_graph.input_value_definitions.iter().enumerate() {
            let id = InputValueDefinitionId::from(graph.input_value_definitions.len());
            schema_locations.push(SchemaLocation::InputValue(id, ix.into()));

            let name_id = self.strings.get_or_new(&federated_graph[input_value.name]);
            let description_id = input_value.description.map(|id| self.get_or_insert_str(id));
            graph.input_value_definitions.push(InputValueDefinitionRecord {
                name_id,
                description_id,
                ty_record: self.convert_type(input_value.r#type),
                // Added afterwards
                default_value_id: None,
                directive_ids: Default::default(),
            });
        }

        for (ix, object) in federated_graph.objects.iter().enumerate() {
            let id = ObjectDefinitionId::from(graph.object_definitions.len());
            schema_locations.push(SchemaLocation::Object(id, ix.into()));

            let name_id = self.strings.get_or_new(&federated_graph[object.name]);
            let description_id = object.description.map(|id| self.get_or_insert_str(id));
            graph.object_definitions.push(ObjectDefinitionRecord {
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
            let id = UnionDefinitionId::from(graph.union_definitions.len());
            schema_locations.push(SchemaLocation::Union(id, ix.into()));

            let possible_type_ids = union
                .members
                .iter()
                .copied()
                .map(ObjectDefinitionId::from)
                .collect::<Vec<_>>();

            let name_id = self.strings.get_or_new(&federated_graph[union.name]);
            let description_id = union.description.map(|id| self.get_or_insert_str(id));
            graph.union_definitions.push(UnionDefinitionRecord {
                name_id,
                description_id,
                possible_type_ids,
                // Added at the end.
                possible_types_ordered_by_typename_ids: Vec::new(),
                directive_ids: Default::default(),
                join_member_records: Vec::new(),
                not_fully_implemented_in_ids: Vec::new(),
            });
        }

        for (ix, interface) in federated_graph.interfaces.iter().enumerate() {
            let id = InterfaceDefinitionId::from(graph.interface_definitions.len());

            schema_locations.push(SchemaLocation::Interface(id, ix.into()));

            let name_id = self.strings.get_or_new(&federated_graph[interface.name]);
            let description_id = interface.description.map(|id| self.get_or_insert_str(id));
            graph.interface_definitions.push(InterfaceDefinitionRecord {
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
        for object_id in (0..graph.object_definitions.len()).map(ObjectDefinitionId::from) {
            for interface_id in graph[object_id].interface_ids.clone() {
                graph[interface_id].possible_type_ids.push(object_id);
                if graph.inaccessible_object_definitions[object_id] {
                    graph.interface_has_inaccessible_implementor.set(interface_id, true);
                }
            }
        }

        for (ix, field) in federated_graph.fields.iter().enumerate() {
            let id = FieldDefinitionId::from(graph.field_definitions.len());
            schema_locations.push(SchemaLocation::Field(id, ix.into()));
            let name_id = self.strings.get_or_new(&federated_graph[field.name]);
            let description_id = field.description.map(|id| self.get_or_insert_str(id));
            graph.field_definitions.push(FieldDefinitionRecord {
                name_id,
                description_id,
                parent_entity_id: field.parent_entity_id.into(),
                ty_record: self.convert_type(field.r#type),
                argument_ids: IdRange::from_start_and_length(field.arguments),
                // Added at the end.
                subgraph_type_records: Default::default(),
                exists_in_subgraph_ids: Default::default(),
                resolver_ids: Default::default(),
                provides_records: Default::default(),
                requires_records: Default::default(),
                directive_ids: Default::default(),
            });
        }

        let ctx = GraphContext {
            ctx: self,
            graph,
            deduplicated_fields: Default::default(),
            field_arguments: Default::default(),
            required_scopes: Default::default(),
            graphql_federated_entity_resolvers: Default::default(),
            value_path: Default::default(),
            input_fields_buffer_pool: Default::default(),
        };

        Ok((ctx, schema_locations))
    }
}
