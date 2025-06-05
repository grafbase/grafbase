use id_newtypes::IdRange;

use crate::{
    ArgumentInjectionId, ArgumentInjectionRecord, ArgumentValueInjection, FieldDefinitionRecord, FieldSetItemRecord,
    FieldSetRecord, InputValueDefinitionId, InputValueDefinitionRecord, KeyValueInjectionRecord, TypeDefinitionId,
    ValueInjection,
    builder::{Error, GraphBuilder},
};

pub(crate) fn try_auto_detect_unique_injection(
    builder: &mut GraphBuilder,
    batch: bool,
    key_fields: &FieldSetRecord,
    argument_ids: IdRange<InputValueDefinitionId>,
) -> Result<Option<IdRange<ArgumentInjectionId>>, Error> {
    let state = builder.selections.current_state();
    let mut candidates = try_auto_detect_all_possible_injections(builder, batch, key_fields, argument_ids)?;
    let Some(candidate) = candidates.pop() else {
        tracing::debug!("No candidiate found");
        return Ok(None);
    };
    if !candidates.is_empty() {
        tracing::debug!("Multiple candidiates found, skipping key");
        builder.selections.reset(state);
        return Ok(None);
    }

    Ok(Some(candidate))
}

fn try_auto_detect_all_possible_injections(
    builder: &mut GraphBuilder,
    batch: bool,
    key_fields: &FieldSetRecord,
    argument_ids: IdRange<InputValueDefinitionId>,
) -> Result<Vec<IdRange<ArgumentInjectionId>>, Error> {
    let mut candidates = Vec::new();

    // Try direct match against arguments
    // We're not supporting cases like `lookup(key1: [ID!], key2: [ID!])` for composite keys.
    // We could, but means creating two lists which can't optimize for.
    if key_fields.len() == 1 || !batch {
        tracing::trace!("Trying to match against arguments directly.");
        let state = builder.selections.current_state();
        if let Some(candidate) = try_auto_detect_arguments_injections(builder, batch, key_fields, argument_ids)? {
            candidates.push(candidate);
        } else {
            builder.selections.reset(state);
        }
    }

    let is_required_arg_bitset: u64 = {
        let mut bitset = 0;
        for (i, arg) in builder.graph[argument_ids].iter().enumerate() {
            if arg.ty_record.is_required() {
                // More than one required input means we'll never find a single nested input object
                // we can use.
                if bitset != 0 {
                    return Ok(candidates);
                }
                bitset |= 1 << i;
            }
        }
        bitset
    };

    tracing::trace!("Trying to match against a nested input object.");
    // Try with a nested input object
    for (i, argument_id) in argument_ids.into_iter().enumerate() {
        let arg = &builder.graph[argument_id];
        let span = tracing::debug_span!("match_argument", key = %builder.ctx[arg.name_id]);
        let _enter = span.enter();
        let Some(input_object_id) = arg.ty_record.definition_id.as_input_object() else {
            continue;
        };
        if (is_required_arg_bitset & !(1 << i)) != 0 {
            tracing::trace!("There exists another required argument.");
            // There exist a one required input, so can't use this argument for the key.
            continue;
        }

        let state = builder.selections.current_state();
        let arg = &builder.graph[argument_id];
        let input_object = &builder.graph[input_object_id];

        if input_object.is_one_of {
            if arg.ty_record.wrapping.is_list() {
                continue;
            }
            if key_fields.len() == 1 {
                tracing::trace!("Trying to match with oneof input object for a key having a single field.");
                if let Some((input_field_id, value)) = try_auto_detect_oneof_input_object_with_single_key(
                    builder,
                    batch,
                    &key_fields[0],
                    input_object.input_field_ids,
                )? {
                    let value = builder
                        .selections
                        .push_argument_value_injection(ArgumentValueInjection::Value(value));
                    let range = builder.selections.push_argument_injections([ArgumentInjectionRecord {
                        definition_id: argument_id,
                        value: ArgumentValueInjection::Nested {
                            key: builder.graph[input_field_id].name_id,
                            value,
                        },
                    }]);
                    candidates.push(range)
                } else {
                    builder.selections.reset(state);
                }
            } else {
                tracing::trace!("Trying to match with oneof input object for a key having multiple fields.");
                for oneof_field_id in input_object.input_field_ids {
                    let oneof_field = &builder.graph[oneof_field_id];
                    let Some(nested_input_object_id) = oneof_field.ty_record.definition_id.as_input_object() else {
                        continue;
                    };
                    if !matches!(
                        (batch, oneof_field.ty_record.wrapping.list_wrappings().len()),
                        (true, 1) | (false, 0)
                    ) {
                        continue;
                    }
                    let name_id = oneof_field.name_id;
                    if let Some(value) = try_auto_detect_input_object_injections(
                        builder,
                        false,
                        key_fields,
                        builder.graph[nested_input_object_id].input_field_ids,
                    )? {
                        let value = builder
                            .selections
                            .push_argument_value_injection(ArgumentValueInjection::Value(value));
                        let range = builder.selections.push_argument_injections([ArgumentInjectionRecord {
                            definition_id: argument_id,
                            value: ArgumentValueInjection::Nested { key: name_id, value },
                        }]);
                        candidates.push(range)
                    } else {
                        builder.selections.reset(state);
                    }
                }
            }
        } else if key_fields.len() > 1 {
            tracing::trace!("Trying to match with nested object for a key having multiple fields.");
            if !matches!(
                (batch, arg.ty_record.wrapping.list_wrappings().len()),
                (true, 1) | (false, 0)
            ) {
                continue;
            }

            if let Some(value) =
                try_auto_detect_input_object_injections(builder, false, key_fields, input_object.input_field_ids)?
            {
                let range = builder.selections.push_argument_injections([ArgumentInjectionRecord {
                    definition_id: argument_id,
                    value: ArgumentValueInjection::Value(value),
                }]);
                candidates.push(range)
            } else {
                builder.selections.reset(state);
            }
        }
    }

    Ok(candidates)
}

fn try_auto_detect_arguments_injections(
    builder: &mut GraphBuilder,
    batch: bool,
    key_fields: &[FieldSetItemRecord],
    argument_ids: IdRange<InputValueDefinitionId>,
) -> Result<Option<IdRange<ArgumentInjectionId>>, Error> {
    assert!(key_fields.len() < 64, "Cannot handle keys with 64 fields or more.");
    let mut missing: u64 = (1 << key_fields.len()) - 1;
    let mut argument_injections = Vec::new();

    for argument_id in argument_ids {
        if let Some((pos, value)) =
            try_auto_detect_unique_input_value_key_mapping(builder, argument_ids, key_fields, batch, argument_id)?
        {
            if let Some(pos) = pos {
                missing &= !(1 << pos);
            }
            argument_injections.push(ArgumentInjectionRecord {
                definition_id: argument_id,
                value: ArgumentValueInjection::Value(value),
            });
        } else if let Some(default_value) = builder.graph[argument_id].default_value_id {
            argument_injections.push(ArgumentInjectionRecord {
                definition_id: argument_id,
                value: ArgumentValueInjection::Value(ValueInjection::DefaultValue(default_value)),
            })
        } else if builder.graph[argument_id].ty_record.wrapping.is_non_null() {
            tracing::trace!("A required argument doesn't match any key.");
            return Ok(None);
        }
    }
    if missing != 0 {
        tracing::trace!("Could not match some key fields.");
        return Ok(None);
    }

    Ok(Some(builder.selections.push_argument_injections(argument_injections)))
}

fn try_auto_detect_input_object_injections(
    builder: &mut GraphBuilder,
    batch: bool,
    key_fields: &[FieldSetItemRecord],
    input_field_ids: IdRange<InputValueDefinitionId>,
) -> Result<Option<ValueInjection>, Error> {
    assert!(key_fields.len() < 64, "Cannot handle keys with 64 fields or more.");
    let mut missing: u64 = (1 << key_fields.len()) - 1;
    let mut key_value_injections = Vec::new();

    for input_id in input_field_ids {
        if let Some((pos, value)) =
            try_auto_detect_unique_input_value_key_mapping(builder, input_field_ids, key_fields, batch, input_id)?
        {
            if let Some(pos) = pos {
                missing &= !(1 << pos);
            }
            key_value_injections.push(KeyValueInjectionRecord {
                key_id: builder.graph[input_id].name_id,
                value,
            });
        } else if let Some(default_value) = builder.graph[input_id].default_value_id {
            key_value_injections.push(KeyValueInjectionRecord {
                key_id: builder.graph[input_id].name_id,
                value: ValueInjection::DefaultValue(default_value),
            })
        } else if builder.graph[input_id].ty_record.wrapping.is_non_null() {
            tracing::trace!("A required argument doesn't match any key.");
            return Ok(None);
        }
    }

    if missing != 0 {
        tracing::trace!("Could not match some key fields.");
        return Ok(None);
    }

    let range = builder.selections.push_key_value_injections(key_value_injections);
    Ok(Some(ValueInjection::Object(range)))
}

fn try_auto_detect_oneof_input_object_with_single_key(
    builder: &mut GraphBuilder,
    batch: bool,
    key: &FieldSetItemRecord,
    input_ids: IdRange<InputValueDefinitionId>,
) -> Result<Option<(InputValueDefinitionId, ValueInjection)>, Error> {
    let mut input_values = Vec::new();
    for input_id in input_ids {
        if let Some((_, value)) = try_auto_detect_unique_input_value_key_mapping(
            builder,
            input_ids,
            std::array::from_ref(key),
            batch,
            input_id,
        )? {
            input_values.push((input_id, value));
        } else if builder.graph[input_id].ty_record.wrapping.is_non_null() {
            tracing::trace!("A required input doesn't match any key.");
            return Ok(None);
        }
    }

    if let Some(out) = input_values.pop() {
        if input_values.len() > 1 {
            tracing::trace!("Multiple inputs matched, cannot auto-detect oneof input object.");
            return Ok(None);
        }

        Ok(Some(out))
    } else {
        tracing::trace!("No input matched.");
        Ok(None)
    }
}

fn try_auto_detect_unique_input_value_key_mapping(
    builder: &mut GraphBuilder,
    input_ids: IdRange<InputValueDefinitionId>,
    key_fields: &[FieldSetItemRecord],
    batch: bool,
    input_id: InputValueDefinitionId,
) -> Result<Option<(Option<usize>, ValueInjection)>, Error> {
    let input = &builder.graph[input_id];

    // position() is enough as at most there will be one unique match.
    if let Some(pos) = key_fields.iter().position(|key_field| {
        let field = &builder.graph[builder.selections[key_field.field_id].definition_id];
        if !can_inject_field_into_input(field, input, batch) {
            tracing::trace!(
                "Field {} cannot be injected into input {}",
                builder.ctx[field.name_id],
                builder.ctx[input.name_id]
            );
            return false;
        }
        // Either name matches or the types are unique in both input & key and thus no other key/input pair could
        // match.
        field.name_id == input.name_id || {
            let item_ptr = key_field as *const FieldSetItemRecord;
            let input_ty = input.ty_record.non_null();
            input_ids
                .into_iter()
                .all(|id| id == input_id || builder.graph[id].ty_record.non_null() != input_ty)
                && key_fields.iter().all(|other_item| {
                    // Comparing pointers directly as they come from the same array.
                    let other_item_ptr = other_item as *const FieldSetItemRecord;
                    let other_field = &builder.graph[builder.selections[other_item.field_id].definition_id];
                    item_ptr == other_item_ptr || other_field.ty_record != field.ty_record
                })
        }
    }) {
        let next_injection = if key_fields[pos].subselection_record.is_empty() {
            ValueInjection::Identity
        } else {
            let Some(input_object_id) = input.ty_record.definition_id.as_input_object() else {
                return Ok(None);
            };
            let Some(injection) = try_auto_detect_input_object_injections(
                builder,
                false,
                &key_fields[pos].subselection_record,
                builder.graph[input_object_id].input_field_ids,
            )?
            else {
                return Ok(None);
            };
            injection
        };
        Ok(Some((
            Some(pos),
            ValueInjection::Select {
                field_id: key_fields[pos].field_id,
                next: builder.selections.push_injection(next_injection),
            },
        )))
    } else if let Some(default_value_id) = input.default_value_id {
        Ok(Some((None, ValueInjection::DefaultValue(default_value_id))))
    } else {
        Ok(None)
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
