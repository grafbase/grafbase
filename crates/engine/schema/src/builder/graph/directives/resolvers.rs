use std::{mem::take, ops::DerefMut};

use crate::{
    EntityDefinitionId, ExtensionDirectiveType, ExtensionResolverDefinitionRecord,
    FieldResolverExtensionDefinitionRecord, GraphqlFederationEntityResolverDefinitionRecord,
    GraphqlRootFieldResolverDefinitionRecord, ResolverDefinitionId, ResolverDefinitionRecord,
    SelectionSetResolverExtensionDefinitionRecord, SubgraphId, TypeSystemDirectiveId, VirtualSubgraphId,
    builder::{
        Error,
        sdl::{self, SdlDefinition},
    },
};

use super::DirectivesIngester;

pub(super) fn generate(ingester: &mut DirectivesIngester<'_, '_>) {
    create_root_graphql_resolvers(ingester);
    create_extension_resolvers(ingester);
    create_apollo_federation_entity_resolvers(ingester);
    ingest_composite_schema_lookup(ingester);
}

fn create_root_graphql_resolvers(ingester: &mut DirectivesIngester<'_, '_>) {
    for root_object_id in ingester.builder.root_object_ids.iter().copied() {
        let endpoint_ids = ingester.graph[root_object_id]
            .exists_in_subgraph_ids
            .iter()
            .filter_map(|id| id.as_graphql())
            .collect::<Vec<_>>();
        let mut resolvers = Vec::new();
        for endpoint_id in endpoint_ids {
            let resolver = ResolverDefinitionRecord::GraphqlRootField(GraphqlRootFieldResolverDefinitionRecord {
                subgraph_id: endpoint_id,
            });
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
                    .filter_map(|id| id.as_graphql())
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
            match directive.ty {
                ExtensionDirectiveType::FieldResolver => {
                    let subgraph_id = directive.subgraph_id;
                    if !directive.subgraph_id.is_virtual() {
                        ingester.errors.push(
                            Error::new("Field resolver extensions can only be used with virtual subgraphs (subgraphs without a URL).")
                        );
                        continue;
                    }
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
                ExtensionDirectiveType::Resolver => {
                    let subgraph_id = directive.subgraph_id;
                    let virtual_subgraph_id = match directive.subgraph_id.as_virtual() {
                        Some(id) => id,
                        None => {
                            ingester.errors.push(
                                Error::new("Resolver extensions can only be used with virtual subgraphs (subgraphs without a URL).")
                            );
                            continue;
                        }
                    };
                    if !field.exists_in_subgraph_ids.contains(&subgraph_id) {
                        field.exists_in_subgraph_ids.push(subgraph_id);
                    }
                    graph.resolver_definitions.push(ResolverDefinitionRecord::Extension(
                        ExtensionResolverDefinitionRecord {
                            directive_id: id,
                            subgraph_id: virtual_subgraph_id,
                            extension_id: directive.extension_id,
                            guest_batch: false,
                        },
                    ));
                    field
                        .resolver_ids
                        .push(ResolverDefinitionId::from(graph.resolver_definitions.len() - 1))
                }
                _ => {}
            }
        }
    }

    // Validate and collect errors, then process rest of the function
    let mut errors = std::mem::take(&mut ingester.errors);
    let builder = ingester.deref_mut();
    // Ensure they're not mixed with field resolvers.
    for resolver in &builder.graph.resolver_definitions {
        if let Some(FieldResolverExtensionDefinitionRecord { directive_id }) = resolver.as_field_resolver_extension() {
            let Some(subgraph_id) = builder.graph[*directive_id].subgraph_id.as_virtual() else {
                // Already validated above that field resolvers are only on virtual subgraphs
                continue;
            };
            if let Some(id) = builder.virtual_subgraph_to_selection_set_resolver[usize::from(subgraph_id)] {
                errors.push(Error::new(format!(
                    "Selection Set Resolver extension {} cannot be mixed with other resolvers in subgraph '{}', found {}",
                    builder[id].manifest.id,
                    builder[builder.subgraphs[subgraph_id].name_id],
                    builder[builder.graph[*directive_id].extension_id].manifest.id
                )));
            }
        }
    }
    ingester.errors = errors;

    let builder = ingester.deref_mut();
    let field_ids_list = {
        let mut list = vec![builder.graph[builder.graph.root_operation_types_record.query_id].field_ids];
        if let Some(mutation_id) = builder.graph.root_operation_types_record.mutation_id {
            list.push(builder.graph[mutation_id].field_ids);
        }
        if let Some(subscription_id) = builder.graph.root_operation_types_record.subscription_id {
            list.push(builder.graph[subscription_id].field_ids);
        }
        list
    };
    let mut resolver_definitions = take(&mut builder.graph.resolver_definitions);
    for (ix, extension_id) in take(&mut builder.virtual_subgraph_to_selection_set_resolver)
        .into_iter()
        .enumerate()
    {
        let Some(extension_id) = extension_id else {
            continue;
        };
        let virtual_subgraph_id = VirtualSubgraphId::from(ix);
        let subgraph_id = SubgraphId::from(virtual_subgraph_id);

        for field_ids in &field_ids_list {
            for field in &mut builder.graph[*field_ids] {
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
    builder.graph.resolver_definitions = resolver_definitions;
}

fn create_apollo_federation_entity_resolvers(ingester: &mut DirectivesIngester<'_, '_>) {
    for ty in ingester.definitions.clone().site_id_to_sdl.values().copied() {
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
            let (join_type, span) = match result {
                Ok(v) => v,
                Err(err) => {
                    ingester.errors.push(err);
                    continue;
                }
            };
            let subgraph_id = match ingester.subgraphs.try_get(join_type.graph, span) {
                Ok(id) => id,
                Err(err) => {
                    ingester.errors.push(err);
                    continue;
                }
            };
            let Some(key_str) = join_type.key.filter(|key| !key.is_empty()) else {
                continue;
            };

            let key = match ingester.parse_field_set(entity.id().into(), key_str) {
                Ok(k) => k,
                Err(err) => {
                    ingester.errors.push(
                        Error::new(format!(
                            "At {}, invalid key FieldSet: {}",
                            entity.to_site_string(ingester),
                            err
                        ))
                        .span(span),
                    );
                    continue;
                }
            };

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
                let Some(endpoint_id) = subgraph_id.as_graphql() else {
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
                        subgraph_id: endpoint_id,
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
}

fn ingest_composite_schema_lookup(ingester: &mut DirectivesIngester<'_, '_>) {
    let query_object_id = ingester.graph.root_operation_types_record.query_id;
    for field_id in ingester.graph[query_object_id].field_ids {
        let Some(&SdlDefinition::FieldDefinition(field)) = ingester.definitions.site_id_to_sdl.get(&field_id.into())
        else {
            // Introspection fields aren't part of the SDL.
            continue;
        };
        for directive in field.directives() {
            if directive.name() == "composite__lookup"
                && let Err(err) = ingester.ingest_composite_lookup(field, directive)
            {
                ingester.errors.push(err.span_if_absent(directive.arguments_span()));
            }
        }
    }
}
