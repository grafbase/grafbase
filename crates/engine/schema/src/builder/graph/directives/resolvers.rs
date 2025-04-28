use std::mem::take;

use cynic_parser_deser::ConstDeserializer;

use crate::{
    EntityDefinitionId, FieldResolverExtensionDefinitionRecord, GraphqlFederationEntityResolverDefinitionRecord,
    GraphqlRootFieldResolverDefinitionRecord, ResolverDefinitionId, ResolverDefinitionRecord,
    SelectionSetResolverExtensionDefinitionRecord, SubgraphId, TypeSystemDirectiveId, VirtualSubgraphId,
    builder::{
        Error, GraphBuilder,
        sdl::{self, SdlDefinition},
    },
};

use super::DirectivesIngester;

pub(super) fn generate(ingester: &mut DirectivesIngester<'_, '_>) -> Result<(), Error> {
    create_root_graphql_resolvers(ingester);
    create_extension_resolvers(ingester);
    create_apollo_federation_entity_resolvers(ingester)?;
    ingest_selection_set_resolvers(ingester)?;
    ingest_composite_schema_lookup(ingester)?;
    Ok(())
}

fn create_root_graphql_resolvers(ingester: &mut DirectivesIngester<'_, '_>) {
    for root_object_id in ingester.builder.root_object_ids.iter().copied() {
        let endpoint_ids = ingester.graph[root_object_id]
            .exists_in_subgraph_ids
            .iter()
            .filter_map(|id| id.as_graphql_endpoint())
            .collect::<Vec<_>>();
        let mut resolvers = Vec::new();
        for endpoint_id in endpoint_ids {
            let resolver =
                ResolverDefinitionRecord::GraphqlRootField(GraphqlRootFieldResolverDefinitionRecord { endpoint_id });
            let id = ingester.graph.resolver_definitions.len().into();
            ingester.builder.graph.resolver_definitions.push(resolver);
            resolvers.push((endpoint_id, id));
        }

        let field_ids = ingester.graph[root_object_id].field_ids;
        for field in &mut ingester.builder.graph[field_ids] {
            field.resolver_ids.extend(
                field
                    .exists_in_subgraph_ids
                    .iter()
                    .filter_map(|id| id.as_graphql_endpoint())
                    .filter_map(|id| {
                        resolvers.iter().find_map(
                            |(endpoint_id, resolver_id)| if id == *endpoint_id { Some(resolver_id) } else { None },
                        )
                    }),
            )
        }
    }
}

fn create_extension_resolvers(ingester: &mut DirectivesIngester<'_, '_>) {
    let graph = &mut ingester.builder.graph;
    for field in &mut graph.field_definitions {
        for id in &field.directive_ids {
            let &TypeSystemDirectiveId::Extension(id) = id else {
                continue;
            };
            let directive = &graph.extension_directives[usize::from(id)];
            if directive.ty.is_field_resolver() {
                let subgraph_id = directive.subgraph_id;
                if !field.exists_in_subgraph_ids.contains(&subgraph_id) {
                    field.exists_in_subgraph_ids.push(subgraph_id);
                }
                graph
                    .resolver_definitions
                    .push(ResolverDefinitionRecord::FieldResolverExtension(
                        FieldResolverExtensionDefinitionRecord { directive_id: id },
                    ));
                field
                    .resolver_ids
                    .push(ResolverDefinitionId::from(graph.resolver_definitions.len() - 1))
            }
        }
    }
}

fn create_apollo_federation_entity_resolvers(ingester: &mut DirectivesIngester<'_, '_>) -> Result<(), Error> {
    for ty in ingester.sdl_definitions.sites.values().copied() {
        let Some(entity) = ty.as_entity() else {
            continue;
        };

        let ext = ingester
            .sdl
            .type_extensions
            .get(entity.name())
            .map(Vec::as_slice)
            .unwrap_or_default();

        let field_ids = match entity.id() {
            EntityDefinitionId::Interface(id) => ingester.graph[id].field_ids,
            EntityDefinitionId::Object(id) => ingester.graph[id].field_ids,
        };

        for result in entity
            .directives()
            .chain(ext.iter().flat_map(|ext| ext.directives()))
            .filter_map(|dir| sdl::as_join_type(&dir))
        {
            let (join_type, span) = result?;
            let subgraph_id = ingester.subgraphs.try_get(join_type.graph, span)?;
            let Some(key_str) = join_type.key.filter(|key| !key.is_empty()) else {
                continue;
            };

            let key = ingester.parse_field_set(entity.id().into(), key_str).map_err(|err| {
                (
                    format!("At {}, invalid key FieldSet: {}", entity.to_site_string(ingester), err),
                    span,
                )
            })?;

            // Any field that is part of a key has to exist in the subgraph.
            let mut stack = vec![&key];
            while let Some(fields) = stack.pop() {
                for item in fields {
                    let id = ingester.selections[item.field_id].definition_id;
                    let field = &mut ingester.graph[id];
                    if !field.exists_in_subgraph_ids.contains(&subgraph_id) {
                        field.exists_in_subgraph_ids.push(subgraph_id);
                    }
                }
            }

            if join_type.resolvable {
                let Some(endpoint_id) = subgraph_id.as_graphql_endpoint() else {
                    continue;
                };
                let id = ingester.graph.resolver_definitions.len().into();

                for field_id in field_ids {
                    // If part of the key we can't be provided by this resolver.
                    if ingester.graph[field_id].exists_in_subgraph_ids.contains(&subgraph_id)
                        && key
                            .iter()
                            .all(|item| ingester.selections[item.field_id].definition_id != field_id)
                    {
                        ingester.graph[field_id].resolver_ids.push(id);
                    }
                }

                let resolver = ResolverDefinitionRecord::GraphqlFederationEntity(
                    GraphqlFederationEntityResolverDefinitionRecord {
                        key_fields_record: key,
                        endpoint_id,
                    },
                );
                ingester.graph.resolver_definitions.push(resolver);
            } else {
                ingester
                    .possible_composite_entity_keys
                    .entry((entity.id(), subgraph_id))
                    .or_default()
                    .push(super::PossibleCompositeEntityKey {
                        key,
                        key_str,
                        used_by: None,
                    });
            }
        }
    }

    Ok(())
}

fn ingest_selection_set_resolvers(ctx: &mut GraphBuilder<'_>) -> Result<(), String> {
    // Ensure they're not mixed with field resolvers.
    for resolver in &ctx.graph.resolver_definitions {
        if let Some(FieldResolverExtensionDefinitionRecord { directive_id }) = resolver.as_field_resolver_extension() {
            let subgraph_id = ctx.graph[*directive_id]
                .subgraph_id
                .as_virtual()
                .expect("should have failed at directive creation");
            if let Some(id) = ctx.virtual_subgraph_to_selection_set_resolver[usize::from(subgraph_id)] {
                return Err(format!(
                    "Selection Set Resolver extension {} cannot be mixed with other resolvers in subgraph '{}', found {}",
                    ctx[id].manifest.id,
                    ctx[ctx.subgraphs[subgraph_id].subgraph_name_id],
                    ctx[ctx.graph[*directive_id].extension_id].manifest.id
                ));
            }
        }
    }

    let field_ids_list = {
        let mut list = vec![ctx.graph[ctx.graph.root_operation_types_record.query_id].field_ids];
        if let Some(mutation_id) = ctx.graph.root_operation_types_record.mutation_id {
            list.push(ctx.graph[mutation_id].field_ids);
        }
        if let Some(subscription_id) = ctx.graph.root_operation_types_record.subscription_id {
            list.push(ctx.graph[subscription_id].field_ids);
        }
        list
    };
    let mut resolver_definitions = take(&mut ctx.graph.resolver_definitions);
    for (ix, extension_id) in take(&mut ctx.virtual_subgraph_to_selection_set_resolver)
        .into_iter()
        .enumerate()
    {
        let Some(extension_id) = extension_id else {
            continue;
        };
        let virtual_subgraph_id = VirtualSubgraphId::from(ix);
        let subgraph_id = SubgraphId::from(virtual_subgraph_id);

        for field_ids in &field_ids_list {
            for field in &mut ctx.graph[*field_ids] {
                if field.exists_in_subgraph_ids.contains(&subgraph_id) {
                    // Each field has its dedicated resolvers and they don't support batching
                    // multiple fields for now.
                    resolver_definitions.push(ResolverDefinitionRecord::SelectionSetResolverExtension(
                        SelectionSetResolverExtensionDefinitionRecord {
                            subgraph_id: virtual_subgraph_id,
                            extension_id,
                        },
                    ));
                    field.resolver_ids.push((resolver_definitions.len() - 1).into());
                }
            }
        }
    }
    ctx.graph.resolver_definitions = resolver_definitions;

    Ok(())
}

fn ingest_composite_schema_lookup(ingester: &mut DirectivesIngester<'_, '_>) -> Result<(), Error> {
    let query_object_id = ingester.graph.root_operation_types_record.query_id;
    for field_id in ingester.graph[query_object_id].field_ids {
        let Some(&SdlDefinition::FieldDefinition(field)) = ingester.sdl_definitions.get(&field_id.into()) else {
            // Introspection fields aren't part of the SDL.
            continue;
        };
        for directive in field.directives() {
            if directive.name() == "composite__lookup" {
                let sdl::LookupDirective { graph } = directive.deserialize().map_err(|err| {
                    (
                        format!(
                            "At {}, invalid composite__lookup directive: {}",
                            field.to_site_string(ingester),
                            err
                        ),
                        directive.arguments_span(),
                    )
                })?;
                let subgraph_id = ingester.subgraphs.try_get(graph, directive.arguments_span())?;
                ingester
                    .ingest_composite_lookup(field, subgraph_id)
                    .map_err(|err| err.with_span_if_absent(directive.arguments_span()))?
            }
        }
    }

    Ok(())
}
