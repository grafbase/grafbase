use cynic_parser_deser::ConstDeserializer as _;
use itertools::Itertools as _;

use crate::{
    DerivedFieldMappingRecord, DerivedFieldRecord, DirectiveSiteId, EntityDefinitionId, Graph, ResolverDefinitionId,
    ResolverDefinitionRecord, SubgraphId,
    builder::{
        BoundSelectedObjectField, DirectivesIngester, Error, PossibleCompositeEntityKey,
        sdl::{self, IsDirective},
    },
};

pub(super) fn ingest<'sdl>(
    ingester: &mut DirectivesIngester<'_, 'sdl>,
    def: sdl::FieldSdlDefinition<'sdl>,
    directive: sdl::Directive<'sdl>,
) -> Result<(), Error> {
    let sdl::DerivedDirective { graph } = directive.deserialize().map_err(|err| {
        (
            format!(
                "At {}, invalid composite__lookup directive: {}",
                def.to_site_string(ingester),
                err
            ),
            directive.arguments_span(),
        )
    })?;
    let subgraph_id = ingester.subgraphs.try_get(graph, directive.arguments_span())?;

    let Some(target_entity_id) = ingester.graph[def.id].ty_record.definition_id.as_entity() else {
        return Err((
            "@is can only be used on fields to compute an object/interface.",
            def.span(),
        )
            .into());
    };
    let target_field_ids = match target_entity_id {
        EntityDefinitionId::Interface(id) => ingester.graph[id].field_ids,
        EntityDefinitionId::Object(id) => ingester.graph[id].field_ids,
    };
    let source_id = ingester.graph[def.id].parent_entity_id;
    let Some(possible_keys) = ingester
        .possible_composite_entity_keys
        .get(&(target_entity_id, subgraph_id))
    else {
        let ty = ingester.sdl_definitions[&target_entity_id.into()].as_type().unwrap();
        return Err((
            format!(
                "Type {} doesn't define any keys with @key directive that may be used for @derive",
                ty.name()
            ),
            ty.span(),
        )
            .into());
    };

    let mapping_records = if let Some((
        is_directive,
        sdl::IsDirective {
            field: field_selection_map,
            ..
        },
    )) = ingester.sdl_definitions[&DirectiveSiteId::Field(def.id)]
        .directives()
        .filter(|dir| dir.name() == "composite__is")
        .map(|dir| {
            dir.deserialize::<IsDirective>()
                .map_err(|err| (format!("for associated @is directive: {err}"), dir.arguments_span()))
                .map(|args| (dir, args))
        })
        .filter_ok(|(_, args)| args.graph == graph)
        .next()
        .transpose()?
    {
        let mut mapping_records = Vec::new();
        let object = ingester
            .builder
            .parse_field_selection_map_for_derived_field(source_id, subgraph_id, def.id, field_selection_map)
            .map_err(|err| {
                (
                    format!("for associated @is directive: {err}"),
                    is_directive.arguments_span(),
                )
            })?
            .into_single()
            .ok_or_else(|| {
                (
                    "for associated @is directive, derived fields do not support multiple alternatives",
                    is_directive.arguments_span(),
                )
            })?
            .into_object()
            .ok_or_else(|| {
                (
                    "for associated @is directive, derived fields must be objects",
                    is_directive.arguments_span(),
                )
            })?;

        for BoundSelectedObjectField { field: to_id, value } in object.fields {
            let from_id = if let Some(value) = value {
                value
                    .into_single()
                    .and_then(|value| value.into_path())
                    .and_then(|path| path.into_single())
                    .ok_or_else(|| {
                        (
                            "Derived object fields can only be mapped to parent scalar/enum fields",
                            directive.arguments_span(),
                        )
                    })?
            } else {
                target_field_ids
                    .into_iter()
                    .find(|id| ingester.graph[*id].name_id == ingester.graph[to_id].name_id)
                    .unwrap()
            };
            mapping_records.push(DerivedFieldMappingRecord { from_id, to_id });
        }

        let mut found_matching_key = false;
        'keys: for PossibleCompositeEntityKey { key, .. } in possible_keys.iter() {
            if key.len() != mapping_records.len() {
                continue;
            }
            for item in key {
                if !item.subselection_record.is_empty() {
                    continue 'keys;
                }
                let id = ingester.selections[item.field_id].definition_id;
                if !mapping_records.iter().any(|record| record.to_id == id) {
                    continue 'keys;
                }
            }
            found_matching_key = true;
            break;
        }
        if !found_matching_key {
            return Err(("Derived field must match at least one @key", directive.arguments_span()).into());
        }
        mapping_records
    } else {
        let prefix = def.name();
        let source_field_ids = match source_id {
            EntityDefinitionId::Interface(id) => ingester.graph[id].field_ids,
            EntityDefinitionId::Object(id) => ingester.graph[id].field_ids,
        };
        let possible_fields = source_field_ids
            .into_iter()
            .filter(|id| *id != def.id)
            .filter_map(|id| {
                let field = &ingester.graph[id];
                ingester[field.name_id]
                    .strip_prefix(prefix)
                    .map(|suffix| (suffix.replace('_', "").to_lowercase(), id, field.ty_record))
            })
            .collect::<Vec<_>>();

        let mut mapping_records = Vec::new();
        let mut possible_mapping_records = Vec::new();
        'keys: for PossibleCompositeEntityKey { key, .. } in possible_keys.iter() {
            possible_mapping_records.clear();
            if possible_fields.len() < key.len() {
                continue;
            }

            for item in key {
                if !item.subselection_record.is_empty() {
                    continue 'keys;
                }
                let key_field_id = ingester.selections[item.field_id].definition_id;
                let key_field = &ingester.graph[key_field_id];
                let key_name = ingester[key_field.name_id].to_lowercase().replace('_', "");

                let mut matches = possible_fields
                    .iter()
                    .filter(|(name, _, ty)| *name == key_name && *ty == key_field.ty_record);

                // Find matching field by name and type
                if let (Some((_, parent_field_id, _)), None) = (matches.next(), matches.next()) {
                    possible_mapping_records.push(DerivedFieldMappingRecord {
                        from_id: *parent_field_id,
                        to_id: key_field_id,
                    });
                } else {
                    continue 'keys;
                }
            }
            mapping_records.append(&mut possible_mapping_records);
        }

        if mapping_records.is_empty() {
            return Err(("Derived field must match at least one @key", directive.arguments_span()).into());
        }
        mapping_records.sort_unstable();
        mapping_records.dedup();
        mapping_records
    };

    for id in target_field_ids {
        if mapping_records.iter().any(|record| record.to_id == id) {
            continue;
        }
        let field = &ingester.graph[id];
        if field.exists_in_subgraph_ids.contains(&subgraph_id)
            && !field
                .resolver_ids
                .iter()
                .any(|id| get_subgraph_id(&ingester.graph, *id) != subgraph_id)
        {
            return Err((
                format!(
                    "Field {}.{} is unprovidable for this @derive",
                    ingester[ingester.definition_name_id(source_id.into())],
                    ingester[field.name_id]
                ),
                directive.name_span(),
            )
                .into());
        }
    }

    assert!(
        ingester.graph[def.id].derived_ids.is_empty(),
        "Suppor derived on multiple subgraphs."
    );
    let parent_entity_id = ingester.graph[def.id].parent_entity_id;
    let start = ingester.graph.derived_fields.len();
    ingester.graph.derived_fields.push(DerivedFieldRecord {
        subgraph_id,
        parent_entity_id,
        mapping_records: mapping_records.clone(),
    });
    ingester.graph[def.id].derived_ids = (start..(start + 1)).into();

    Ok(())
}

fn get_subgraph_id(graph: &Graph, id: ResolverDefinitionId) -> SubgraphId {
    match &graph[id] {
        ResolverDefinitionRecord::FieldResolverExtension(record) => graph[record.directive_id].subgraph_id,
        ResolverDefinitionRecord::GraphqlFederationEntity(record) => record.endpoint_id.into(),
        ResolverDefinitionRecord::GraphqlRootField(record) => record.endpoint_id.into(),
        ResolverDefinitionRecord::Introspection => SubgraphId::Introspection,
        ResolverDefinitionRecord::Lookup(id) => get_subgraph_id(graph, graph[*id].resolver_id),
        ResolverDefinitionRecord::SelectionSetResolverExtension(record) => record.subgraph_id.into(),
    }
}
