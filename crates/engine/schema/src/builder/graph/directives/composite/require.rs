use cynic_parser_deser::ConstDeserializer as _;
use itertools::Itertools as _;
use wrapping::Wrapping;

use crate::{
    DirectiveSiteId, FieldRequiresRecord, ResolverDefinitionRecord, SubgraphId, TypeRecord,
    builder::{
        DirectivesIngester, Error, graph::directives::composite::injection::create_requirements_and_injections, sdl,
    },
};

pub(super) fn ingest_field<'sdl>(
    ingester: &mut DirectivesIngester<'_, 'sdl>,
    def: sdl::FieldSdlDefinition<'sdl>,
) -> Result<(), Error> {
    let mut require_directives = Vec::new();
    for arg_id in ingester.graph[def.id].argument_ids {
        for directive in ingester.builder.definitions.site_id_to_sdl[&DirectiveSiteId::InputValue(arg_id)].directives()
        {
            if directive.name() != "composite__require" {
                continue;
            }
            let sdl::RequireDirective {
                graph: subgraph_name,
                field: field_selection_map,
            } = directive.deserialize().map_err(|err| {
                (
                    format!(
                        "At {}, invalid composite__require directive: {}",
                        def.to_site_string(ingester),
                        err
                    ),
                    directive.arguments_span(),
                )
            })?;
            let subgraph_id = ingester.subgraphs.try_get(subgraph_name, directive.arguments_span())?;
            require_directives.push((subgraph_id, arg_id, field_selection_map, directive));
        }
    }
    if require_directives.is_empty() {
        return Ok(());
    }

    let graph = &ingester.builder.graph;
    let parent_entity_id = graph[def.id].parent_entity_id;

    require_directives.sort_unstable_by_key(|(subgraph_id, _, _, _)| *subgraph_id);
    for (subgraph_id, mut directives) in require_directives
        .into_iter()
        .chunk_by(|(subgraph_id, _, _, _)| *subgraph_id)
        .into_iter()
    {
        let (_, first_arg_id, first_field_selection_map, directive) = directives.next().unwrap();
        // All @require must be consistent. Requires a bit of effort to detect it, so we just try
        // for now.
        let single_source = TypeRecord {
            definition_id: parent_entity_id.into(),
            wrapping: Wrapping::default().non_null(),
        };
        let batch_source = TypeRecord {
            definition_id: parent_entity_id.into(),
            wrapping: Wrapping::default().non_null().list_non_null(),
        };

        let (source, first_value) = if first_field_selection_map.contains('[') {
            // Probably batch
            match ingester.parse_field_selection_map_for_argument(
                batch_source,
                subgraph_id,
                first_arg_id,
                first_field_selection_map,
            ) {
                Ok(field_selection_map) => (batch_source, field_selection_map),
                Err(batch_err) => {
                    match ingester.parse_field_selection_map_for_argument(
                        single_source,
                        subgraph_id,
                        first_arg_id,
                        first_field_selection_map,
                    ) {
                        Ok(field_selection_map) => (single_source, field_selection_map),
                        Err(single_err) => {
                            let message = format!(
                                "Could not infer whether to batch requirements or not.\nBatch error: {batch_err}\nSingle error: {single_err}"
                            );
                            return Err((message, directive.arguments_span()).into());
                        }
                    }
                }
            }
        } else {
            match ingester.parse_field_selection_map_for_argument(
                single_source,
                subgraph_id,
                first_arg_id,
                first_field_selection_map,
            ) {
                Ok(field_selection_map) => (single_source, field_selection_map),
                Err(single_err) => {
                    match ingester.parse_field_selection_map_for_argument(
                        batch_source,
                        subgraph_id,
                        first_arg_id,
                        first_field_selection_map,
                    ) {
                        Ok(field_selection_map) => (batch_source, field_selection_map),
                        Err(batch_err) => {
                            let message = format!(
                                "Could not infer whether to batch requirements or not.\nBatch error: {batch_err}\nSingle error: {single_err}"
                            );
                            return Err((message, directive.arguments_span()).into());
                        }
                    }
                }
            }
        };
        let mut injections = vec![(first_arg_id, first_value)];
        for (_, arg_id, field_selection_map, directive) in directives {
            let value = ingester
                .parse_field_selection_map_for_argument(source, subgraph_id, arg_id, field_selection_map)
                .map_err(|err| (err, directive.arguments_span()))?;
            injections.push((arg_id, value));
        }

        let (requires, arguments) = create_requirements_and_injections(ingester.builder, injections)?;
        let injection_ids = ingester.builder.selections.push_argument_injections(arguments);

        if let Some(field_requires) = ingester.builder.graph[def.id]
            .requires_records
            .iter_mut()
            .find(|req| req.subgraph_id == subgraph_id)
        {
            debug_assert!(field_requires.injection_ids.is_empty());
            field_requires.field_set_record = field_requires.field_set_record.union(&requires);
            field_requires.injection_ids = injection_ids;
        } else {
            ingester.builder.graph[def.id]
                .requires_records
                .push(FieldRequiresRecord {
                    subgraph_id,
                    field_set_record: requires,
                    injection_ids,
                });
        }
        for id in &ingester.builder.graph.field_definitions[usize::from(def.id)].resolver_ids {
            let ResolverDefinitionRecord::Extension(record) =
                &mut ingester.builder.graph.resolver_definitions[usize::from(*id)]
            else {
                continue;
            };
            if SubgraphId::from(record.subgraph_id) == subgraph_id {
                record.guest_batch = source.wrapping.is_list();
            }
        }
    }

    Ok(())
}
