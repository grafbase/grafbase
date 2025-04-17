use crate::{
    EntityDefinitionId, FieldResolverExtensionDefinitionRecord, GraphqlFederationEntityResolverDefinitionRecord,
    GraphqlRootFieldResolverDefinitionRecord, ResolverDefinitionId, ResolverDefinitionRecord, TypeSystemDirectiveId,
    builder::{
        Error,
        extension::LoadedExtensionOrCompositeSchema,
        sdl::{self, SdlDefinition},
    },
};

use super::DirectivesIngester;

pub(super) fn generate(ingester: &mut DirectivesIngester<'_, '_>) -> Result<(), Error> {
    create_root_graphql_resolvers(ingester);
    create_extension_resolvers(ingester);
    create_apollo_federation_entity_resolvers(ingester)?;
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
            let Some(endpoint_id) = subgraph_id.as_graphql_endpoint() else {
                continue;
            };
            let Some(key) = join_type.key.filter(|key| !key.is_empty()) else {
                continue;
            };

            let key = ingester.parse_field_set(entity.id().into(), key).map_err(|err| {
                (
                    format!("At {}, invalid key FieldSet: {}", entity.to_site_string(ingester), err),
                    span,
                )
            })?;

            // Any field that is part of a key has to exist in the subgraph.
            let mut stack = vec![&key];
            while let Some(fields) = stack.pop() {
                for item in fields {
                    let id = ingester.graph[item.field_id].definition_id;
                    let field = &mut ingester.graph[id];
                    if !field.exists_in_subgraph_ids.contains(&subgraph_id) {
                        field.exists_in_subgraph_ids.push(subgraph_id);
                    }
                }
            }

            if join_type.resolvable {
                let id = ingester.graph.resolver_definitions.len().into();

                for field_id in field_ids {
                    // If part of the key we can't be provided by this resolver.
                    if ingester.graph[field_id].exists_in_subgraph_ids.contains(&subgraph_id)
                        && key
                            .iter()
                            .all(|item| ingester.graph[item.field_id].definition_id != field_id)
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
            }
        }
    }

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
            if directive.name() == "extension__directive" {
                let dir = sdl::parse_extension_directive(directive)?;
                let LoadedExtensionOrCompositeSchema::CompositeSchema = ingester.extensions.get(dir.extension) else {
                    continue;
                };
                if matches!(dir.name, "lookup") {
                    let subgraph_id = ingester.subgraphs.try_get(dir.graph, directive.arguments_span())?;
                    ingester
                        .ingest_composite_schema_lookup(field, subgraph_id)
                        .map_err(|err| err.with_span_if_absent(directive.arguments_span()))?
                }
            }
        }
    }

    Ok(())
}
