use crate::{
    SubgraphId,
    builder::{DirectivesIngester, Error, sdl},
};

#[derive(serde::Deserialize)]
pub(super) struct Arguments<'a> {
    #[serde(borrow)]
    fields: &'a str,
}

pub(super) fn ingest<'sdl>(
    ingester: &mut DirectivesIngester<'_, 'sdl>,
    entity: sdl::EntitySdlDefinition<'sdl>,
    subgraph_id: SubgraphId,
    arguments: Arguments<'_>,
) -> Result<(), Error> {
    let fields = ingester
        .parse_field_set(entity.id().into(), arguments.fields)
        .map_err(|err| format!("invalid SelectionSet for argument 'fields': {}", err))?;

    let mut stack = vec![&fields];
    while let Some(field_set) = stack.pop() {
        for item in field_set {
            let field = &ingester.graph[item.field_id];
            if !field.sorted_argument_ids.is_empty() {
                return Err(format!(
                    "invalid SelectionSet for argument 'fields', cannot use field '{}' having arguments",
                    ingester[ingester.graph[field.definition_id].name_id]
                )
                .into());
            }
            stack.push(&item.subselection_record);
        }
    }

    // Temporary limitation
    for item in &fields {
        if !item.subselection_record.is_empty() {
            return Err(format!(
                "cannot use nested selection sets for field {} in @key for now",
                ingester[ingester.graph[ingester.graph[item.field_id].definition_id].name_id]
            )
            .into());
        }
    }

    ingester
        .composite_entity_keys
        .entry((entity.id(), subgraph_id))
        .or_default()
        .push(fields);
    Ok(())
}
