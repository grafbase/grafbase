use std::{
    collections::{hash_map::Entry, BTreeSet},
    mem::take,
};

use builder::SchemaLocation;
use federated_graph::{JoinFieldDirective, JoinImplementsDirective, JoinTypeDirective, JoinUnionMemberDirective};
use itertools::Itertools;

use super::*;

pub(super) fn post_process_schema_locations(
    ctx: &mut GraphContext<'_>,
    locations: Vec<SchemaLocation>,
) -> Result<(), BuildError> {
    let root_entities = [
        Some(EntityDefinitionId::from(ctx.graph.root_operation_types_record.query_id)),
        ctx.graph.root_operation_types_record.mutation_id.map(Into::into),
        ctx.graph.root_operation_types_record.subscription_id.map(Into::into),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    for location in locations {
        match location {
            SchemaLocation::Enum(id, federated_id) => {
                ctx.graph[id].directive_ids =
                    ctx.push_directives(location, &ctx.federated_graph[federated_id].directives)?;
                ingest_enum_definition_directive(ctx, id, federated_id)?
            }
            SchemaLocation::InputObject(id, federated_id) => {
                ctx.graph[id].directive_ids =
                    ctx.push_directives(location, &ctx.federated_graph[federated_id].directives)?;
                ingest_input_object_definition_directive(ctx, id, federated_id)?
            }
            SchemaLocation::Interface(id, federated_id) => {
                ctx.graph[id].directive_ids =
                    ctx.push_directives(location, &ctx.federated_graph[federated_id].directives)?;
                ingest_interface_definition_directive(ctx, id, federated_id)?
            }
            SchemaLocation::Object(id, federated_id) => {
                ctx.graph[id].directive_ids =
                    ctx.push_directives(location, &ctx.federated_graph[federated_id].directives)?;
                ingest_object_definition_directive(ctx, id, federated_id)?
            }
            SchemaLocation::Scalar(id, federated_id) => {
                ctx.graph[id].directive_ids =
                    ctx.push_directives(location, &ctx.federated_graph[federated_id].directives)?;
                ingest_scalar_definition_directive(ctx, id, federated_id)?
            }
            SchemaLocation::Union(id, federated_id) => {
                ctx.graph[id].directive_ids =
                    ctx.push_directives(location, &ctx.federated_graph[federated_id].directives)?;
                ingest_union_definition_directive(ctx, id, federated_id)?
            }
            SchemaLocation::Field(id, federated_id) => {
                ctx.graph[id].directive_ids =
                    ctx.push_directives(location, &ctx.federated_graph[federated_id].directives)?;
                ingest_field_directive(ctx, &root_entities, id, federated_id)?
            }
            SchemaLocation::InputValue(id, federated_id) => {
                ctx.graph[id].directive_ids =
                    ctx.push_directives(location, &ctx.federated_graph[federated_id].directives)?;
                ingest_input_value_directive(ctx, id, federated_id)?
            }
            SchemaLocation::EnumValue(id, federated_id) => {
                ctx.graph[id].directive_ids =
                    ctx.push_directives(location, &ctx.federated_graph[federated_id].directives)?;
                ingest_enum_value_directive(ctx, id, federated_id)?
            }
        }
    }

    finalize_inaccessible(&mut ctx.graph);
    add_not_fully_implemented_in(&mut ctx.graph);
    add_extra_vecs_with_different_ordering(ctx);

    Ok(())
}

fn ingest_enum_definition_directive(
    ctx: &mut GraphContext<'_>,
    id: EnumDefinitionId,
    federated_id: federated_graph::EnumDefinitionId,
) -> Result<(), BuildError> {
    let directives = &ctx.federated_graph[federated_id].directives;
    if has_inaccessible(directives) {
        ctx.graph.inaccessible_enum_definitions.set(id, true);
    }

    Ok(())
}

fn ingest_input_object_definition_directive(
    ctx: &mut GraphContext<'_>,
    id: InputObjectDefinitionId,
    federated_id: federated_graph::InputObjectId,
) -> Result<(), BuildError> {
    let directives = &ctx.federated_graph[federated_id].directives;
    if has_inaccessible(directives) {
        ctx.graph.inaccessible_input_object_definitions.set(id, true);
    }
    Ok(())
}

fn ingest_interface_definition_directive(
    GraphContext { ctx, graph, .. }: &mut GraphContext<'_>,
    id: InterfaceDefinitionId,
    federated_id: federated_graph::InterfaceId,
) -> Result<(), BuildError> {
    let directives = &ctx.federated_graph[federated_id].directives;
    if has_inaccessible(directives) {
        graph.inaccessible_interface_definitions.set(id, true);
    }

    let interface = &mut graph[id];
    for dir in directives.iter().filter_map(|dir| dir.as_join_type()) {
        interface.exists_in_subgraph_ids.push(ctx.subgraphs[dir.subgraph_id]);
        if dir.is_interface_object {
            interface
                .is_interface_object_in_ids
                .push(ctx.subgraphs[dir.subgraph_id]);
        }
    }

    Ok(())
}

fn ingest_object_definition_directive(
    GraphContext { ctx, graph, .. }: &mut GraphContext<'_>,
    id: ObjectDefinitionId,
    federated_id: federated_graph::ObjectId,
) -> Result<(), BuildError> {
    let directives = &ctx.federated_graph[federated_id].directives;
    if has_inaccessible(directives) {
        graph.inaccessible_object_definitions.set(id, true);
        for interface_id in &graph.object_definitions[usize::from(id)].interface_ids {
            graph.interface_has_inaccessible_implementor.set(*interface_id, true);
        }
    }

    let object = &mut graph[id];

    object.join_implement_records = directives
        .iter()
        .filter_map(|dir| dir.as_join_implements())
        .map(
            |&JoinImplementsDirective {
                 subgraph_id,
                 interface_id,
             }| {
                JoinImplementsDefinitionRecord {
                    subgraph_id: ctx.subgraphs[subgraph_id],
                    interface_id: interface_id.into(),
                }
            },
        )
        .collect();

    object
        .join_implement_records
        .sort_by_key(|record| (record.subgraph_id, record.interface_id));

    object.exists_in_subgraph_ids = directives
        .iter()
        .filter_map(|dir| dir.as_join_type())
        .map(|dir| ctx.subgraphs[dir.subgraph_id])
        .collect::<Vec<_>>();
    if object.exists_in_subgraph_ids.is_empty() {
        object.exists_in_subgraph_ids = ctx.subgraphs.all.clone()
    } else {
        object.exists_in_subgraph_ids.sort_unstable();
    }

    Ok(())
}

fn ingest_scalar_definition_directive(
    ctx: &mut GraphContext<'_>,
    id: ScalarDefinitionId,
    federated_id: federated_graph::ScalarDefinitionId,
) -> Result<(), BuildError> {
    let directives = &ctx.federated_graph[federated_id].directives;
    if has_inaccessible(directives) {
        ctx.graph.inaccessible_scalar_definitions.set(id, true);
    }
    Ok(())
}

fn ingest_union_definition_directive(
    GraphContext { ctx, graph, .. }: &mut GraphContext<'_>,
    id: UnionDefinitionId,
    federated_id: federated_graph::UnionId,
) -> Result<(), BuildError> {
    let directives = &ctx.federated_graph[federated_id].directives;
    if has_inaccessible(directives) {
        graph.inaccessible_union_definitions.set(id, true);
    }

    let union = &mut graph[id];
    union.join_member_records = directives
        .iter()
        .filter_map(|dir| dir.as_join_union_member())
        .map(
            |&JoinUnionMemberDirective { subgraph_id, object_id }| JoinMemberDefinitionRecord {
                subgraph_id: ctx.subgraphs[subgraph_id],
                member_id: object_id.into(),
            },
        )
        .collect();

    union
        .join_member_records
        .sort_by_key(|record| (record.subgraph_id, record.member_id));

    Ok(())
}

fn ingest_field_directive(
    ctx: &mut GraphContext<'_>,
    root_entities: &[EntityDefinitionId],
    id: FieldDefinitionId,
    federated_id: federated_graph::FieldId,
) -> Result<(), BuildError> {
    let federated_field = &ctx.federated_graph[federated_id];

    if has_inaccessible(&federated_field.directives) {
        ctx.graph.inaccessible_field_definitions.set(id, true);
    }

    let field = &mut ctx.graph[id];
    let mut subgraph_type_records = take(&mut field.subgraph_type_records);
    let mut requires_records = take(&mut field.requires_records);
    let mut provides_records = take(&mut field.provides_records);
    let mut resolver_ids: Vec<ResolverDefinitionId> = take(&mut field.resolver_ids);
    // BTreeSet to ensures consistent ordering of resolvers.
    let mut resolvable_in = take(&mut field.exists_in_subgraph_ids)
        .into_iter()
        .collect::<BTreeSet<_>>();
    let mut has_join_field = false;

    for JoinFieldDirective {
        subgraph_id: federated_subgraph_id,
        requires,
        provides,
        r#type,
        ..
    } in federated_field.directives.iter().filter_map(|dir| dir.as_join_field())
    {
        // If there is a @join__field we rely solely on that to define the subgraphs in
        // which this field exists. It may not specify a subgraph at all, in that case it's
        // a interfaceObject field.
        has_join_field = true;
        if let Some(federated_subgraph_id) = *federated_subgraph_id {
            let subgraph_id = ctx.subgraphs[federated_subgraph_id];
            if let Some(r#type) = r#type.filter(|ty| ty != &federated_field.r#type) {
                subgraph_type_records.push(SubgraphTypeRecord {
                    subgraph_id,
                    ty_record: ctx.convert_type(r#type),
                });
            }
            if let Some(provides) = provides.as_ref().filter(|provides| !provides.is_empty()) {
                provides_records.push(FieldProvidesRecord {
                    subgraph_id,
                    field_set_record: ctx.convert_field_set(provides).map_err(|err| {
                        BuildError::RequiredFieldArgumentCoercionError {
                            location: ctx.strings[ctx.graph[id].name_id].to_string(),
                            err,
                        }
                    })?,
                });
            }
            if let Some(requires) = requires.as_ref().filter(|requires| !requires.is_empty()) {
                requires_records.push(FieldRequiresRecord {
                    subgraph_id,
                    field_set_record: ctx.convert_field_set(requires).map_err(|err| {
                        BuildError::RequiredFieldArgumentCoercionError {
                            location: ctx.strings[ctx.graph[id].name_id].to_string(),
                            err,
                        }
                    })?,
                });
            }
            resolvable_in.insert(subgraph_id);
        }
    }

    let parent_entity = ctx.federated_graph.entity(federated_field.parent_entity_id);
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
            resolvable_in.insert(ctx.subgraphs[*subgraph_id]);
        } else if !has_join_field && *resolvable {
            // If there is no @join__field we rely solely @join__type to define the subgraphs
            // in which this field is resolvable in.
            resolvable_in.insert(ctx.subgraphs[*subgraph_id]);
        }
    }

    // Remove any overridden subgraphs
    for directive in federated_field.directives.iter().filter_map(|dir| dir.as_join_field()) {
        if let Some(r#override) = &directive.r#override {
            match r#override {
                federated_graph::OverrideSource::Subgraph(subgraph_id) => {
                    resolvable_in.remove(&ctx.subgraphs[*subgraph_id]);
                }
                federated_graph::OverrideSource::Missing(_) => (),
            };
        }
    }

    // If there is no @join__field and no @join__type at all, we assume this field to be
    // available everywhere.
    let exists_in_subgraph_ids = if !has_join_field && !parent_has_join_type {
        ctx.subgraphs.all.clone()
    } else {
        resolvable_in.into_iter().collect::<Vec<_>>()
    };

    let parent_entity_id = ctx.graph[id].parent_entity_id;
    let is_root_entity = root_entities.contains(&parent_entity_id);
    let mut graphql_federated_entity_resolvers = take(&mut ctx.graphql_federated_entity_resolvers);
    for &subgraph_id in &exists_in_subgraph_ids {
        match subgraph_id {
            SubgraphId::GraphqlEndpoint(endpoint_id) if is_root_entity => {
                resolver_ids.extend(
                    graphql_federated_entity_resolvers
                        .entry((parent_entity_id, endpoint_id))
                        .or_insert_with(|| {
                            let id = ctx.graph.resolver_definitions.len().into();
                            ctx.graph
                                .resolver_definitions
                                .push(ResolverDefinitionRecord::GraphqlRootField(
                                    GraphqlRootFieldResolverDefinitionRecord { endpoint_id },
                                ));
                            vec![EntityResovler::Root(id)]
                        })
                        .iter()
                        .map(|res| res.id()),
                );
            }
            SubgraphId::GraphqlEndpoint(endpoint_id) => {
                let endpoint_resolvers = match graphql_federated_entity_resolvers.entry((parent_entity_id, endpoint_id))
                {
                    Entry::Occupied(entry) => entry.into_mut(),
                    Entry::Vacant(entry) => {
                        let mut result = Vec::new();

                        for dir in parent_entity.directives() {
                            let Some(join_type) = dir.as_join_type() else {
                                continue;
                            };
                            let Some(key) = join_type.key.as_ref().filter(|key| {
                                !key.is_empty()
                                    && ctx.subgraphs[join_type.subgraph_id] == subgraph_id
                                    && join_type.resolvable
                            }) else {
                                continue;
                            };
                            let resolver = ResolverDefinitionRecord::GraphqlFederationEntity(
                                GraphqlFederationEntityResolverDefinitionRecord {
                                    key_fields_record: ctx.convert_field_set(key).map_err(|err| {
                                        BuildError::RequiredFieldArgumentCoercionError {
                                            location: ctx.strings[ctx.graph[id].name_id].to_string(),
                                            err,
                                        }
                                    })?,
                                    endpoint_id,
                                },
                            );
                            let id = ctx.graph.resolver_definitions.len().into();
                            ctx.graph.resolver_definitions.push(resolver);
                            result.push(EntityResovler::Entity { key: key.clone(), id });
                        }
                        entry.insert(result)
                    }
                };
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
    ctx.graphql_federated_entity_resolvers = graphql_federated_entity_resolvers;

    let directive_ids = ctx.push_directives(SchemaLocation::Field(id, federated_id), &federated_field.directives)?;
    resolver_ids.extend(
        directive_ids
            .iter()
            .filter_map(|id| id.as_extension())
            .filter_map(|directive_id| {
                if exists_in_subgraph_ids.contains(&ctx.graph[directive_id].subgraph_id) {
                    ctx.graph
                        .resolver_definitions
                        .push(ResolverDefinitionRecord::FieldResolverExtension(
                            FieldResolverExtensionDefinitionRecord { directive_id },
                        ));
                    Some(ResolverDefinitionId::from(ctx.graph.resolver_definitions.len() - 1))
                } else {
                    None
                }
            }),
    );

    let field = &mut ctx.graph[id];
    field.subgraph_type_records = subgraph_type_records;
    field.exists_in_subgraph_ids = exists_in_subgraph_ids;
    field.resolver_ids = resolver_ids;
    field.provides_records = provides_records;
    field.requires_records = requires_records;

    Ok(())
}

fn ingest_input_value_directive(
    ctx: &mut GraphContext<'_>,
    id: InputValueDefinitionId,
    federated_id: federated_graph::InputValueDefinitionId,
) -> Result<(), BuildError> {
    let directives = &ctx.federated_graph[federated_id].directives;
    if has_inaccessible(directives) {
        ctx.graph.inaccessible_input_value_definitions.set(id, true);
    }
    if let Some(value) = ctx.federated_graph[federated_id].default.clone() {
        ctx.graph[id].default_value_id =
            Some(
                ctx.coerce(id, value)
                    .map_err(|err| BuildError::DefaultValueCoercionError {
                        err,
                        name: ctx.strings[ctx.graph[id].name_id].to_string(),
                    })?,
            );
    }
    Ok(())
}

fn ingest_enum_value_directive(
    ctx: &mut GraphContext<'_>,
    id: EnumValueId,
    federated_id: federated_graph::EnumValueId,
) -> Result<(), BuildError> {
    let directives = &ctx.federated_graph[federated_id].directives;
    if has_inaccessible(directives) {
        ctx.graph.inaccessible_enum_values.set(id, true);
    }
    Ok(())
}

impl GraphContext<'_> {
    fn push_directives<'d>(
        &mut self,
        schema_location: SchemaLocation,
        directives: impl IntoIterator<Item = &'d federated_graph::Directive>,
    ) -> Result<Vec<TypeSystemDirectiveId>, BuildError> {
        let mut directive_ids = Vec::new();

        for directive in directives {
            let id = match directive {
                federated_graph::Directive::Authenticated => TypeSystemDirectiveId::Authenticated,
                federated_graph::Directive::RequiresScopes(federated_scopes) => {
                    let scope = RequiresScopesDirectiveRecord::new(
                        federated_scopes
                            .iter()
                            .map(|scopes| {
                                scopes
                                    .iter()
                                    .copied()
                                    .map(|scope| self.get_or_insert_str(scope))
                                    .collect()
                            })
                            .collect(),
                    );
                    let id = self.required_scopes.get_or_insert(scope);
                    TypeSystemDirectiveId::RequiresScopes(id)
                }
                federated_graph::Directive::Deprecated { reason } => {
                    TypeSystemDirectiveId::Deprecated(DeprecatedDirectiveRecord {
                        reason_id: reason.map(|id| self.get_or_insert_str(id)),
                    })
                }
                federated_graph::Directive::Authorized(authorized) => {
                    let record = AuthorizedDirectiveRecord {
                        arguments: authorized
                            .arguments
                            .as_ref()
                            .map(convert_input_value_set)
                            .unwrap_or_default(),
                        fields_record: authorized
                            .fields
                            .as_ref()
                            .map(|field_set| {
                                self.convert_field_set(field_set).map_err(|err| {
                                    BuildError::RequiredFieldArgumentCoercionError {
                                        location: schema_location.to_string(self),
                                        err,
                                    }
                                })
                            })
                            .transpose()?,
                        node_record: authorized
                            .node
                            .as_ref()
                            .map(|field_set| {
                                self.convert_field_set(field_set).map_err(|err| {
                                    BuildError::RequiredFieldArgumentCoercionError {
                                        location: schema_location.to_string(self),
                                        err,
                                    }
                                })
                            })
                            .transpose()?,
                        metadata_id: authorized.metadata.clone().map(|value| {
                            let value = self.ingest_arbitrary_value(value);
                            self.graph.input_values.push_value(value)
                        }),
                    };
                    self.graph.authorized_directives.push(record);

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
                    let extension_id = &self[*extension_id].id;
                    let Some(id) = self.extension_catalog.find_compatible_extension(extension_id) else {
                        return Err(BuildError::UnsupportedExtension {
                            id: Box::new(extension_id.clone()),
                        });
                    };

                    let record = ExtensionDirectiveRecord {
                        subgraph_id: self.subgraphs[*subgraph_id],
                        extension_id: id,
                        name_id: self.get_or_insert_str(*name),
                        arguments_id: arguments.as_ref().map(|arguments| {
                            let value =
                                self.ingest_arbitrary_value(federated_graph::Value::Object(arguments.clone().into()));
                            self.graph.input_values.push_value(value)
                        }),
                    };
                    self.graph.extension_directives.push(record);
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

fn finalize_inaccessible(graph: &mut Graph) {
    // Must be done after ingesting all @inaccessible for objects.
    for (ix, union) in graph.union_definitions.iter().enumerate() {
        let id = UnionDefinitionId::from(ix);
        for possible_type in &union.possible_type_ids {
            if graph.inaccessible_object_definitions[*possible_type] {
                graph.union_has_inaccessible_member.set(id, true);
                break;
            }
        }
    }

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
        if is_definition_inaccessible(graph, field.ty_record.definition_id) {
            graph.inaccessible_field_definitions.set(ix.into(), true);
        }
    }

    for (ix, input_value) in graph.input_value_definitions.iter().enumerate() {
        if is_definition_inaccessible(graph, input_value.ty_record.definition_id) {
            graph.inaccessible_input_value_definitions.set(ix.into(), true);
        }
    }
}

fn add_not_fully_implemented_in(graph: &mut Graph) {
    let mut not_fully_implemented_in_ids = Vec::new();
    for (ix, interface) in graph.interface_definitions.iter_mut().enumerate() {
        let interface_id = InterfaceDefinitionId::from(ix);

        // For every possible type implementing this interface.
        for object_id in &interface.possible_type_ids {
            let object = &graph.object_definitions[usize::from(*object_id)];

            // Check in which subgraphs these are resolved.
            for subgraph_id in &interface.exists_in_subgraph_ids {
                // The object implements the interface if it defines az `@join__implements`
                // corresponding to the interface and to the subgraph.
                if object.implements_interface_in_subgraph(subgraph_id, &interface_id) {
                    continue;
                }

                not_fully_implemented_in_ids.push(*subgraph_id);
            }
        }

        not_fully_implemented_in_ids.sort_unstable();
        // Sorted by the subgraph id
        interface
            .not_fully_implemented_in_ids
            .extend(not_fully_implemented_in_ids.drain(..).dedup())
    }

    let mut exists_in_subgraph_ids = Vec::new();
    for union in graph.union_definitions.iter_mut() {
        exists_in_subgraph_ids.clear();
        exists_in_subgraph_ids.extend(union.join_member_records.iter().map(|join| join.subgraph_id));
        exists_in_subgraph_ids.sort_unstable();
        exists_in_subgraph_ids.dedup();

        for object_id in &union.possible_type_ids {
            for subgraph_id in &exists_in_subgraph_ids {
                // The object implements the interface if it defines az `@join__implements`
                // corresponding to the interface and to the subgraph.
                if union
                    .join_member_records
                    .binary_search_by(|probe| probe.subgraph_id.cmp(subgraph_id).then(probe.member_id.cmp(object_id)))
                    .is_err()
                {
                    not_fully_implemented_in_ids.push(*subgraph_id);
                }
            }
        }

        not_fully_implemented_in_ids.sort_unstable();
        // Sorted by the subgraph id
        union
            .not_fully_implemented_in_ids
            .extend(not_fully_implemented_in_ids.drain(..).dedup())
    }
}

fn add_extra_vecs_with_different_ordering(GraphContext { ctx, graph, .. }: &mut GraphContext<'_>) {
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
        definitions
            .extend((0..graph.scalar_definitions.len()).map(|id| DefinitionId::Scalar(ScalarDefinitionId::from(id))));
        definitions
            .extend((0..graph.object_definitions.len()).map(|id| DefinitionId::Object(ObjectDefinitionId::from(id))));
        definitions.extend(
            (0..graph.interface_definitions.len()).map(|id| DefinitionId::Interface(InterfaceDefinitionId::from(id))),
        );
        definitions
            .extend((0..graph.union_definitions.len()).map(|id| DefinitionId::Union(UnionDefinitionId::from(id))));
        definitions.extend((0..graph.enum_definitions.len()).map(|id| DefinitionId::Enum(EnumDefinitionId::from(id))));
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
