use id_newtypes::IdRange;

use crate::{
    ArgumentInjectionId, ArgumentInjectionRecord, ArgumentValueInjection, EntityDefinitionId, FieldDefinitionId,
    FieldDefinitionRecord, FieldSetItemRecord, FieldSetRecord, Graph, InputValueDefinitionId,
    InputValueDefinitionRecord, KeyValueInjectionRecord, LookupResolverDefinitionId, LookupResolverDefinitionRecord,
    SubgraphId, TypeDefinitionId, ValueInjection,
    builder::{
        DirectivesIngester, Error,
        context::BuildContext,
        graph::{directives::PossibleCompositeEntityKey, selections::SelectionsBuilder},
        sdl,
    },
};

#[tracing::instrument(name = "ingest_composite_loopy", fields(field = %field.to_site_string(ingester)), skip_all)]
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

    let Some(possible_keys) = ingester
        .possible_composite_entity_keys
        .get_mut(&(entity_id, subgraph_id))
    else {
        let ty = ingester.sdl_definitions[&entity_id.into()].as_type().unwrap();
        return Err((
            format!("output type {} doesn't define any keys with @key directive", ty.name()),
            ty.span(),
        )
            .into());
    };

    let mut lookup_keys = Vec::new();
    let mut builder = ValueInjectionBuilder {
        ctx: &ingester.builder.ctx,
        graph,
        last_selections_injection_state: ingester.builder.selections.current_injection_state(),
        selections: &mut ingester.builder.selections,
    };
    for PossibleCompositeEntityKey { key, key_str, used_by } in possible_keys {
        let span = tracing::debug_span!("match_key", key = %key_str);
        let _enter = span.enter();
        let mut candidates = builder.try_build_arguments_value_injections(batch, key, field_definition.argument_ids);
        let Some(candidate) = candidates.pop() else {
            tracing::debug!("No candidiate found");
            continue;
        };
        if !candidates.is_empty() {
            tracing::debug!("Multiple candidiates found, skipping key");
            builder.reset();
            continue;
        }
        builder.save();

        if let Some(used_by) = used_by {
            return Err((
                format!(
                    "matching a key already used by a separate @lookup field: {}",
                    used_by.to_site_string(ingester)
                ),
                field.span(),
            )
                .into());
        }
        *used_by = Some(field);
        lookup_keys.push((key.clone(), candidate));
    }

    if lookup_keys.is_empty() {
        return Err(("no matching @key directive was found", field.span()).into());
    };

    add_lookup_entity_resolvers(
        &mut ingester.builder.graph,
        &ingester.builder.selections,
        field.id,
        entity_id,
        batch,
        lookup_keys,
    );

    Ok(())
}

fn add_lookup_entity_resolvers(
    graph: &mut Graph,
    selections: &SelectionsBuilder,
    lookup_field_id: FieldDefinitionId,
    output: EntityDefinitionId,
    batch: bool,
    lookup_keys: Vec<(FieldSetRecord, IdRange<ArgumentInjectionId>)>,
) {
    let mut resolvers = Vec::new();
    for (key, injection_ids) in lookup_keys {
        debug_assert!(resolvers.is_empty());
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
        resolvers.clear();
    }
}

struct ValueInjectionBuilder<'a> {
    ctx: &'a BuildContext<'a>,
    graph: &'a Graph,
    selections: &'a mut SelectionsBuilder,
    last_selections_injection_state: [usize; 4],
}

impl ValueInjectionBuilder<'_> {
    fn save(&mut self) {
        self.last_selections_injection_state = self.selections.current_injection_state();
    }

    fn reset(&mut self) {
        self.selections
            .reset_injection_state(self.last_selections_injection_state);
    }

    fn try_build_arguments_value_injections(
        &mut self,
        batch: bool,
        key: &FieldSetRecord,
        argument_ids: IdRange<InputValueDefinitionId>,
    ) -> Vec<IdRange<ArgumentInjectionId>> {
        self.last_selections_injection_state = self.selections.current_injection_state();
        let mut candidates = Vec::new();

        // Try direct match against arguments
        // We're not supporting cases like `lookup(key1: [ID!], key2: [ID!])` for composite keys.
        // We could, but means creating two lists which can't optimize for.
        if key.len() == 1 || !batch {
            tracing::trace!("Trying to match against arguments directly.");
            candidates.extend(self.try_build_arguments_injections(batch, key, argument_ids))
        }

        let is_required_arg_bitset: u64 = {
            let mut bitset = 0;
            for (i, arg) in self.graph[argument_ids].iter().enumerate() {
                if arg.ty_record.is_required() {
                    // More than one required input means we'll never find a single nested input object
                    // we can use.
                    if bitset != 0 {
                        return candidates;
                    }
                    bitset |= 1 << i;
                }
            }
            bitset
        };

        tracing::trace!("Trying to match against a nested input object.");
        // Try with a nested input object
        for (i, arg_id) in argument_ids.into_iter().enumerate() {
            let arg = &self.graph[arg_id];
            let span = tracing::debug_span!("match_argument", key = %self.ctx[arg.name_id]);
            let _enter = span.enter();
            let Some(input_object_id) = arg.ty_record.definition_id.as_input_object() else {
                continue;
            };
            if (is_required_arg_bitset & !(1 << i)) != 0 {
                tracing::trace!("There exists another required argument.");
                // There exist a separate required input, so can't use this argument for the key.
                continue;
            }

            let input_object = &self.graph[input_object_id];

            if input_object.is_one_of {
                if arg.ty_record.wrapping.is_list() {
                    continue;
                }
                if key.len() == 1 {
                    tracing::trace!("Trying to match with oneof input object for a key having a single field.");
                    if let Some(value) = self.try_build_input_value_injections(batch, key, input_object.input_field_ids)
                    {
                        debug_assert_eq!(value.len(), 1);
                        let (input_field_id, value) = value.into_iter().next().unwrap();
                        let value = self
                            .selections
                            .push_argument_value_injection(ArgumentValueInjection::Value(value));
                        let range = self.selections.push_argument_injections([ArgumentInjectionRecord {
                            definition_id: arg_id,
                            value: ArgumentValueInjection::Nested {
                                key: self.graph[input_field_id].name_id,
                                value,
                            },
                        }]);
                        candidates.push(range)
                    }
                } else {
                    tracing::trace!("Trying to match with oneof input object for a key having multiple fields.");
                    for oneof_field_id in input_object.input_field_ids {
                        let oneof_field = &self.graph[oneof_field_id];
                        let Some(nested_input_object_id) = oneof_field.ty_record.definition_id.as_input_object() else {
                            continue;
                        };
                        if !matches!(
                            (batch, oneof_field.ty_record.wrapping.list_wrappings().len()),
                            (true, 1) | (false, 0)
                        ) {
                            continue;
                        }
                        if let Some(value) = self.try_build_object_injections(
                            false,
                            key,
                            self.graph[nested_input_object_id].input_field_ids,
                        ) {
                            let value = self
                                .selections
                                .push_argument_value_injection(ArgumentValueInjection::Value(value));
                            let range = self.selections.push_argument_injections([ArgumentInjectionRecord {
                                definition_id: arg_id,
                                value: ArgumentValueInjection::Nested {
                                    key: oneof_field.name_id,
                                    value,
                                },
                            }]);
                            candidates.push(range)
                        }
                    }
                }
            } else if key.len() > 1 {
                tracing::trace!("Trying to match with nested object for a key having multiple fields.");
                if !matches!(
                    (batch, arg.ty_record.wrapping.list_wrappings().len()),
                    (true, 1) | (false, 0)
                ) {
                    continue;
                }

                if let Some(value) = self.try_build_object_injections(false, key, input_object.input_field_ids) {
                    let range = self.selections.push_argument_injections([ArgumentInjectionRecord {
                        definition_id: arg_id,
                        value: ArgumentValueInjection::Value(value),
                    }]);
                    candidates.push(range)
                }
            }
        }

        candidates
    }

    fn try_build_arguments_injections(
        &mut self,
        batch: bool,
        key: &FieldSetRecord,
        argument_ids: IdRange<InputValueDefinitionId>,
    ) -> Option<IdRange<ArgumentInjectionId>> {
        let arguments = self.try_build_input_value_injections(batch, key, argument_ids)?;
        Some(
            self.selections
                .push_argument_injections(arguments.into_iter().map(|(definition_id, value)| {
                    ArgumentInjectionRecord {
                        definition_id,
                        value: ArgumentValueInjection::Value(value),
                    }
                })),
        )
    }

    fn try_build_object_injections(
        &mut self,
        batch: bool,
        key: &FieldSetRecord,
        input_field_ids: IdRange<InputValueDefinitionId>,
    ) -> Option<ValueInjection> {
        let fields = self.try_build_input_value_injections(batch, key, input_field_ids)?;
        let range = self
            .selections
            .push_key_value_injections(fields.into_iter().map(|(def_id, value)| KeyValueInjectionRecord {
                key_id: self.graph[def_id].name_id,
                value,
            }));
        Some(ValueInjection::Object(range))
    }

    fn try_build_input_value_injections(
        &mut self,
        batch: bool,
        key: &FieldSetRecord,
        input_ids: IdRange<InputValueDefinitionId>,
    ) -> Option<Vec<(InputValueDefinitionId, ValueInjection)>> {
        assert!(key.len() < 64, "Cannot handle keys with 64 fields or more.");
        let mut missing: u64 = (1 << key.len()) - 1;

        let mut input_values = Vec::new();
        for input_id in input_ids {
            let input = &self.graph[input_id];
            if let Some(pos) = key.iter().position(|item| {
                let field = &self.graph[self.selections[item.field_id].definition_id];
                if !can_inject_field_into_input(field, input, batch) {
                    tracing::trace!(
                        "Field {} cannot be injected into input {}",
                        self.ctx[field.name_id],
                        self.ctx[input.name_id]
                    );
                    return false;
                }
                // Either name matches or the types are unique in both input & key and thus no other key/input pair could
                // match.
                field.name_id == input.name_id || {
                    let item_ptr = item as *const FieldSetItemRecord;
                    let input_ty = input.ty_record.non_null();
                    input_ids
                        .into_iter()
                        .all(|id| id == input_id || self.graph[id].ty_record.non_null() != input_ty)
                        && key.iter().all(|other_item| {
                            // Comparing pointers directly as they come from the same array.
                            let other_item_ptr = other_item as *const FieldSetItemRecord;
                            let other_field = &self.graph[self.selections[other_item.field_id].definition_id];
                            item_ptr == other_item_ptr || other_field.ty_record != field.ty_record
                        })
                }
            }) {
                missing &= !(1 << pos);

                input_values.push((
                    input_id,
                    ValueInjection::Select {
                        field_id: key[pos].field_id,
                        next: if key[pos].subselection_record.is_empty() {
                            None
                        } else {
                            let input_object_id = input.ty_record.definition_id.as_input_object()?;
                            let injection = self.try_build_object_injections(
                                false,
                                &key[pos].subselection_record,
                                self.graph[input_object_id].input_field_ids,
                            )?;
                            Some(self.selections.push_injection(injection))
                        },
                    },
                ));
            } else if let Some(default_value_id) = input.default_value_id {
                input_values.push((input_id, ValueInjection::Const(default_value_id)));
            } else if input.ty_record.wrapping.is_required() {
                tracing::trace!("A required input doesn't match any key.");
                return None;
            }
        }

        if missing != 0 {
            tracing::trace!("Could not match some key fields.");
            return None;
        }

        Some(input_values)
    }
}

/// Can inject a `ID` into a `ID!` but not the opposite.
fn can_inject_field_into_input(field: &FieldDefinitionRecord, input: &InputValueDefinitionRecord, batch: bool) -> bool {
    // if it's a union/interface/object, the input will have a different type, So we validate it
    // field by field later.
    match field.ty_record.definition_id {
        TypeDefinitionId::Enum(_) | TypeDefinitionId::Scalar(_) => {
            if field.ty_record.definition_id != input.ty_record.definition_id {
                return false;
            }
        }
        _ => {
            if !input.ty_record.definition_id.is_input_object() {
                return false;
            }
        }
    }
    let mut input = input.ty_record.wrapping;
    let field = field.ty_record.wrapping;
    if batch {
        let mut w = input.to_mutable();
        if w.pop_outermost_list_wrapping().is_none() {
            return false;
        }
        input = w.into();
    }
    input == field || input.non_null() == field
}
