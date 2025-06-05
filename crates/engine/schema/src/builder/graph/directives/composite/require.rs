use cynic_parser_deser::ConstDeserializer as _;
use itertools::Itertools as _;
use wrapping::Wrapping;

use crate::{
    ArgumentInjectionRecord, ArgumentValueInjection, DirectiveSiteId, FieldRequiresRecord, TypeRecord,
    builder::{
        DirectivesIngester, Error, graph::directives::composite::injection::create_requirements_and_injection, sdl,
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
        let (_, arg_id, field_selection_map, directive) = directives.next().unwrap();
        // All @require must be consistent
        let batch = field_selection_map.trim_start().starts_with('[');
        let source = if batch {
            TypeRecord {
                definition_id: parent_entity_id.into(),
                wrapping: Wrapping::default().non_null().list_non_null(),
            }
        } else {
            TypeRecord {
                definition_id: parent_entity_id.into(),
                wrapping: Wrapping::default().non_null(),
            }
        };
        let field_selection_map = ingester
            .parse_field_selection_map_for_argument(source, def.id, arg_id, field_selection_map)
            .map_err(|err| (err, directive.arguments_span()))?;

        let (mut requires, value_injection) = create_requirements_and_injection(ingester.builder, field_selection_map)?;
        let mut injections = vec![ArgumentInjectionRecord {
            definition_id: arg_id,
            value: ArgumentValueInjection::Value(value_injection),
        }];

        for (_, arg_id, field_selection_map, directive) in directives {
            let field_selection_map = ingester
                .parse_field_selection_map_for_argument(source, def.id, arg_id, field_selection_map)
                .map_err(|err| (err, directive.arguments_span()))?;
            let (arg_requires, value_injection) =
                create_requirements_and_injection(ingester.builder, field_selection_map)?;
            requires = requires.union(&arg_requires);
            injections.push(ArgumentInjectionRecord {
                definition_id: arg_id,
                value: ArgumentValueInjection::Value(value_injection),
            });
        }
        let injection_ids = ingester.builder.selections.push_argument_injections(injections);

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
    }

    todo!()
}
