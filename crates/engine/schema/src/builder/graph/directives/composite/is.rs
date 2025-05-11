use cynic_parser_deser::ConstDeserializer as _;

use crate::{
    DerivedFieldMappingRecord, DerivedFieldRecord, EntityDefinitionId,
    builder::{BoundSelectedObjectField, DirectivesIngester, Error, sdl},
};

pub(super) fn ingest_field<'sdl>(
    ingester: &mut DirectivesIngester<'_, 'sdl>,
    def: sdl::FieldSdlDefinition<'sdl>,
    dir: sdl::Directive<'sdl>,
) -> Result<(), Error> {
    let sdl::IsDirective {
        graph,
        field: field_selection_map,
    } = dir
        .deserialize()
        .map_err(|err| (err.to_string(), dir.arguments_span()))?;
    let subgraph_id = ingester.subgraphs.try_get(graph, dir.arguments_span())?;

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

    let output = ingester.graph[def.id].parent_entity_id;
    let object = ingester
        .parse_field_selection_map_for_field(output, subgraph_id, def.id, field_selection_map)?
        .into_single()
        .ok_or_else(|| {
            (
                "Computed fields do not support multiple alternatives",
                dir.arguments_span(),
            )
        })?
        .into_object()
        .ok_or_else(|| ("Computed fields must be objects", dir.arguments_span()))?;

    let mut mapping_records = Vec::new();
    for BoundSelectedObjectField { field: to_id, value } in object.fields {
        let from_id = if let Some(value) = value {
            value
                .into_single()
                .and_then(|value| value.into_path())
                .and_then(|path| path.into_single())
                .ok_or_else(|| {
                    (
                        "Computed object fields can only be mapped to parent scalar/enum fields",
                        dir.arguments_span(),
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
