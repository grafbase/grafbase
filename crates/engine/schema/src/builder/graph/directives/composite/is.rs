use cynic_parser_deser::ConstDeserializer as _;

use crate::{
    ComputedFieldRecord, ComputedObjectRecord, EntityDefinitionId,
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

    let mut field_records: Vec<ComputedFieldRecord> = Vec::new();
    for BoundSelectedObjectField {
        field: target_id,
        value,
    } in object.fields
    {
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
                .find(|id| ingester.graph[*id].name_id == ingester.graph[target_id].name_id)
                .unwrap()
        };
        field_records.push(ComputedFieldRecord { from_id, target_id });
    }
    let computed = &mut ingester.graph[def.id].computed_records;
    if computed.iter().any(|c| c.subgraph_id == subgraph_id) {
        return Err((
            "Multiple @composite__is were used in the same subgraph",
            dir.name_span(),
        )
            .into());
    }
    computed.push(ComputedObjectRecord {
        subgraph_id,
        field_records,
    });
    Ok(())
}
