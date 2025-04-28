use id_newtypes::IdRange;

use crate::{
    EntityDefinitionId, FieldDefinitionId, FieldSetRecord, Graph, InputValueDefinitionId, InputValueInjection,
    InputValueInjectionId, LookupResolverDefinitionId, LookupResolverDefinitionRecord, SubgraphId, ValueInjection,
    builder::{DirectivesIngester, Error, graph::selections::SelectionsBuilder, sdl},
};

pub(super) fn ingest<'sdl>(
    ingester: &mut DirectivesIngester<'_, 'sdl>,
    field: sdl::FieldSdlDefinition<'sdl>,
    subgraph_id: SubgraphId,
) -> Result<(), Error> {
    let graph = &ingester.builder.graph;
    let field_definition = &graph[field.id];
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

    let selections = &mut ingester.builder.selections;
    let mut found_lookup_key = None;
    if batch {
        for argument_id in field_definition.argument_ids {
            let arg = &graph[argument_id];
            if arg.ty_record.wrapping.list_wrappings().len() != 1 {
                continue;
            }
            if let Some(input_object_id) = arg.ty_record.definition_id.as_input_object() {
                for key in keys {
                    let state = selections.current_injection_state();
                    let Some(mapping) = try_build_input_value_injections(
                        graph,
                        selections,
                        key,
                        graph[input_object_id].input_field_ids,
                    ) else {
                        selections.reset_injection_state(state);
                        continue;
                    };
                    if found_lookup_key.is_some() {
                        return Err("multiple matching @key directive found on the output type".into());
                    }
                    let injection_ids = selections.push_input_value_injections(&mut vec![InputValueInjection {
                        definition_id: argument_id,
                        injection: ValueInjection::Object(mapping),
                    }]);
                    found_lookup_key = Some((key, true, injection_ids))
                }
            } else {
                for key in keys {
                    if key.len() == 1 && key[0].subselection_record.is_empty() {
                        let def = &graph[selections[key[0].field_id].definition_id];
                        if arg.ty_record.definition_id == def.ty_record.definition_id
                            && !def.ty_record.wrapping.is_list()
                        {
                            if found_lookup_key.is_some() {
                                return Err("multiple matching @key directive found on the output type".into());
                            }
                            let injection_ids =
                                selections.push_input_value_injections(&mut vec![InputValueInjection {
                                    definition_id: argument_id,
                                    injection: ValueInjection::Select {
                                        field_id: key[0].field_id,
                                        next: None,
                                    },
                                }]);
                            found_lookup_key = Some((key, true, injection_ids));
                        }
                    }
                }
            }
        }
    } else {
        for key in keys {
            let state = selections.current_injection_state();
            let Some(mapping) = try_build_input_value_injections(graph, selections, key, field_definition.argument_ids)
            else {
                selections.reset_injection_state(state);
                continue;
            };
            if found_lookup_key.is_some() {
                return Err("multiple matching @key directive found on the output type".into());
            }
            found_lookup_key = Some((key, false, mapping));
        }
    };

    let Some((key, batch, injection_ids)) = found_lookup_key else {
        return Err("not matching @key directive was found on the output type".into());
    };

    add_lookup_entity_resolvers(
        &mut ingester.builder.graph,
        &ingester.builder.selections,
        field.id,
        entity_id,
        key,
        batch,
        injection_ids,
    );

    Ok(())
}

fn try_build_input_value_injections(
    graph: &Graph,
    selections: &mut SelectionsBuilder,
    key: &FieldSetRecord,
    input_ids: IdRange<InputValueDefinitionId>,
) -> Option<IdRange<InputValueInjectionId>> {
    assert!(key.len() < 64, "Cannot handle keys with 64 fields or more.");
    let mut missing: u64 = (1 << key.len()) - 1;

    let mut input_values = Vec::new();
    for input_id in input_ids {
        let input = &graph[input_id];
        if let Some(pos) = key.iter().position(|item| {
            let field = &selections[item.field_id];
            let def = &graph[field.definition_id];
            input.name_id == def.name_id && input.ty_record == def.ty_record
        }) {
            missing &= !(1 << pos);

            input_values.push(InputValueInjection {
                definition_id: input_id,
                injection: ValueInjection::Select {
                    field_id: key[0].field_id,
                    next: if key[pos].subselection_record.is_empty() {
                        None
                    } else {
                        let input_object_id = input.ty_record.definition_id.as_input_object()?;
                        let range = try_build_input_value_injections(
                            graph,
                            selections,
                            &key[pos].subselection_record,
                            graph[input_object_id].input_field_ids,
                        )?;
                        Some(selections.push_value_injection(ValueInjection::Object(range)))
                    },
                },
            });
        } else if let Some(default_value_id) = input.default_value_id {
            input_values.push(InputValueInjection {
                definition_id: input_id,
                injection: ValueInjection::Const(default_value_id),
            });
        } else if input.ty_record.wrapping.is_required() {
            return None;
        }
    }

    if missing != 0 {
        return None;
    }

    let range = selections.push_input_value_injections(&mut input_values);
    Some(range)
}

fn add_lookup_entity_resolvers(
    graph: &mut Graph,
    selections: &SelectionsBuilder,
    lookup_field_id: FieldDefinitionId,
    output: EntityDefinitionId,
    key: &FieldSetRecord,
    batch: bool,
    injection_ids: IdRange<InputValueInjectionId>,
) {
    let mut resolvers = Vec::new();
    for &resolver_id in &graph.field_definitions[usize::from(lookup_field_id)].resolver_ids {
        let lookup_resolver_id = LookupResolverDefinitionId::from(graph.lookup_resolver_definitions.len());
        graph.lookup_resolver_definitions.push(LookupResolverDefinitionRecord {
            key_record: key.clone(),
            field_definition_id: lookup_field_id,
            resolver_id,
            batch,
            injection_ids,
        });
        resolvers.push(graph.resolver_definitions.len().into());
        graph.resolver_definitions.push(lookup_resolver_id.into());
    }

    let field_ids = match output {
        EntityDefinitionId::Object(id) => graph[id].field_ids,
        EntityDefinitionId::Interface(id) => graph[id].field_ids,
    };
    for field_id in field_ids {
        // If part of the key we can't be provided by this resolver.
        if key
            .iter()
            .all(|item| selections[item.field_id].definition_id != field_id)
        {
            graph[field_id].resolver_ids.extend_from_slice(&resolvers);
        }
    }
}
