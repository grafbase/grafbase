use std::{collections::HashMap, hash::BuildHasherDefault};

use fxhash::FxHasher32;

use crate::{BuildError, builder::sdl};

use super::*;

const BUILTIN_SCALARS: [&str; 5] = ["String", "ID", "Boolean", "Int", "Float"];

struct Ingester<'a> {
    sdl: &'a sdl::Sdl<'a>,
    builder: GraphBuilder<'a>,
    schema_locations: Vec<SchemaLocation<'a>>,
    default_values: HashMap<InputValueDefinitionId, sdl::ConstValue<'a>, BuildHasherDefault<FxHasher32>>,
    root_query_type_name: &'a str,
}

pub(crate) fn ingest_definitions(
    ctx: BuildContext<'_>,
) -> Result<(GraphBuilder<'_>, Vec<SchemaLocation<'_>>, IntrospectionSubgraph), BuildError> {
    let sdl = ctx.sdl;
    let graph = Graph {
        description_id: None,
        root_operation_types_record: RootOperationTypesRecord {
            query_id: ObjectDefinitionId::from(u32::MAX - 1),
            mutation_id: None,
            subscription_id: None,
        },
        object_definitions: Vec::with_capacity(sdl.object_count),
        inaccessible_object_definitions: BitSet::new(),
        interface_definitions: Vec::with_capacity(sdl.interface_count),
        inaccessible_interface_definitions: BitSet::new(),
        interface_has_inaccessible_implementor: BitSet::new(),
        union_definitions: Vec::with_capacity(sdl.union_count),
        inaccessible_union_definitions: BitSet::new(),
        union_has_inaccessible_member: BitSet::new(),
        scalar_definitions: Vec::with_capacity(sdl.scalar_count),
        inaccessible_scalar_definitions: BitSet::new(),
        enum_definitions: Vec::with_capacity(sdl.enum_count),
        inaccessible_enum_definitions: BitSet::new(),
        enum_values: Vec::with_capacity(sdl.enum_count),
        inaccessible_enum_values: BitSet::new(),
        input_object_definitions: Vec::with_capacity(sdl.input_object_count),
        inaccessible_input_object_definitions: BitSet::new(),
        // Minimal size
        field_definitions: Vec::with_capacity(sdl.object_count + sdl.interface_count),
        inaccessible_field_definitions: BitSet::new(),
        input_value_definitions: Vec::with_capacity(sdl.input_object_count),
        inaccessible_input_value_definitions: BitSet::new(),
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
        templates: Vec::new(),
    };

    let mut builder = GraphBuilder {
        graph,
        type_definitions: RapidHashMap::with_capacity_and_hasher(sdl.type_definitions.len(), Default::default()),
        deduplicated_fields: Default::default(),
        required_scopes: Default::default(),
        entity_resolvers: Default::default(),
        value_path: Default::default(),
        input_fields_buffer_pool: Default::default(),
        virtual_subgraph_to_selection_set_resolver: vec![None; ctx.subgraphs.virtual_subgraphs.len()],
        ctx,
    };

    for builtin in BUILTIN_SCALARS {
        let id = ScalarDefinitionId::from(builder.graph.scalar_definitions.len());
        let def = ScalarDefinitionRecord {
            name_id: builder.ingest_str(builtin),
            ty: ScalarType::from_scalar_name(builtin),
            description_id: None,
            specified_by_url_id: None,
            directive_ids: Default::default(),
            exists_in_subgraph_ids: builder.subgraphs.all.clone(),
        };
        builder.type_definitions.insert(builtin, id.into());
        builder.graph.scalar_definitions.push(def);
    }

    let mut ingester = Ingester {
        sdl,
        schema_locations: Vec::with_capacity(sdl.type_definitions.len()),
        root_query_type_name: sdl.root_types.query.unwrap_or("Query"),
        default_values: HashMap::with_capacity_and_hasher(
            builder.graph.input_value_definitions.len() >> 3,
            Default::default(),
        ),
        builder,
    };

    for ty in sdl.type_definitions.iter().copied() {
        let id = match ty {
            sdl::TypeDefinition::Scalar(scalar) => ingester.ingest_scalar(scalar).into(),
            sdl::TypeDefinition::Enum(enm) => ingester.ingest_enum(enm).into(),
            sdl::TypeDefinition::InputObject(input_object) => ingester.ingest_input_object(input_object).into(),
            sdl::TypeDefinition::Object(object) => ingester.ingest_object(object)?.into(),
            sdl::TypeDefinition::Union(union) => ingester.ingest_union(union).into(),
            sdl::TypeDefinition::Interface(interface) => ingester.ingest_interface(interface)?.into(),
        };
        ingester.builder.type_definitions.insert(ty.name(), id);
    }

    ingester.add_root_types()?;
    ingester.add_type_references()?;
    ingester.add_default_values()?;

    let Ingester {
        mut builder,
        schema_locations,
        ..
    } = ingester;

    let introspection = builder.create_introspection_subgraph();
    add_extra_vecs_for_definitions_with_different_ordering(&mut builder);
    create_inaccessible_bitsets(&mut builder.graph);

    Ok((builder, schema_locations, introspection))
}

impl<'a> Ingester<'a> {
    fn ingest_fields(
        &mut self,
        parent: sdl::TypeDefinition<'a>,
        parent_entity_id: EntityDefinitionId,
        fields: impl Iterator<Item = sdl::FieldDefinition<'a>>,
    ) -> Result<IdRange<FieldDefinitionId>, BuildError> {
        let fields_start = self.builder.graph.field_definitions.len();
        for field in fields {
            let field_id = FieldDefinitionId::from(self.builder.graph.field_definitions.len());

            let args_start = self.builder.graph.input_value_definitions.len();
            for argument in field.arguments() {
                let id = InputValueDefinitionId::from(self.builder.graph.input_value_definitions.len());
                self.schema_locations
                    .push(SchemaLocation::ArgumentDefinition(field_id, id, argument));

                if let Some(default_value) = argument.default_value() {
                    self.default_values.insert(id, default_value);
                }

                let name_id = self.ingest_str(argument.name());
                let description_id = argument.description().map(|desc| self.ingest_str(desc.to_cow()));
                self.builder
                    .graph
                    .input_value_definitions
                    .push(InputValueDefinitionRecord {
                        name_id,
                        description_id,
                        ty_record: TypeRecord {
                            wrapping: sdl::convert_wrappers(argument.ty().wrappers()),
                            // Replaced afterwards
                            definition_id: TypeDefinitionId::Object((u32::MAX - 1).into()),
                        },
                        // Added afterwards
                        default_value_id: None,
                        directive_ids: Default::default(),
                    });
            }
            let argument_ids = (args_start..self.builder.graph.input_value_definitions.len()).into();

            self.schema_locations
                .push(SchemaLocation::FieldDefinition(field_id, parent, field));
            let name_id = self.ingest_str(field.name());

            if self.builder.graph.field_definitions[fields_start..]
                .iter()
                .any(|field| field.name_id == name_id)
            {
                return Err(BuildError::GraphQLSchemaValidationError(format!(
                    "Duplicate field {}.{}",
                    parent.name(),
                    field.name(),
                )));
            }

            let description_id = field.description().map(|desc| self.ingest_str(desc.to_cow()));
            self.builder.graph.field_definitions.push(FieldDefinitionRecord {
                name_id,
                description_id,
                parent_entity_id,
                ty_record: TypeRecord {
                    wrapping: sdl::convert_wrappers(field.ty().wrappers()),
                    // Replaced afterwards
                    definition_id: TypeDefinitionId::Object((u32::MAX - 1).into()),
                },
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
        let end = self.builder.graph.field_definitions.len();
        Ok((fields_start..end).into())
    }

    fn ingest_str(&mut self, s: impl AsRef<str>) -> StringId {
        self.builder.ingest_str(s.as_ref())
    }

    fn ingest_scalar(&mut self, scalar: sdl::ScalarDefinition<'a>) -> ScalarDefinitionId {
        if BUILTIN_SCALARS.contains(&scalar.name()) {
            return self
                .builder
                .type_definitions
                .get(scalar.name())
                .unwrap()
                .as_scalar()
                .unwrap();
        }

        let id = ScalarDefinitionId::from(self.builder.graph.scalar_definitions.len());
        self.builder.type_definitions.insert(scalar.name(), id.into());
        self.schema_locations.push(SchemaLocation::Scalar(id, scalar));

        let name_id = self.ingest_str(scalar.name());
        let description_id = scalar.description().map(|desc| self.ingest_str(desc.to_cow()));

        self.builder.graph.scalar_definitions.push(ScalarDefinitionRecord {
            name_id,
            ty: ScalarType::from_scalar_name(&self.builder[name_id]),
            description_id,
            specified_by_url_id: None,
            directive_ids: Default::default(),
            exists_in_subgraph_ids: Vec::new(),
        });
        id
    }

    fn ingest_enum(&mut self, enm: sdl::EnumDefinition<'a>) -> EnumDefinitionId {
        let enum_id = EnumDefinitionId::from(self.builder.graph.enum_definitions.len());
        self.builder.type_definitions.insert(enm.name(), enum_id.into());
        self.schema_locations.push(SchemaLocation::Enum(enum_id, enm));

        let start = self.builder.graph.enum_values.len();

        for enum_value in enm.values() {
            let id = EnumValueId::from(self.builder.graph.enum_values.len());
            self.schema_locations.push(SchemaLocation::EnumValue(id, enum_value));

            let name_id = self.ingest_str(enum_value.value());
            let description_id = enum_value.description().map(|desc| self.ingest_str(desc.to_cow()));
            self.builder.graph.enum_values.push(EnumValueRecord {
                name_id,
                description_id,
                parent_enum_id: enum_id,
                directive_ids: Default::default(),
            });
        }
        let value_ids = (start..self.builder.graph.enum_values.len()).into();

        let name_id = self.ingest_str(enm.name());
        let description_id = enm.description().map(|desc| self.ingest_str(desc.to_cow()));
        self.builder.graph.enum_definitions.push(EnumDefinitionRecord {
            name_id,
            description_id,
            value_ids,
            directive_ids: Default::default(),
            exists_in_subgraph_ids: Vec::new(),
        });
        enum_id
    }

    fn ingest_input_object(&mut self, input_object: sdl::InputObjectDefinition<'a>) -> InputObjectDefinitionId {
        let input_object_id = InputObjectDefinitionId::from(self.builder.graph.input_object_definitions.len());
        self.builder
            .type_definitions
            .insert(input_object.name(), input_object_id.into());

        let start = self.builder.graph.input_value_definitions.len();
        for input_value in input_object.fields() {
            let id = InputValueDefinitionId::from(self.builder.graph.input_value_definitions.len());
            self.schema_locations
                .push(SchemaLocation::InputFieldDefinition(input_object_id, id, input_value));

            if let Some(default_value) = input_value.default_value() {
                self.default_values.insert(id, default_value);
            }

            let name_id = self.ingest_str(input_value.name());
            let description_id = input_value.description().map(|desc| self.ingest_str(desc.to_cow()));
            self.builder
                .graph
                .input_value_definitions
                .push(InputValueDefinitionRecord {
                    name_id,
                    description_id,
                    ty_record: TypeRecord {
                        wrapping: sdl::convert_wrappers(input_value.ty().wrappers()),
                        definition_id: TypeDefinitionId::Object((u32::MAX - 1).into()),
                    },
                    default_value_id: None,
                    directive_ids: Default::default(),
                });
        }
        let input_field_ids = (start..self.builder.graph.input_value_definitions.len()).into();

        self.schema_locations
            .push(SchemaLocation::InputObject(input_object_id, input_object));
        self.builder
            .type_definitions
            .insert(input_object.name(), input_object_id.into());

        let name_id = self.ingest_str(input_object.name());
        let description_id = input_object.description().map(|desc| self.ingest_str(desc.to_cow()));
        self.builder
            .graph
            .input_object_definitions
            .push(InputObjectDefinitionRecord {
                name_id,
                description_id,
                input_field_ids,
                directive_ids: Default::default(),
                exists_in_subgraph_ids: Vec::new(),
            });
        input_object_id
    }

    fn ingest_object(&mut self, object: sdl::ObjectDefinition<'a>) -> Result<ObjectDefinitionId, BuildError> {
        let id = ObjectDefinitionId::from(self.builder.graph.object_definitions.len());
        self.schema_locations.push(SchemaLocation::Object(id, object));
        self.builder.type_definitions.insert(object.name(), id.into());

        let name_id = self.ingest_str(object.name());
        let description_id = object.description().map(|desc| self.ingest_str(desc.to_cow()));
        let mut merged_fields = vec![object.fields()];
        if let Some(extensions) = self.sdl.type_extensions.get(object.name()) {
            for ext in extensions {
                let sdl::TypeDefinition::Object(obj) = ext else {
                    return Err(BuildError::GraphQLSchemaValidationError(format!(
                        "Cannot extend object named '{}' with anything else but an object",
                        object.name()
                    )));
                };
                merged_fields.push(obj.fields());
            }
        }
        let mut field_ids = self.ingest_fields(
            sdl::TypeDefinition::Object(object),
            id.into(),
            merged_fields.into_iter().flatten(),
        )?;
        if object.name() == self.root_query_type_name {
            self.push_query_introspection_fields(id);
            field_ids.end = self.builder.graph.field_definitions.len().into();
        }
        self.builder.graph.object_definitions.push(ObjectDefinitionRecord {
            name_id,
            description_id,
            field_ids,
            // Added later
            interface_ids: Vec::new(),
            directive_ids: Default::default(),
            join_implement_records: Default::default(),
            exists_in_subgraph_ids: Default::default(),
        });
        Ok(id)
    }

    fn ingest_union(&mut self, union: sdl::UnionDefinition<'a>) -> UnionDefinitionId {
        let id = UnionDefinitionId::from(self.builder.graph.union_definitions.len());
        self.schema_locations.push(SchemaLocation::Union(id, union));
        self.builder.type_definitions.insert(union.name(), id.into());

        let name_id = self.ingest_str(union.name());
        let description_id = union.description().map(|desc| self.ingest_str(desc.to_cow()));
        self.builder.graph.union_definitions.push(UnionDefinitionRecord {
            name_id,
            description_id,
            // Added later
            possible_type_ids: Vec::new(),
            possible_types_ordered_by_typename_ids: Vec::new(),
            directive_ids: Default::default(),
            join_member_records: Vec::new(),
            not_fully_implemented_in_ids: Vec::new(),
            exists_in_subgraph_ids: Vec::new(),
        });
        id
    }

    fn ingest_interface(
        &mut self,
        interface: sdl::InterfaceDefinition<'a>,
    ) -> Result<InterfaceDefinitionId, BuildError> {
        let id = InterfaceDefinitionId::from(self.builder.graph.interface_definitions.len());
        self.schema_locations.push(SchemaLocation::Interface(id, interface));
        self.builder.type_definitions.insert(interface.name(), id.into());

        let name_id = self.ingest_str(interface.name());
        let description_id = interface.description().map(|desc| self.ingest_str(desc.to_cow()));
        let field_ids = self.ingest_fields(sdl::TypeDefinition::Interface(interface), id.into(), interface.fields())?;
        self.builder
            .graph
            .interface_definitions
            .push(InterfaceDefinitionRecord {
                name_id,
                description_id,
                field_ids,
                // Added later
                interface_ids: Vec::new(),
                possible_type_ids: Vec::new(),
                possible_types_ordered_by_typename_ids: Vec::new(),
                not_fully_implemented_in_ids: Vec::new(),
                directive_ids: Default::default(),
                exists_in_subgraph_ids: Default::default(),
                is_interface_object_in_ids: Default::default(),
            });
        Ok(id)
    }

    fn add_root_types(&mut self) -> Result<(), BuildError> {
        let query_object_id = self
            .builder
            .get_object_id(self.root_query_type_name)
            .unwrap_or_else(|_| {
                let id = ObjectDefinitionId::from(self.builder.graph.object_definitions.len());

                let start = self.builder.graph.field_definitions.len();
                self.push_query_introspection_fields(id);
                let field_ids = (start..self.builder.graph.field_definitions.len()).into();

                let name_id = self.ingest_str(self.root_query_type_name);
                self.builder.graph.object_definitions.push(ObjectDefinitionRecord {
                    name_id,
                    description_id: None,
                    field_ids,
                    // Added later
                    interface_ids: Vec::new(),
                    directive_ids: Default::default(),
                    join_implement_records: Default::default(),
                    exists_in_subgraph_ids: Default::default(),
                });

                id
            });
        let builder = &mut self.builder;
        builder.graph.root_operation_types_record.query_id = query_object_id;
        builder.graph.root_operation_types_record.mutation_id = self
            .sdl
            .root_types
            .mutation
            .map(|name| builder.get_object_id(name))
            .transpose()?
            .or_else(|| builder.get_object_id("Mutation").ok());
        builder.graph.root_operation_types_record.subscription_id = self
            .sdl
            .root_types
            .subscription
            .map(|name| builder.get_object_id(name))
            .transpose()?
            .or_else(|| builder.get_object_id("Subscription").ok());

        Ok(())
    }

    fn push_query_introspection_fields(&mut self, query_object_id: ObjectDefinitionId) {
        for name in ["__type", "__schema"] {
            let name_id = self.builder.ingest_str(name);
            self.builder.graph.field_definitions.push(FieldDefinitionRecord {
                name_id,
                description_id: None,
                parent_entity_id: query_object_id.into(),
                ty_record: TypeRecord {
                    wrapping: Wrapping::required(),
                    // Replaced afterwards
                    definition_id: TypeDefinitionId::Object((u32::MAX - 1).into()),
                },
                argument_ids: IdRange::empty(),
                subgraph_type_records: Default::default(),
                exists_in_subgraph_ids: vec![SubgraphId::Introspection],
                resolver_ids: Default::default(),
                provides_records: Default::default(),
                requires_records: Default::default(),
                directive_ids: Default::default(),
            });
        }
    }

    fn add_type_references(&mut self) -> Result<(), BuildError> {
        let builder = &mut self.builder;
        for location in self.schema_locations.iter().copied() {
            match location {
                SchemaLocation::FieldDefinition(id, _, field) => {
                    builder.graph[id].ty_record.definition_id = builder.get_type_id(field.ty().name())?;
                }
                SchemaLocation::InputFieldDefinition(_, id, input_value) => {
                    builder.graph[id].ty_record.definition_id = builder.get_type_id(input_value.ty().name())?;
                }
                SchemaLocation::ArgumentDefinition(_, id, input_value) => {
                    builder.graph[id].ty_record.definition_id = builder.get_type_id(input_value.ty().name())?;
                }
                SchemaLocation::Object(id, obj) => {
                    let interface_ids = obj
                        .implements_interfaces()
                        .map(|inf| builder.get_interface_id(inf))
                        .collect::<Result<Vec<_>, _>>()?;
                    for inf_id in &interface_ids {
                        builder.graph[*inf_id].possible_type_ids.push(id);
                    }
                    builder.graph[id].interface_ids = interface_ids;
                }
                SchemaLocation::Interface(id, inf) => {
                    let interface_ids = inf
                        .implements_interfaces()
                        .map(|inf| builder.get_interface_id(inf))
                        .collect::<Result<Vec<_>, _>>()?;
                    builder.graph[id].interface_ids = interface_ids;
                }
                SchemaLocation::Union(id, union) => {
                    let member_ids = union
                        .members()
                        .map(|member| builder.get_object_id(member.name()))
                        .collect::<Result<Vec<_>, _>>()?;
                    builder.graph[id].possible_type_ids = member_ids;
                }

                _ => (),
            }
        }

        Ok(())
    }

    fn add_default_values(&mut self) -> Result<(), BuildError> {
        let mut seen = BitSet::with_capacity(self.builder.graph.input_object_definitions.len());
        let mut input_values_stack = Vec::new();

        while let Some(id) = self.default_values.keys().next().copied() {
            input_values_stack.push(id);
            while let Some(id) = input_values_stack.pop() {
                if let TypeDefinitionId::InputObject(input_object_id) = self.builder.graph[id].ty_record.definition_id {
                    // Nested default values must be treated first.
                    if !seen.put(input_object_id) {
                        // put back our current id.
                        input_values_stack.push(id);
                        input_values_stack.extend(self.builder.graph[input_object_id].input_field_ids);
                        continue;
                    }
                }
                let Some(default_value) = self.default_values.remove(&id) else {
                    continue;
                };
                self.builder.graph[id].default_value_id =
                    Some(self.builder.coerce_input_value(id, default_value).map_err(|err| {
                        BuildError::DefaultValueCoercionError {
                            err,
                            name: self.builder[self.builder.graph[id].name_id].to_string(),
                        }
                    })?);
            }
        }

        Ok(())
    }
}

fn add_extra_vecs_for_definitions_with_different_ordering(GraphBuilder { ctx, graph, .. }: &mut GraphBuilder<'_>) {
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
            TypeDefinitionId::Scalar(id) => &ctx[graph[id].name_id],
            TypeDefinitionId::Object(id) => &ctx[graph[id].name_id],
            TypeDefinitionId::Interface(id) => &ctx[graph[id].name_id],
            TypeDefinitionId::Union(id) => &ctx[graph[id].name_id],
            TypeDefinitionId::Enum(id) => &ctx[graph[id].name_id],
            TypeDefinitionId::InputObject(id) => &ctx[graph[id].name_id],
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
            .sort_unstable_by_key(|id| &ctx[graph[*id].name_id]);
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
            .sort_unstable_by_key(|id| &ctx[graph[*id].name_id]);
    }
    graph.union_definitions = union_definitions;
}

fn create_inaccessible_bitsets(graph: &mut Graph) {
    graph
        .inaccessible_object_definitions
        .grow(graph.object_definitions.len());
    graph.inaccessible_field_definitions.grow(graph.field_definitions.len());

    graph
        .inaccessible_scalar_definitions
        .grow(graph.scalar_definitions.len());

    graph
        .inaccessible_input_object_definitions
        .grow(graph.input_object_definitions.len());
    graph
        .inaccessible_input_value_definitions
        .grow(graph.input_value_definitions.len());

    graph.inaccessible_enum_definitions.grow(graph.enum_definitions.len());
    graph.inaccessible_enum_values.grow(graph.enum_values.len());

    graph.inaccessible_union_definitions.grow(graph.union_definitions.len());
    graph.union_has_inaccessible_member.grow(graph.union_definitions.len());

    graph
        .inaccessible_interface_definitions
        .grow(graph.interface_definitions.len());
    graph
        .interface_has_inaccessible_implementor
        .grow(graph.interface_definitions.len());
}
