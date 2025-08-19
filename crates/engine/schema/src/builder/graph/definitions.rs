use std::{collections::HashMap, hash::BuildHasherDefault};

use fxhash::FxHasher32;

use crate::builder::{Error, sdl};

use super::*;

const BUILTIN_SCALARS: [&str; 5] = ["String", "ID", "Boolean", "Int", "Float"];

struct Ingester<'a> {
    sdl: &'a sdl::Sdl<'a>,
    builder: GraphBuilder<'a>,
    default_values: HashMap<
        InputValueDefinitionId,
        (sdl::InputValueSdlDefinition<'a>, sdl::ConstValue<'a>),
        BuildHasherDefault<FxHasher32>,
    >,
    definitions: GraphDefinitions<'a>,
    root_query_type_name: &'a str,
    errors: Vec<Error>,
}

impl<'a> std::ops::Deref for Ingester<'a> {
    type Target = GraphBuilder<'a>;
    fn deref(&self) -> &Self::Target {
        &self.builder
    }
}

impl std::ops::DerefMut for Ingester<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.builder
    }
}

pub(crate) fn ingest_definitions(ctx: BuildContext<'_>) -> Result<(GraphBuilder<'_>, IntrospectionSubgraph), Vec<Error>> {
    let sdl = ctx.sdl;
    let graph = Graph {
        description_id: None,
        root_operation_types_record: RootOperationTypesRecord {
            query_id: ObjectDefinitionId::from(u32::MAX - 1),
            mutation_id: None,
            subscription_id: None,
        },
        // Inaccessible are initialized at the end.
        inaccessible: Inaccessible {
            object_definitions: BitSet::new(),
            interface_definitions: BitSet::new(),
            field_definitions: BitSet::new(),
            enum_definitions: BitSet::new(),
            enum_values: BitSet::new(),
            union_definitions: BitSet::new(),
            scalar_definitions: BitSet::new(),
            input_object_definitions: BitSet::new(),
            input_value_definitions: BitSet::new(),
        },
        interface_has_inaccessible_implementor: BitSet::new(),
        union_has_inaccessible_member: BitSet::new(),
        object_definitions: Vec::with_capacity(sdl.object_count),
        interface_definitions: Vec::with_capacity(sdl.interface_count),
        union_definitions: Vec::with_capacity(sdl.union_count),
        scalar_definitions: Vec::with_capacity(sdl.scalar_count),
        enum_definitions: Vec::with_capacity(sdl.enum_count),
        enum_values: Vec::with_capacity(sdl.enum_count),
        input_object_definitions: Vec::with_capacity(sdl.input_object_count),
        // Minimal size
        field_definitions: Vec::with_capacity(sdl.object_count + sdl.interface_count),
        input_value_definitions: Vec::with_capacity(sdl.input_object_count),
        // Initialized in the relevant functions as there is no obvious default.
        resolver_definitions: Vec::new(),
        type_definitions_ordered_by_name: Vec::new(),
        input_values: Default::default(),
        cost_directives: Vec::new(),
        list_size_directives: Vec::new(),
        extension_directives: Vec::new(),
        extension_directive_arguments: Vec::new(),
        templates: Vec::new(),
        lookup_resolver_definitions: Vec::new(),
        derive_definitions: Vec::new(),
    };

    let builder = GraphBuilder {
        graph,
        selections: Default::default(),
        value_path: Default::default(),
        input_fields_buffer_pool: Default::default(),
        root_object_ids: Vec::new(),
        virtual_subgraph_to_selection_set_resolver: vec![None; ctx.subgraphs.virtual_subgraphs.len()],
        // Added at the end
        definitions: Default::default(),
        ctx,
    };

    let mut ingester = Ingester {
        sdl,
        root_query_type_name: sdl.root_types.query.unwrap_or("Query"),
        default_values: HashMap::with_capacity_and_hasher(
            builder.graph.input_value_definitions.len() >> 3,
            Default::default(),
        ),
        definitions: GraphDefinitions {
            type_name_to_id: RapidHashMap::with_capacity_and_hasher(sdl.type_definitions.len(), Default::default()),
            site_id_to_sdl: HashMap::with_capacity_and_hasher(sdl.type_definitions.len() << 1, Default::default()),
        },
        builder,
        errors: Vec::new(),
    };

    for builtin in BUILTIN_SCALARS {
        let id = ScalarDefinitionId::from(ingester.graph.scalar_definitions.len());
        let def = ScalarDefinitionRecord {
            name_id: ingester.ingest_str(builtin),
            ty: ScalarType::from_scalar_name(builtin),
            description_id: None,
            specified_by_url_id: None,
            directive_ids: Default::default(),
            exists_in_subgraph_ids: ingester.subgraphs.all.clone(),
        };
        ingester.definitions.type_name_to_id.insert(builtin, id.into());
        ingester.graph.scalar_definitions.push(def);
    }

    for ty in sdl.type_definitions.iter().copied() {
        let id = match ty {
            sdl::TypeDefinition::Scalar(scalar) => Some(ingester.ingest_scalar(scalar).into()),
            sdl::TypeDefinition::Enum(enm) => Some(ingester.ingest_enum(enm).into()),
            sdl::TypeDefinition::InputObject(input_object) => ingester.ingest_input_object(input_object).map(|id| id.into()),
            sdl::TypeDefinition::Object(object) => ingester.ingest_object(object).map(|id| id.into()),
            sdl::TypeDefinition::Union(union) => Some(ingester.ingest_union(union).into()),
            sdl::TypeDefinition::Interface(interface) => ingester.ingest_interface(interface).map(|id| id.into()),
        };
        if let Some(id) = id {
            ingester.definitions.type_name_to_id.insert(ty.name(), id);
        }
    }

    ingester.add_root_types();
    ingester.add_type_references();
    ingester.add_default_values();

    let Ingester {
        mut builder,
        definitions,
        errors,
        ..
    } = ingester;

    let introspection = builder.create_introspection_subgraph();
    add_extra_vecs_for_definitions_with_different_ordering(&mut builder);
    create_inaccessible_bitsets(&mut builder.graph);
    builder.definitions = Rc::new(definitions);

    if !errors.is_empty() {
        Err(errors)
    } else {
        Ok((builder, introspection))
    }
}

impl<'a> Ingester<'a> {
    fn ingest_fields(
        &mut self,
        parent: sdl::TypeDefinition<'a>,
        parent_entity_id: EntityDefinitionId,
        fields: impl Iterator<Item = sdl::FieldDefinition<'a>>,
    ) -> IdRange<FieldDefinitionId> {
        let fields_start = self.graph.field_definitions.len();
        for field in fields {
            let field_id = FieldDefinitionId::from(self.graph.field_definitions.len());

            let args_start = self.graph.input_value_definitions.len();
            for argument in field.arguments() {
                let id = InputValueDefinitionId::from(self.graph.input_value_definitions.len());
                let sdl_def = sdl::ArgumentSdlDefinition {
                    field_id,
                    id,
                    definition: argument,
                };
                self.definitions.site_id_to_sdl.insert(id.into(), sdl_def.into());

                if let Some(default_value) = argument.default_value() {
                    self.default_values.insert(id, (sdl_def.into(), default_value));
                }

                let name_id = self.ingest_str(argument.name());
                let description_id = argument.description().map(|desc| self.ingest_str(desc.to_cow()));
                self.graph.input_value_definitions.push(InputValueDefinitionRecord {
                    name_id,
                    description_id,
                    parent_id: field_id.into(),
                    ty_record: TypeRecord {
                        wrapping: sdl::convert_wrappers(argument.ty().wrappers()),
                        // Replaced afterwards
                        definition_id: TypeDefinitionId::Object((u32::MAX - 1).into()),
                    },
                    // Added afterwards
                    default_value_id: None,
                    directive_ids: Default::default(),
                    is_internal_in_id: None,
                });
            }
            let argument_ids = (args_start..self.graph.input_value_definitions.len()).into();

            self.definitions.site_id_to_sdl.insert(
                field_id.into(),
                sdl::FieldSdlDefinition {
                    id: field_id,
                    parent,
                    definition: field,
                }
                .into(),
            );
            let name_id = self.ingest_str(field.name());

            if self.graph.field_definitions[fields_start..]
                .iter()
                .any(|field| field.name_id == name_id)
            {
                self.errors.push(Error::new(format!("Duplicate field {}.{}", parent.name(), field.name())).span(field.span()));
                continue;
            }

            let description_id = field.description().map(|desc| self.ingest_str(desc.to_cow()));
            self.graph.field_definitions.push(FieldDefinitionRecord {
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
                derive_ids: Default::default(),
            });
        }
        let end = self.graph.field_definitions.len();
        (fields_start..end).into()
    }

    fn ingest_scalar(&mut self, scalar: sdl::ScalarDefinition<'a>) -> ScalarDefinitionId {
        if BUILTIN_SCALARS.contains(&scalar.name()) {
            return self
                .definitions
                .type_name_to_id
                .get(scalar.name())
                .unwrap()
                .as_scalar()
                .unwrap();
        }

        let id = ScalarDefinitionId::from(self.graph.scalar_definitions.len());
        self.definitions.type_name_to_id.insert(scalar.name(), id.into());
        self.definitions
            .site_id_to_sdl
            .insert(id.into(), sdl::ScalarSdlDefinition { id, definition: scalar }.into());

        let name_id = self.ingest_str(scalar.name());
        let description_id = scalar.description().map(|desc| self.ingest_str(desc.to_cow()));

        let ty = ScalarType::from_scalar_name(&self[name_id]);
        self.graph.scalar_definitions.push(ScalarDefinitionRecord {
            name_id,
            ty,
            description_id,
            specified_by_url_id: None,
            directive_ids: Default::default(),
            exists_in_subgraph_ids: Vec::new(),
        });
        id
    }

    fn ingest_enum(&mut self, enm: sdl::EnumDefinition<'a>) -> EnumDefinitionId {
        let enum_id = EnumDefinitionId::from(self.graph.enum_definitions.len());
        self.definitions.type_name_to_id.insert(enm.name(), enum_id.into());
        self.definitions.site_id_to_sdl.insert(
            enum_id.into(),
            sdl::EnumSdlDefinition {
                id: enum_id,
                definition: enm,
            }
            .into(),
        );

        let start = self.graph.enum_values.len();

        for enum_value in enm.values() {
            let id = EnumValueId::from(self.graph.enum_values.len());
            self.definitions.site_id_to_sdl.insert(
                id.into(),
                sdl::EnumValueSdlDefinition {
                    id,
                    definition: enum_value,
                }
                .into(),
            );

            let name_id = self.ingest_str(enum_value.value());
            let description_id = enum_value.description().map(|desc| self.ingest_str(desc.to_cow()));
            self.graph.enum_values.push(EnumValueRecord {
                name_id,
                description_id,
                parent_enum_id: enum_id,
                directive_ids: Default::default(),
            });
        }
        let value_ids = (start..self.graph.enum_values.len()).into();

        let name_id = self.ingest_str(enm.name());
        let description_id = enm.description().map(|desc| self.ingest_str(desc.to_cow()));
        self.graph.enum_definitions.push(EnumDefinitionRecord {
            name_id,
            description_id,
            value_ids,
            directive_ids: Default::default(),
            exists_in_subgraph_ids: Vec::new(),
        });
        enum_id
    }

    fn ingest_input_object(
        &mut self,
        input_object: sdl::InputObjectDefinition<'a>,
    ) -> Option<InputObjectDefinitionId> {
        let input_object_id = InputObjectDefinitionId::from(self.graph.input_object_definitions.len());
        self.definitions
            .type_name_to_id
            .insert(input_object.name(), input_object_id.into());

        let start = self.graph.input_value_definitions.len();
        for input_value in input_object.fields() {
            let id = InputValueDefinitionId::from(self.graph.input_value_definitions.len());
            let sdl_def = sdl::InputFieldSdlDefinition {
                parent_id: input_object_id,
                id,
                definition: input_value,
            };
            self.definitions.site_id_to_sdl.insert(id.into(), sdl_def.into());

            if let Some(default_value) = input_value.default_value() {
                self.default_values.insert(id, (sdl_def.into(), default_value));
            }

            let name_id = self.ingest_str(input_value.name());
            let description_id = input_value.description().map(|desc| self.ingest_str(desc.to_cow()));
            self.graph.input_value_definitions.push(InputValueDefinitionRecord {
                name_id,
                description_id,
                parent_id: input_object_id.into(),
                ty_record: TypeRecord {
                    wrapping: sdl::convert_wrappers(input_value.ty().wrappers()),
                    definition_id: TypeDefinitionId::Object((u32::MAX - 1).into()),
                },
                default_value_id: None,
                directive_ids: Default::default(),
                is_internal_in_id: None,
            });
        }
        let input_field_ids: IdRange<InputValueDefinitionId> = (start..self.graph.input_value_definitions.len()).into();

        // Only directive to be processed immediately as rely on it for default values.
        let is_one_of = if let Some(dir) = input_object.directives().find(|dir| dir.name() == "oneOf") {
            for input_field in &self.graph[input_field_ids] {
                if input_field.ty_record.wrapping.is_non_null() {
                    self.errors.push(Error::new(format!(
                        "@oneOf requires that all input fields of {} must be nullable, {} isn't.",
                        input_object.name(),
                        self[input_field.name_id]
                    )).span(dir.name_span()));
                    return None;
                }
            }
            true
        } else {
            false
        };

        self.definitions.site_id_to_sdl.insert(
            input_object_id.into(),
            sdl::InputObjectSdlDefinition {
                id: input_object_id,
                definition: input_object,
            }
            .into(),
        );
        self.definitions
            .type_name_to_id
            .insert(input_object.name(), input_object_id.into());

        let name_id = self.ingest_str(input_object.name());
        let description_id = input_object.description().map(|desc| self.ingest_str(desc.to_cow()));
        self.graph.input_object_definitions.push(InputObjectDefinitionRecord {
            name_id,
            description_id,
            input_field_ids,
            is_one_of,
            directive_ids: Default::default(),
            exists_in_subgraph_ids: Vec::new(),
        });

        Some(input_object_id)
    }

    fn ingest_object(&mut self, object: sdl::ObjectDefinition<'a>) -> Option<ObjectDefinitionId> {
        let id = ObjectDefinitionId::from(self.graph.object_definitions.len());
        self.definitions
            .site_id_to_sdl
            .insert(id.into(), sdl::ObjectSdlDefinition { id, definition: object }.into());
        self.definitions.type_name_to_id.insert(object.name(), id.into());

        let name_id = self.ingest_str(object.name());
        let description_id = object.description().map(|desc| self.ingest_str(desc.to_cow()));
        let mut merged_fields = vec![object.fields()];
        if let Some(extensions) = self.sdl.type_extensions.get(object.name()) {
            for ext in extensions {
                let sdl::TypeDefinition::Object(obj) = ext else {
                    self.errors.push(Error::new(format!(
                        "Cannot extend object named '{}' with anything else but an object",
                        object.name()
                    )).span(object.span()));
                    return None;
                };
                merged_fields.push(obj.fields());
            }
        }
        let mut field_ids = self.ingest_fields(
            sdl::TypeDefinition::Object(object),
            id.into(),
            merged_fields.into_iter().flatten(),
        );
        if object.name() == self.root_query_type_name {
            self.push_query_introspection_fields(id);
            field_ids.end = self.graph.field_definitions.len().into();
        }
        self.graph.object_definitions.push(ObjectDefinitionRecord {
            name_id,
            description_id,
            field_ids,
            // Added later
            interface_ids: Vec::new(),
            directive_ids: Default::default(),
            join_implement_records: Default::default(),
            exists_in_subgraph_ids: Default::default(),
        });
        Some(id)
    }

    fn ingest_union(&mut self, union: sdl::UnionDefinition<'a>) -> UnionDefinitionId {
        let id = UnionDefinitionId::from(self.graph.union_definitions.len());
        self.definitions
            .site_id_to_sdl
            .insert(id.into(), sdl::UnionSdlDefinition { id, definition: union }.into());
        self.definitions.type_name_to_id.insert(union.name(), id.into());

        let name_id = self.ingest_str(union.name());
        let description_id = union.description().map(|desc| self.ingest_str(desc.to_cow()));
        self.graph.union_definitions.push(UnionDefinitionRecord {
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

    fn ingest_interface(&mut self, interface: sdl::InterfaceDefinition<'a>) -> Option<InterfaceDefinitionId> {
        let id = InterfaceDefinitionId::from(self.graph.interface_definitions.len());
        self.definitions.site_id_to_sdl.insert(
            id.into(),
            sdl::InterfaceSdlDefinition {
                id,
                definition: interface,
            }
            .into(),
        );
        self.definitions.type_name_to_id.insert(interface.name(), id.into());

        let name_id = self.ingest_str(interface.name());
        let description_id = interface.description().map(|desc| self.ingest_str(desc.to_cow()));
        let field_ids = self.ingest_fields(sdl::TypeDefinition::Interface(interface), id.into(), interface.fields());
        self.graph.interface_definitions.push(InterfaceDefinitionRecord {
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
        Some(id)
    }

    fn add_root_types(&mut self) {
        let query_object_id = self
            .definitions
            .get_object_id(self.root_query_type_name, sdl::Span::default())
            .unwrap_or_else(|_| {
                let id = ObjectDefinitionId::from(self.graph.object_definitions.len());

                let start = self.graph.field_definitions.len();
                self.push_query_introspection_fields(id);
                let field_ids = (start..self.graph.field_definitions.len()).into();

                let name_id = self.builder.ingest_str(self.root_query_type_name);
                self.graph.object_definitions.push(ObjectDefinitionRecord {
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
        self.graph.root_operation_types_record.query_id = query_object_id;
        self.graph.root_operation_types_record.mutation_id = self
            .sdl
            .root_types
            .mutation
            .and_then(|name| match self.definitions.get_object_id(name, sdl::Span::default()) {
                Ok(id) => Some(id),
                Err(err) => {
                    self.errors.push(err);
                    None
                }
            })
            .or_else(|| self.definitions.get_object_id("Mutation", sdl::Span::default()).ok());
        self.graph.root_operation_types_record.subscription_id = self
            .sdl
            .root_types
            .subscription
            .and_then(|name| match self.definitions.get_object_id(name, sdl::Span::default()) {
                Ok(id) => Some(id),
                Err(err) => {
                    self.errors.push(err);
                    None
                }
            })
            .or_else(|| {
                self.definitions
                    .get_object_id("Subscription", sdl::Span::default())
                    .ok()
            });

        self.root_object_ids = [
            Some(self.graph.root_operation_types_record.query_id),
            self.graph.root_operation_types_record.mutation_id,
            self.graph.root_operation_types_record.subscription_id,
        ]
        .into_iter()
        .flatten()
        .collect();
    }

    fn push_query_introspection_fields(&mut self, query_object_id: ObjectDefinitionId) {
        for name in ["__type", "__schema"] {
            let name_id = self.ingest_str(name);
            self.graph.field_definitions.push(FieldDefinitionRecord {
                name_id,
                description_id: None,
                parent_entity_id: query_object_id.into(),
                ty_record: TypeRecord {
                    wrapping: Wrapping::default().non_null(),
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
                derive_ids: Default::default(),
            });
        }
    }

    fn add_type_references(&mut self) {
        let builder = &mut self.builder;
        
        for def in self.definitions.site_id_to_sdl.values().copied() {
            match def {
                sdl::SdlDefinition::FieldDefinition(def) => {
                    match self.definitions.get_type_id(def.ty().name(), def.ty().span()) {
                        Ok(id) => builder.graph[def.id].ty_record.definition_id = id,
                        Err(err) => self.errors.push(err),
                    }
                }
                sdl::SdlDefinition::InputFieldDefinition(def) => {
                    match self.definitions.get_type_id(def.ty().name(), def.ty().span()) {
                        Ok(id) => builder.graph[def.id].ty_record.definition_id = id,
                        Err(err) => self.errors.push(err),
                    }
                }
                sdl::SdlDefinition::ArgumentDefinition(def) => {
                    match self.definitions.get_type_id(def.ty().name(), def.ty().span()) {
                        Ok(id) => builder.graph[def.id].ty_record.definition_id = id,
                        Err(err) => self.errors.push(err),
                    }
                }
                sdl::SdlDefinition::Object(def) => {
                    let mut interface_ids = Vec::new();
                    for inf in def.implements_interfaces() {
                        match self.definitions.get_interface_id(inf, def.span()) {
                            Ok(id) => interface_ids.push(id),
                            Err(err) => self.errors.push(err),
                        }
                    }
                    for inf_id in &interface_ids {
                        builder.graph[*inf_id].possible_type_ids.push(def.id);
                    }
                    builder.graph[def.id].interface_ids = interface_ids;
                }
                sdl::SdlDefinition::Interface(def) => {
                    let mut interface_ids = Vec::new();
                    for inf in def.implements_interfaces() {
                        match self.definitions.get_interface_id(inf, def.span()) {
                            Ok(id) => interface_ids.push(id),
                            Err(err) => self.errors.push(err),
                        }
                    }
                    builder.graph[def.id].interface_ids = interface_ids;
                }
                sdl::SdlDefinition::Union(def) => {
                    let mut member_ids = Vec::new();
                    for member in def.members() {
                        match self.definitions.get_object_id(member.name(), member.span()) {
                            Ok(id) => member_ids.push(id),
                            Err(err) => self.errors.push(err),
                        }
                    }
                    builder.graph[def.id].possible_type_ids = member_ids;
                }
                _ => (),
            }
        }
    }

    fn add_default_values(&mut self) {
        let mut seen = BitSet::with_capacity(self.graph.input_object_definitions.len());
        let mut input_values_stack = Vec::new();

        while let Some(id) = self.default_values.keys().next().copied() {
            input_values_stack.push(id);
            while let Some(id) = input_values_stack.pop() {
                if let TypeDefinitionId::InputObject(input_object_id) = self.graph[id].ty_record.definition_id {
                    // Nested default values must be treated first.
                    if !seen.put(input_object_id) {
                        // put back our current id.
                        input_values_stack.push(id);
                        input_values_stack.extend(self.graph[input_object_id].input_field_ids);
                        continue;
                    }
                }
                let Some((def, default_value)) = self.default_values.remove(&id) else {
                    continue;
                };
                match self.coerce_input_value(id, default_value) {
                    Ok(coerced) => self.graph[id].default_value_id = Some(coerced),
                    Err(err) => {
                        self.errors.push(Error::new(format!("At {}, found an invalid default value: {err}", def.to_site_string(self))).span(default_value.span()));
                    }
                }
            }
        }
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
        .inaccessible
        .object_definitions
        .grow(graph.object_definitions.len());
    graph.inaccessible.field_definitions.grow(graph.field_definitions.len());

    graph
        .inaccessible
        .scalar_definitions
        .grow(graph.scalar_definitions.len());

    graph
        .inaccessible
        .input_object_definitions
        .grow(graph.input_object_definitions.len());
    graph
        .inaccessible
        .input_value_definitions
        .grow(graph.input_value_definitions.len());

    graph.inaccessible.enum_definitions.grow(graph.enum_definitions.len());
    graph.inaccessible.enum_values.grow(graph.enum_values.len());

    graph.inaccessible.union_definitions.grow(graph.union_definitions.len());
    graph.union_has_inaccessible_member.grow(graph.union_definitions.len());

    graph
        .inaccessible
        .interface_definitions
        .grow(graph.interface_definitions.len());
    graph
        .interface_has_inaccessible_implementor
        .grow(graph.interface_definitions.len());
}
