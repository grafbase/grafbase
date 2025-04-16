use crate::{
    EntityDefinitionId, FieldDefinitionId, FieldSetRecord, Graph, InputValueDefinitionId, InputValueDefinitionRecord,
    LookupResolverDefinitionRecord, SubgraphId,
    builder::{DirectivesIngester, Error, sdl},
};

pub(super) fn ingest<'sdl>(
    ingester: &mut DirectivesIngester<'_, 'sdl>,
    field: sdl::FieldSdlDefinition<'sdl>,
    subgraph_id: SubgraphId,
) -> Result<(), Error> {
    let field_definition = &ingester.graph[field.id];
    let Some(entity_id) = field_definition.ty_record.definition_id.as_entity() else {
        return Err(("can only be used to return objects or interfaces.", field.span()).into());
    };

    let batch = match field_definition.ty_record.wrapping.list_wrappings().len() {
        0 => false,
        1 => true,
        _ => return Err(("output wrapping cannot be multiple lists.", field.span()).into()),
    };

    let Some(keys) = ingester.composite_entity_keys.get(&(entity_id, subgraph_id)) else {
        let ty = ingester.sdl_definitions[&entity_id.into()].as_type().unwrap();
        return Err((
            format!("output type {} doesn't define any keys with @key directive", ty.name()),
            ty.span(),
        )
            .into());
    };

    if batch {
        let mut batch_arg_matches = field_definition
            .argument_ids
            .into_iter()
            .filter_map(|id| {
                let arg = &ingester.graph[id];
                if arg.ty_record.wrapping.list_wrappings().len() != 1 {
                    return None;
                }
                let input_object_id = arg.ty_record.definition_id.as_input_object()?;
                let matching = find_matching_keys(
                    &ingester.graph,
                    keys,
                    &ingester.graph[ingester.graph[input_object_id].input_field_ids],
                );
                if !matching.is_empty() {
                    Some((id, matching))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let Some((batch_arg_id, mut key_matches)) = batch_arg_matches.pop() else {
            return Err("no batch argument matching a @key directive was found on the field".into());
        };

        if !batch_arg_matches.is_empty() {
            return Err("multiple batch argument matching a @key directive were found on the field".into());
        }

        let key = key_matches.pop().unwrap();
        if !key_matches.is_empty() {
            return Err(format!(
                "multiple matching @key directive found for the batch argument {}",
                ingester[ingester.graph[batch_arg_id].name_id]
            )
            .into());
        }

        add_lookup_entity_resolvers(
            &mut ingester.builder.graph,
            field.id,
            entity_id,
            key,
            Some(batch_arg_id),
        );
    } else {
        let mut matches = find_matching_keys(&ingester.graph, keys, &ingester.graph[field_definition.argument_ids]);
        let Some(key) = matches.pop() else {
            return Err("not matching @key directive was found on the output type".into());
        };

        if !matches.is_empty() {
            return Err("multiple matching @key directive found on the output type".into());
        }

        add_lookup_entity_resolvers(&mut ingester.builder.graph, field.id, entity_id, key, None);
    };

    Ok(())
}

fn find_matching_keys<'k>(
    graph: &Graph,
    keys: &'k [FieldSetRecord],
    inputs: &[InputValueDefinitionRecord],
) -> Vec<&'k FieldSetRecord> {
    let required_bitset: u64 = {
        let mut bitset: u64 = 0;
        for (i, arg) in inputs.iter().enumerate() {
            if arg.ty_record.wrapping.is_required() && arg.default_value_id.is_none() {
                bitset |= 1 << i;
            }
        }
        bitset
    };

    let mut matching = Vec::new();

    'keys: for key in keys {
        let mut required_bitset = required_bitset;
        for item in key {
            debug_assert!(item.subselection_record.is_empty());
            let field = &graph[item.field_id];
            let def = &graph[field.definition_id];
            let Some(pos) = inputs
                .iter()
                .position(|input| input.name_id == def.name_id && input.ty_record == def.ty_record)
            else {
                continue 'keys;
            };
            required_bitset &= !(1 << pos);
        }
        if required_bitset == 0 {
            matching.push(key);
        }
    }

    matching
}

fn add_lookup_entity_resolvers(
    graph: &mut Graph,
    lookup_field_id: FieldDefinitionId,
    output: EntityDefinitionId,
    key: &FieldSetRecord,
    batch_argument_id: Option<InputValueDefinitionId>,
) {
    let mut resolvers = Vec::new();
    for id in &graph.field_definitions[usize::from(lookup_field_id)].resolver_ids {
        resolvers.push(graph.resolver_definitions.len().into());
        graph.resolver_definitions.push(
            LookupResolverDefinitionRecord {
                key_record: key.clone(),
                batch_argument_id,
                field_id: lookup_field_id,
                resolver_id: *id,
            }
            .into(),
        );
    }

    let field_ids = match output {
        EntityDefinitionId::Object(id) => graph[id].field_ids,
        EntityDefinitionId::Interface(id) => graph[id].field_ids,
    };
    for field_id in field_ids {
        // If part of the key we can't be provided by this resolver.
        if key.iter().all(|item| graph[item.field_id].definition_id != field_id) {
            graph[field_id].resolver_ids.extend_from_slice(&resolvers);
        }
    }
}
