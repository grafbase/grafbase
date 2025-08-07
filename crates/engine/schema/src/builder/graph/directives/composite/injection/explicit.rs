use crate::{
    ArgumentInjectionRecord, ArgumentValueInjection, FieldDefinitionId, FieldSetItemRecord, FieldSetRecord,
    InputValueDefinitionId, KeyValueInjectionRecord, SchemaFieldRecord, ValueInjection,
    builder::{
        BoundFieldValue, BoundInputField, BoundInputFieldValue, BoundInputObject, BoundPath, BoundSelectedObjectValue,
        BoundSelectedValue, BoundSelectedValueEntry, BoundValue, GraphBuilder,
    },
};

pub(crate) fn create_requirements_and_injections(
    builder: &mut GraphBuilder<'_>,
    injections: impl IntoIterator<Item = (InputValueDefinitionId, BoundValue)>,
) -> Result<(FieldSetRecord, Vec<ArgumentInjectionRecord>), String> {
    let mut requirements = FieldSetRecord::default();
    let mut arguments = Vec::new();
    for (input_value_id, value) in injections {
        let (r, arg) = create_requirements_and_injection(builder, input_value_id, value)?;
        requirements = requirements.union(&r);
        arguments.push(arg);
    }
    Ok((requirements, arguments))
}

pub(crate) fn create_requirements_and_injection(
    builder: &mut GraphBuilder<'_>,
    input_value_id: InputValueDefinitionId,
    value: BoundValue,
) -> Result<(FieldSetRecord, ArgumentInjectionRecord), String> {
    match value {
        BoundValue::InputObject(input_object) => {
            create_requirements_and_injection_for_input_object(builder, input_value_id, input_object)
        }
        BoundValue::Value(value) => {
            let (requirements, value) = create_requirements_and_injection_for_selected_value(builder, value)?;
            let arg = ArgumentInjectionRecord {
                definition_id: input_value_id,
                value: ArgumentValueInjection::Value(value),
            };
            Ok((requirements, arg))
        }
    }
}

pub(crate) fn create_requirements_and_injection_for_input_object(
    builder: &mut GraphBuilder<'_>,
    input_value_id: InputValueDefinitionId,
    input_object: BoundInputObject,
) -> Result<(FieldSetRecord, ArgumentInjectionRecord), String> {
    let mut requirements = FieldSetRecord::default();
    let mut injections = Vec::new();
    for BoundInputField { id, value } in input_object.input_fields {
        match value {
            BoundInputFieldValue::InputObject(input_object) => {
                let (r, injection) = create_requirements_and_injection_for_input_object(builder, id, input_object)?;
                requirements = requirements.union(&r);
                injections.push(injection);
            }
            BoundInputFieldValue::Value(value) => {
                let (r, value) = create_requirements_and_injection_for_field_value(builder, value)?;
                requirements = requirements.union(&r);
                let arg = ArgumentInjectionRecord {
                    definition_id: id,
                    value: ArgumentValueInjection::Value(value),
                };
                injections.push(arg);
            }
        }
    }
    let ids = builder.selections.push_argument_injections(injections);
    Ok((
        requirements,
        ArgumentInjectionRecord {
            definition_id: input_value_id,
            value: ArgumentValueInjection::InputObject(ids),
        },
    ))
}

pub(crate) fn create_requirements_and_injection_for_selected_value(
    builder: &mut GraphBuilder<'_>,
    value: BoundSelectedValue<InputValueDefinitionId>,
) -> Result<(FieldSetRecord, ValueInjection), String> {
    if value.alternatives.len() > 1 {
        let mut field_set = FieldSetRecord::default();
        let mut alternatives = Vec::new();
        for entry in value.alternatives {
            let (requirements, injection) = create_requirements_and_injection_for_selected_entry(builder, entry)?;
            field_set = field_set.union(&requirements);
            alternatives.push(injection);
        }
        let _ids = builder.selections.push_injections(alternatives);
        Err("Alternatives with '|' are not supported yet.".to_string())
    } else {
        create_requirements_and_injection_for_selected_entry(builder, value.alternatives.into_iter().next().unwrap())
    }
}

fn create_requirements_and_injection_for_selected_entry(
    builder: &mut GraphBuilder<'_>,
    entry: BoundSelectedValueEntry<InputValueDefinitionId>,
) -> Result<(FieldSetRecord, ValueInjection), String> {
    Ok(match entry {
        BoundSelectedValueEntry::Identity => (Default::default(), ValueInjection::Identity),
        BoundSelectedValueEntry::Path(path) => {
            let [head, rest @ ..] = &path[..] else {
                unreachable!("Path must have at least one element");
            };
            create_requirements_and_injection_for_path(builder, *head, rest)
        }
        BoundSelectedValueEntry::Object { path, object } => {
            let result = create_requirements_and_injection_for_selected_object(builder, object)?;
            if let Some(path) = path {
                prepend_requirements_and_injection_with_path(builder, path, result)
            } else {
                result
            }
        }
        BoundSelectedValueEntry::List { path, list } => {
            let result = create_requirements_and_injection_for_selected_value(builder, list.0)?;
            if let Some(path) = path {
                prepend_requirements_and_injection_with_path(builder, path, result)
            } else {
                result
            }
        }
    })
}

pub(crate) fn prepend_requirements_and_injection_with_path(
    builder: &mut GraphBuilder<'_>,
    mut path: BoundPath,
    (mut field_set, mut injection): (FieldSetRecord, ValueInjection),
) -> (FieldSetRecord, ValueInjection) {
    while let Some(field) = path.0.pop() {
        let field_id = builder.selections.insert_field(SchemaFieldRecord {
            definition_id: field,
            sorted_argument_ids: Default::default(),
        });
        field_set = FieldSetRecord::from_iter([FieldSetItemRecord {
            field_id,
            subselection_record: field_set,
        }]);
        let next_injection = builder.selections.push_injection(injection);
        injection = ValueInjection::Select {
            field_id,
            next: next_injection,
        };
    }
    (field_set, injection)
}

fn create_requirements_and_injection_for_path(
    builder: &mut GraphBuilder<'_>,
    first: FieldDefinitionId,
    rest: &[FieldDefinitionId],
) -> (FieldSetRecord, ValueInjection) {
    let field_id = builder.selections.insert_field(SchemaFieldRecord {
        definition_id: first,
        sorted_argument_ids: Default::default(),
    });
    let (subselection_record, next_injection) = match rest {
        [] => (Default::default(), ValueInjection::Identity),
        [next, rest @ ..] => {
            let (subselection_record, next_injection) =
                create_requirements_and_injection_for_path(builder, *next, rest);
            (subselection_record, next_injection)
        }
    };
    let next = builder.selections.push_injection(next_injection);
    (
        FieldSetRecord::from_iter([FieldSetItemRecord {
            field_id,
            subselection_record,
        }]),
        ValueInjection::Select { field_id, next },
    )
}

fn create_requirements_and_injection_for_selected_object(
    builder: &mut GraphBuilder<'_>,
    object: BoundSelectedObjectValue<InputValueDefinitionId>,
) -> Result<(FieldSetRecord, ValueInjection), String> {
    let mut field_set = FieldSetRecord::default();
    let mut key_value_injections = Vec::with_capacity(object.fields.len());
    let mut present_ids = Vec::with_capacity(object.fields.len());
    for field in object.fields {
        present_ids.push(field.id);
        let key_id = builder.graph[field.id].name_id;
        let (requires, value) = create_requirements_and_injection_for_field_value(builder, field.value)?;
        field_set = field_set.union(&requires);
        key_value_injections.push(KeyValueInjectionRecord { key_id, value });
    }

    Ok((
        field_set,
        ValueInjection::Object(builder.selections.push_key_value_injections(key_value_injections)),
    ))
}

fn create_requirements_and_injection_for_field_value(
    builder: &mut GraphBuilder<'_>,
    value: BoundFieldValue<InputValueDefinitionId>,
) -> Result<(FieldSetRecord, ValueInjection), String> {
    match value {
        BoundFieldValue::Value(value) => create_requirements_and_injection_for_selected_value(builder, value),
        BoundFieldValue::Field(field_definition_id) => {
            let field_id = builder.selections.insert_field(SchemaFieldRecord {
                definition_id: field_definition_id,
                sorted_argument_ids: Default::default(),
            });
            let injection = ValueInjection::Select {
                field_id,
                next: builder.selections.push_injection(ValueInjection::Identity),
            };
            let requires = vec![FieldSetItemRecord {
                field_id,
                subselection_record: Default::default(),
            }];
            Ok((requires.into(), injection))
        }
        BoundFieldValue::DefaultValue(id) => Ok((Default::default(), ValueInjection::DefaultValue(id))),
    }
}
