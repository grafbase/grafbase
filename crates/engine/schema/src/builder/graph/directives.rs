use std::{collections::BTreeSet, mem::take};

use cynic_parser_deser::ConstDeserializer;
use itertools::Itertools;

use crate::builder::{
    Error,
    extension::LoadedExtensionOrCompositeSchema,
    sdl::{self, GraphName, SdlDefinition, SdlNestedDefinition, SdlTypeDefinition},
};

use super::*;

pub(crate) fn process_directives<'a>(
    builder: &mut GraphBuilder<'a>,
    sdl_definitions: sdl::SdlDefinitions<'a>,
) -> Result<(), Error> {
    let mut directives: Vec<sdl::Directive<'a>> = Vec::new();
    for def in sdl_definitions.types.iter().copied() {
        directives.clear();
        match def {
            SdlTypeDefinition::Enum(def) => {
                directives.extend(def.directives());
                if let Some(ext) = builder.sdl.type_extensions.get(def.name()) {
                    directives.extend(ext.iter().flat_map(|ext| ext.directives()));
                }
                builder.graph[def.id].directive_ids = builder.push_common_directives(def.into(), &directives)?;
                ingest_enum_definition_directive(builder, def.id, &directives)?
            }
            SdlTypeDefinition::InputObject(def) => {
                directives.extend(def.directives());
                if let Some(ext) = builder.sdl.type_extensions.get(def.name()) {
                    directives.extend(ext.iter().flat_map(|ext| ext.directives()));
                }
                builder.graph[def.id].directive_ids = builder.push_common_directives(def.into(), &directives)?;
                ingest_input_object_definition_directive(builder, def.id, &directives)?
            }
            SdlTypeDefinition::Interface(def) => {
                directives.extend(def.directives());
                if let Some(ext) = builder.sdl.type_extensions.get(def.name()) {
                    directives.extend(ext.iter().flat_map(|ext| ext.directives()));
                }
                builder.graph[def.id].directive_ids = builder.push_common_directives(def.into(), &directives)?;
                ingest_interface_definition_directive(builder, def, &directives)?
            }
            SdlTypeDefinition::Object(def) => {
                directives.extend(def.directives());
                if let Some(ext) = builder.sdl.type_extensions.get(def.name()) {
                    directives.extend(ext.iter().flat_map(|ext| ext.directives()));
                }
                builder.graph[def.id].directive_ids = builder.push_common_directives(def.into(), &directives)?;
                ingest_object_definition_directive(builder, def, &directives)?
            }
            SdlTypeDefinition::Scalar(def) => {
                directives.extend(def.directives());
                if let Some(ext) = builder.sdl.type_extensions.get(def.name()) {
                    directives.extend(ext.iter().flat_map(|ext| ext.directives()));
                }
                builder.graph[def.id].directive_ids = builder.push_common_directives(def.into(), &directives)?;
                ingest_scalar_definition_directive(builder, def.id, &directives)?
            }
            SdlTypeDefinition::Union(def) => {
                directives.extend(def.directives());
                if let Some(ext) = builder.sdl.type_extensions.get(def.name()) {
                    directives.extend(ext.iter().flat_map(|ext| ext.directives()));
                }
                builder.graph[def.id].directive_ids = builder.push_common_directives(def.into(), &directives)?;
                ingest_union_definition_directive(builder, def.id, &directives)?
            }
        }
    }

    for root_object_id in [
        Some(builder.graph.root_operation_types_record.query_id),
        builder.graph.root_operation_types_record.mutation_id,
        builder.graph.root_operation_types_record.subscription_id,
    ]
    .into_iter()
    .flatten()
    {
        let endpoint_ids = builder.graph[root_object_id]
            .exists_in_subgraph_ids
            .iter()
            .filter_map(|id| id.as_graphql_endpoint())
            .collect::<Vec<_>>();
        for endpoint_id in endpoint_ids {
            let resolver =
                ResolverDefinitionRecord::GraphqlRootField(GraphqlRootFieldResolverDefinitionRecord { endpoint_id });
            let id = builder.graph.resolver_definitions.len().into();
            builder.graph.resolver_definitions.push(resolver);
            builder
                .entity_resolvers
                .entry((root_object_id.into(), endpoint_id.into()))
                .or_default()
                .push(id);
        }
    }

    for def in sdl_definitions.nested.iter().copied() {
        directives.clear();
        match def {
            SdlNestedDefinition::FieldDefinition(def) => {
                directives.extend(def.directives());
                builder.graph[def.id].directive_ids = builder.push_common_directives(def.into(), &directives)?;
                ingest_field_directive(builder, def, &directives)?;
            }
            SdlNestedDefinition::InputFieldDefinition(def) => {
                directives.extend(def.directives());
                builder.graph[def.id].directive_ids = builder.push_common_directives(def.into(), &directives)?;
                ingest_input_value_directive(builder, def.id, &directives)?;
            }
            SdlNestedDefinition::ArgumentDefinition(def) => {
                directives.extend(def.directives());
                builder.graph[def.id].directive_ids = builder.push_common_directives(def.into(), &directives)?;
                ingest_input_value_directive(builder, def.id, &directives)?;
            }
            SdlNestedDefinition::EnumValue(def) => {
                directives.extend(def.directives());
                builder.graph[def.id].directive_ids = builder.push_common_directives(def.into(), &directives)?;
                ingest_enum_value_directive(builder, def.id, &directives)?;
            }
        }
    }

    finalize_inaccessible(&mut builder.graph);
    add_not_fully_implemented_in(&mut builder.graph);

    Ok(())
}

fn ingest_enum_definition_directive<'a>(
    GraphBuilder { ctx, graph, .. }: &mut GraphBuilder<'a>,
    id: EnumDefinitionId,
    directives: &[sdl::Directive<'a>],
) -> Result<(), Error> {
    if graph[id].exists_in_subgraph_ids.contains(&SubgraphId::Introspection) {
        return Ok(());
    }

    if has_inaccessible(directives) {
        graph.inaccessible_enum_definitions.set(id, true);
    }

    let enum_def = &mut graph[id];
    enum_def.exists_in_subgraph_ids = directives
        .iter()
        .filter_map(sdl::as_join_type)
        .map(|result| result.and_then(|(dir, span)| ctx.subgraphs.try_get(dir.graph, span)))
        .collect::<Result<Vec<_>, _>>()?;
    if enum_def.exists_in_subgraph_ids.is_empty() {
        enum_def.exists_in_subgraph_ids = ctx.subgraphs.all.clone()
    } else {
        enum_def.exists_in_subgraph_ids.sort_unstable();
    }

    Ok(())
}

fn ingest_input_object_definition_directive<'a>(
    GraphBuilder { ctx, graph, .. }: &mut GraphBuilder<'a>,
    id: InputObjectDefinitionId,
    directives: &[sdl::Directive<'a>],
) -> Result<(), Error> {
    if has_inaccessible(directives) {
        graph.inaccessible_input_object_definitions.set(id, true);
    }
    let input_object = &mut graph[id];
    input_object.exists_in_subgraph_ids = directives
        .iter()
        .filter_map(sdl::as_join_type)
        .map(|result| result.and_then(|(dir, span)| ctx.subgraphs.try_get(dir.graph, span)))
        .collect::<Result<Vec<_>, _>>()?;
    if input_object.exists_in_subgraph_ids.is_empty() {
        input_object.exists_in_subgraph_ids = ctx.subgraphs.all.clone()
    } else {
        input_object.exists_in_subgraph_ids.sort_unstable();
    }

    Ok(())
}

fn ingest_interface_definition_directive<'a>(
    builder: &mut GraphBuilder<'a>,
    def: sdl::InterfaceSdlDefinition<'a>,
    directives: &[sdl::Directive<'a>],
) -> Result<(), Error> {
    if has_inaccessible(directives) {
        builder.graph.inaccessible_interface_definitions.set(def.id, true);
    }

    let mut exists_in_subgraph_ids = take(&mut builder.graph[def.id].exists_in_subgraph_ids);
    for result in directives.iter().filter_map(sdl::as_join_type) {
        let (join_type, span) = result?;
        let subgraph_id = builder.subgraphs.try_get(join_type.graph, span)?;
        exists_in_subgraph_ids.push(subgraph_id);
        if join_type.is_interface_object {
            builder.graph[def.id].is_interface_object_in_ids.push(subgraph_id);
        }
        if let Some(graphql_endpoint_id) = subgraph_id.as_graphql_endpoint() {
            builder.push_apollo_federation_entity_resolver(def.into(), graphql_endpoint_id, join_type, span)?;
        }
    }
    if exists_in_subgraph_ids.is_empty() {
        exists_in_subgraph_ids = builder.subgraphs.all.clone()
    } else {
        exists_in_subgraph_ids.sort_unstable();
    }
    builder.graph[def.id].exists_in_subgraph_ids = exists_in_subgraph_ids;

    Ok(())
}

fn ingest_object_definition_directive<'a>(
    builder: &mut GraphBuilder<'a>,
    def: sdl::ObjectSdlDefinition<'a>,
    directives: &[sdl::Directive<'a>],
) -> Result<(), Error> {
    if builder.graph[def.id]
        .exists_in_subgraph_ids
        .contains(&SubgraphId::Introspection)
    {
        return Ok(());
    }

    if has_inaccessible(directives) {
        builder.graph.inaccessible_object_definitions.set(def.id, true);
        for interface_id in &builder.graph.object_definitions[usize::from(def.id)].interface_ids {
            builder
                .graph
                .interface_has_inaccessible_implementor
                .set(*interface_id, true);
        }
    }

    builder.graph[def.id].join_implement_records = directives
        .iter()
        .filter_map(sdl::as_join_implements)
        .map(|result| {
            let (dir, span) = result?;
            let subgraph_id = builder.subgraphs.try_get(dir.graph, span)?;
            builder
                .get_interface_id(dir.interface, span)
                .map(|interface_id| JoinImplementsDefinitionRecord {
                    subgraph_id,
                    interface_id,
                })
        })
        .collect::<Result<_, _>>()?;

    builder.graph[def.id]
        .join_implement_records
        .sort_by_key(|record| (record.subgraph_id, record.interface_id));

    let mut exists_in_subgraph_ids = take(&mut builder.graph[def.id].exists_in_subgraph_ids);
    for result in directives.iter().filter_map(sdl::as_join_type) {
        let (join_type, span) = result?;
        let subgraph_id = builder.subgraphs.try_get(join_type.graph, span)?;
        exists_in_subgraph_ids.push(subgraph_id);
        if let Some(graphql_endpoint_id) = subgraph_id.as_graphql_endpoint() {
            builder.push_apollo_federation_entity_resolver(def.into(), graphql_endpoint_id, join_type, span)?;
        }
    }

    if exists_in_subgraph_ids.is_empty() {
        exists_in_subgraph_ids = builder.ctx.subgraphs.all.clone()
    } else {
        exists_in_subgraph_ids.sort_unstable();
    }
    builder.graph[def.id].exists_in_subgraph_ids = exists_in_subgraph_ids;

    Ok(())
}

fn ingest_scalar_definition_directive<'a>(
    GraphBuilder { ctx, graph, .. }: &mut GraphBuilder<'a>,
    id: ScalarDefinitionId,
    directives: &[sdl::Directive<'a>],
) -> Result<(), Error> {
    if has_inaccessible(directives) {
        graph.inaccessible_scalar_definitions.set(id, true);
    }

    let scalar = &mut graph[id];
    scalar.exists_in_subgraph_ids = directives
        .iter()
        .filter_map(sdl::as_join_type)
        .map(|result| result.and_then(|(dir, span)| ctx.subgraphs.try_get(dir.graph, span)))
        .collect::<Result<Vec<_>, _>>()?;
    if scalar.exists_in_subgraph_ids.is_empty() {
        scalar.exists_in_subgraph_ids = ctx.subgraphs.all.clone()
    } else {
        scalar.exists_in_subgraph_ids.sort_unstable();
    }

    Ok(())
}

fn ingest_union_definition_directive<'a>(
    builder: &mut GraphBuilder<'a>,
    id: UnionDefinitionId,
    directives: &[sdl::Directive<'a>],
) -> Result<(), Error> {
    if builder.graph[id]
        .exists_in_subgraph_ids
        .contains(&SubgraphId::Introspection)
    {
        return Ok(());
    }

    if has_inaccessible(directives) {
        builder.graph.inaccessible_union_definitions.set(id, true);
    }

    builder.graph[id].join_member_records = directives
        .iter()
        .filter_map(sdl::as_join_union_member)
        .map(|result| {
            let (dir, span) = result?;
            let subgraph_id = builder.subgraphs.try_get(dir.graph, span)?;
            builder
                .get_object_id(dir.member, span)
                .map(|member_id| JoinMemberDefinitionRecord { subgraph_id, member_id })
        })
        .collect::<Result<_, _>>()?;

    let union = &mut builder.graph[id];
    union
        .join_member_records
        .sort_by_key(|record| (record.subgraph_id, record.member_id));

    union.exists_in_subgraph_ids = directives
        .iter()
        .filter_map(sdl::as_join_type)
        .map(|result| result.and_then(|(dir, span)| builder.ctx.subgraphs.try_get(dir.graph, span)))
        .collect::<Result<Vec<_>, _>>()?;
    if union.exists_in_subgraph_ids.is_empty() {
        union.exists_in_subgraph_ids = builder.ctx.subgraphs.all.clone()
    } else {
        union.exists_in_subgraph_ids.sort_unstable();
    }

    Ok(())
}

fn ingest_field_directive<'a>(
    builder: &mut GraphBuilder<'a>,
    def: sdl::FieldSdlDefinition<'a>,
    directives: &[sdl::Directive<'a>],
) -> Result<(), Error> {
    if builder.graph[def.id]
        .exists_in_subgraph_ids
        .contains(&SubgraphId::Introspection)
    {
        return Ok(());
    }

    if has_inaccessible(directives) {
        builder.graph.inaccessible_field_definitions.set(def.id, true);
    }

    let field = &mut builder.graph[def.id];
    let mut subgraph_type_records = take(&mut field.subgraph_type_records);
    let mut requires_records = take(&mut field.requires_records);
    let mut provides_records = take(&mut field.provides_records);
    let mut resolver_ids: Vec<ResolverDefinitionId> = take(&mut field.resolver_ids);
    // BTreeSet to ensures consistent ordering of resolvers.
    let mut resolvable_in = take(&mut field.exists_in_subgraph_ids)
        .into_iter()
        .collect::<BTreeSet<_>>();
    let parent_entity_id = field.parent_entity_id;
    let ty_record = field.ty_record;

    let mut has_join_field = false;
    let mut overrides = Vec::new();
    for result in directives.iter().filter_map(sdl::as_join_field) {
        let (dir, span) = result?;
        let subgraph_id = dir
            .graph
            .map(|name| builder.subgraphs.try_get(name, span))
            .transpose()?;

        // If there is a @join__field we rely solely on that to define the subgraphs in
        // which this field exists. It may not specify a subgraph at all, in that case it's
        // a interfaceObject field.
        has_join_field = true;
        if let Some(subgraph_id) = subgraph_id {
            if let Some(ty) = dir.r#type {
                let ty = builder.parse_type(ty, span)?;
                if ty != ty_record {
                    subgraph_type_records.push(SubgraphTypeRecord {
                        subgraph_id,
                        ty_record: ty,
                    });
                }
            }
            if !dir.external {
                if let Some(provides) = dir.provides.filter(|fields| !fields.is_empty()) {
                    let Some(parent) = ty_record.definition_id.as_composite_type() else {
                        return Err((
                            format!("Field {}.{} cannot have @provides", def.parent.name(), def.name()),
                            span,
                        )
                            .into());
                    };
                    let provides = builder.parse_field_set(parent, provides).map_err(|err| {
                        (
                            format!("At {}, invalid provides FieldSet: {err}", def.to_site_string(builder)),
                            span,
                        )
                    })?;
                    provides_records.push(FieldProvidesRecord {
                        subgraph_id,
                        field_set_record: provides,
                    });
                }
                if let Some(requires) = dir.requires.filter(|fields| !fields.is_empty()) {
                    let requires = builder
                        .parse_field_set(parent_entity_id.into(), requires)
                        .map_err(|err| {
                            (
                                format!("At {}, invalid requires FieldSet: {err}", def.to_site_string(builder)),
                                span,
                            )
                        })?;
                    requires_records.push(FieldRequiresRecord {
                        subgraph_id,
                        field_set_record: requires,
                    });
                }
                resolvable_in.insert(subgraph_id);
            }
        }

        if let Some(name) = dir.r#override {
            if let Ok(graph) = builder.subgraphs.try_get(GraphName(name), span) {
                overrides.push(graph);
            }
        }
    }

    let mut parent_has_join_type = false;
    let mut parent_directives = Vec::new();
    parent_directives.extend(def.parent.directives());
    if let Some(ext) = builder.sdl.type_extensions.get(def.parent.name()) {
        parent_directives.extend(ext.iter().flat_map(|ext| ext.directives()));
    }
    for result in parent_directives.iter().filter_map(sdl::as_join_type) {
        let (dir, span) = result?;

        parent_has_join_type = true;
        if !has_join_field && dir.resolvable {
            let subgraph_id = builder.subgraphs.try_get(dir.graph, span)?;
            // If there is no @join__field we rely solely @join__type to define the subgraphs
            // in which this field is resolvable in.
            resolvable_in.insert(subgraph_id);
        }
    }

    // Remove any overridden subgraphs
    for subgraph_id in overrides.iter() {
        resolvable_in.remove(subgraph_id);
    }

    // If there is no @join__field and no @join__type at all, we assume this field to be
    // available everywhere.
    let mut exists_in_subgraph_ids = if !has_join_field && !parent_has_join_type {
        builder.subgraphs.all.clone()
    } else {
        resolvable_in.into_iter().collect::<Vec<_>>()
    };

    let parent_entity_id = builder.graph[def.id].parent_entity_id;
    for &subgraph_id in &exists_in_subgraph_ids {
        let Some(entity_resolver_ids) = builder.entity_resolvers.get(&(parent_entity_id, subgraph_id)) else {
            continue;
        };
        for resolver_definition_id in entity_resolver_ids {
            match &builder.graph[*resolver_definition_id] {
                ResolverDefinitionRecord::GraphqlFederationEntity(
                    GraphqlFederationEntityResolverDefinitionRecord { key_fields_record, .. },
                ) => {
                    // If part of the key we can't be provided by this resolver.
                    if key_fields_record
                        .iter()
                        .all(|item| builder.graph[item.field_id].definition_id != def.id)
                    {
                        resolver_ids.push(*resolver_definition_id);
                    }
                }
                ResolverDefinitionRecord::GraphqlRootField(_) => {
                    resolver_ids.push(*resolver_definition_id);
                }
                ResolverDefinitionRecord::FieldResolverExtension(_)
                | ResolverDefinitionRecord::Introspection
                | ResolverDefinitionRecord::SelectionSetResolverExtension(_) => {}
            }
        }
    }

    let directive_ids = take(&mut builder.graph[def.id].directive_ids);
    for id in &directive_ids {
        let &TypeSystemDirectiveId::Extension(id) = id else {
            continue;
        };
        let directive = &builder.graph.extension_directives[usize::from(id)];
        if directive.ty.is_field_resolver() {
            let subgraph_id = directive.subgraph_id;
            if !exists_in_subgraph_ids.contains(&subgraph_id) {
                exists_in_subgraph_ids.push(subgraph_id);
            }
            builder
                .graph
                .resolver_definitions
                .push(ResolverDefinitionRecord::FieldResolverExtension(
                    FieldResolverExtensionDefinitionRecord { directive_id: id },
                ));
            resolver_ids.push(ResolverDefinitionId::from(builder.graph.resolver_definitions.len() - 1))
        }
    }

    let field = &mut builder.graph[def.id];
    field.directive_ids = directive_ids;
    field.subgraph_type_records = subgraph_type_records;
    field.exists_in_subgraph_ids = exists_in_subgraph_ids;
    field.resolver_ids = resolver_ids;
    field.provides_records = provides_records;
    field.requires_records = requires_records;

    Ok(())
}

fn ingest_input_value_directive<'a>(
    builder: &mut GraphBuilder<'a>,
    id: InputValueDefinitionId,
    directives: &[sdl::Directive<'a>],
) -> Result<(), Error> {
    if has_inaccessible(directives) {
        builder.graph.inaccessible_input_value_definitions.set(id, true);
    }
    Ok(())
}

fn ingest_enum_value_directive<'a>(
    builder: &mut GraphBuilder<'a>,
    id: EnumValueId,
    directives: &[sdl::Directive<'a>],
) -> Result<(), Error> {
    if has_inaccessible(directives) {
        builder.graph.inaccessible_enum_values.set(id, true);
    }
    Ok(())
}

impl<'a> GraphBuilder<'a> {
    fn push_common_directives(
        &mut self,
        def: SdlDefinition<'a>,
        directives: &[sdl::Directive<'a>],
    ) -> Result<Vec<TypeSystemDirectiveId>, Error> {
        let mut directive_ids = Vec::new();

        for &directive in directives {
            let id = match directive.name() {
                "authenticated" => TypeSystemDirectiveId::Authenticated,
                "requiresScopes" => {
                    let dir = directive.deserialize::<sdl::RequiresScopesDirective>().map_err(|err| {
                        (
                            format!("Invalid @requiresScopes directive: {}", err),
                            directive.arguments_span(),
                        )
                    })?;
                    let scope = RequiresScopesDirectiveRecord::new(
                        dir.scopes
                            .into_iter()
                            .map(|scopes| scopes.into_iter().map(|scope| self.ingest_str(scope)).collect())
                            .collect(),
                    );
                    let id = self.required_scopes.get_or_insert(scope);
                    TypeSystemDirectiveId::RequiresScopes(id)
                }
                "deprecated" => {
                    let dir = directive.deserialize::<sdl::DeprecatedDirective>().map_err(|err| {
                        (
                            format!("Invalid @deprecated directive: {}", err),
                            directive.arguments_span(),
                        )
                    })?;
                    let reason_id = dir.reason.map(|reason| self.ingest_str(reason));
                    TypeSystemDirectiveId::Deprecated(DeprecatedDirectiveRecord { reason_id })
                }
                "cost" => {
                    let dir = directive
                        .deserialize::<sdl::CostDirective>()
                        .map_err(|err| (format!("Invalid @cost directive: {}", err), directive.arguments_span()))?;
                    self.graph
                        .cost_directives
                        .push(CostDirectiveRecord { weight: dir.weight });
                    TypeSystemDirectiveId::Cost((self.graph.cost_directives.len() - 1).into())
                }
                "listSize" => {
                    let SdlDefinition::FieldDefinition(def) = def else {
                        return Err((
                            format!("Invalid @listSize directive location: {}", def.location()),
                            directive.name_span(),
                        )
                            .into());
                    };
                    let dir = directive.deserialize::<sdl::ListSizeDirective>().map_err(|err| {
                        (
                            format!("Invalid @listSize directive: {}", err),
                            directive.arguments_span(),
                        )
                    })?;
                    let slicing_argument_ids = {
                        let field_argument_ids = self.graph[def.id].argument_ids;
                        dir.slicing_arguments
                            .into_iter()
                            .map(|name| {
                                field_argument_ids
                                    .into_iter()
                                    .find(|id| self.ctx[self.graph[*id].name_id] == name)
                                    .ok_or_else(|| {
                                        (
                                            format!("Invalid @listSize directive slicing_argument: {}", name),
                                            directive.arguments_span(),
                                        )
                                    })
                            })
                            .collect::<Result<Vec<_>, _>>()
                    }?;
                    let sized_field_ids = if !dir.sized_fields.is_empty() {
                        let output_field_ids = match self.graph[def.id].ty_record.definition_id {
                            TypeDefinitionId::Interface(id) => self.graph[id].field_ids,
                            TypeDefinitionId::Object(id) => self.graph[id].field_ids,
                            _ => {
                                return Err((
                                    "sized_fields can only be used with a interface/object output type",
                                    directive.arguments_span(),
                                )
                                    .into());
                            }
                        };
                        dir.sized_fields
                            .into_iter()
                            .map(|name| {
                                output_field_ids
                                    .into_iter()
                                    .find(|id| self.ctx[self.graph[*id].name_id] == name)
                                    .ok_or_else(|| {
                                        (
                                            format!("Invalid @listSize directive sized_field: {}", name),
                                            directive.arguments_span(),
                                        )
                                    })
                            })
                            .collect::<Result<Vec<_>, _>>()?
                    } else {
                        Vec::new()
                    };
                    self.graph.list_size_directives.push(ListSizeDirectiveRecord {
                        assumed_size: dir.assumed_size,
                        slicing_argument_ids,
                        sized_field_ids,
                        require_one_slicing_argument: dir.require_one_slicing_argument,
                    });
                    TypeSystemDirectiveId::ListSize((self.graph.list_size_directives.len() - 1).into())
                }
                "extension__directive" => {
                    let dir = sdl::parse_extension_directive(directive)?;
                    let subgraph_id = self.subgraphs.try_get(dir.graph, directive.arguments_span())?;
                    match self.extensions.get(dir.extension) {
                        LoadedExtensionOrCompositeSchema::Extension(extension) => {
                            let id = self
                                .ingest_extension_directive(def, subgraph_id, extension, dir.name, dir.arguments)
                                .map_err(|txt| (txt, directive.arguments_span()))?;
                            TypeSystemDirectiveId::Extension(id)
                        }
                        LoadedExtensionOrCompositeSchema::CompositeSchema => {
                            self.ingest_composite_schema_directive(def, subgraph_id, dir.name, dir.arguments)
                                .map_err(|txt| (txt, directive.arguments_span()))?;
                            continue;
                        }
                    }
                }
                _ => continue,
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
    fn is_definition_inaccessible(graph: &Graph, definition_id: TypeDefinitionId) -> bool {
        match definition_id {
            TypeDefinitionId::Scalar(id) => graph.inaccessible_scalar_definitions[id],
            TypeDefinitionId::Object(id) => graph.inaccessible_object_definitions[id],
            TypeDefinitionId::Interface(id) => graph.inaccessible_interface_definitions[id],
            TypeDefinitionId::Union(id) => graph.inaccessible_union_definitions[id],
            TypeDefinitionId::Enum(id) => graph.inaccessible_enum_definitions[id],
            TypeDefinitionId::InputObject(id) => graph.inaccessible_input_object_definitions[id],
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

impl<'a> GraphBuilder<'a> {
    fn push_apollo_federation_entity_resolver(
        &mut self,
        entity: sdl::EntitySdlDefinition<'_>,
        endpoint_id: GraphqlEndpointId,
        join_type: sdl::JoinTypeDirective<'a>,
        span: sdl::Span,
    ) -> Result<(), Error> {
        let subgraph_id = SubgraphId::from(endpoint_id);
        let Some(key) = join_type.key.filter(|key| !key.is_empty()) else {
            return Ok(());
        };
        let key = self.parse_field_set(entity.id().into(), key).map_err(|err| {
            (
                format!("At {}, invalid key FieldSet: {}", entity.to_site_string(self), err),
                span,
            )
        })?;

        // Any field that is part of a key has to exist in the subgraph.
        let mut stack = vec![&key];
        while let Some(fields) = stack.pop() {
            for item in fields {
                let id = self.graph[item.field_id].definition_id;
                let field = &mut self.graph[id];
                if !field.exists_in_subgraph_ids.contains(&subgraph_id) {
                    field.exists_in_subgraph_ids.push(subgraph_id);
                }
            }
        }

        if join_type.resolvable {
            let resolver =
                ResolverDefinitionRecord::GraphqlFederationEntity(GraphqlFederationEntityResolverDefinitionRecord {
                    key_fields_record: key,
                    endpoint_id,
                });
            let id = self.graph.resolver_definitions.len().into();
            self.graph.resolver_definitions.push(resolver);
            self.entity_resolvers
                .entry((entity.id(), subgraph_id))
                .or_default()
                .push(id);
        }
        Ok(())
    }
}

fn has_inaccessible(directives: &[sdl::Directive<'_>]) -> bool {
    directives.iter().any(|dir| dir.name() == "inaccessible")
}
