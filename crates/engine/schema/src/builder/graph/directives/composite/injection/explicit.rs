use crate::{
    FieldDefinitionId, FieldSetItemRecord, FieldSetRecord, InputValueDefinitionId, KeyValueInjectionRecord,
    SchemaFieldRecord, ValueInjection,
    builder::{
        BoundPath, BoundSelectedObjectValue, BoundSelectedValue, BoundSelectedValueEntry, GraphBuilder,
        SelectedValueOrField,
    },
};

pub(crate) fn create_requirements_and_injection(
    builder: &mut GraphBuilder<'_>,
    value: BoundSelectedValue<InputValueDefinitionId>,
) -> Result<(FieldSetRecord, ValueInjection), String> {
    if value.alternatives.len() > 1 {
        let mut field_set = FieldSetRecord::default();
        let mut alternatives = Vec::new();
        for entry in value.alternatives {
            let (requirements, injection) = create_requirements_and_injection_for_entry(builder, entry)?;
            field_set = field_set.union(&requirements);
            alternatives.push(injection);
        }
        let ids = builder.selections.push_injections(alternatives);
        Ok((field_set, ValueInjection::OneOf(ids)))
    } else {
        create_requirements_and_injection_for_entry(builder, value.alternatives.into_iter().next().unwrap())
    }
}

fn create_requirements_and_injection_for_entry(
    builder: &mut GraphBuilder<'_>,
    entry: BoundSelectedValueEntry<InputValueDefinitionId>,
) -> Result<(FieldSetRecord, ValueInjection), String> {
    Ok(match entry {
        BoundSelectedValueEntry::Identity => (Default::default(), ValueInjection::Identity),
        BoundSelectedValueEntry::Path(path) => {
            let [head, rest @ ..] = &path[..] else {
                unreachable!("Path must have at least one element");
            };
            create_requirements_and_injection_from_path(builder, *head, rest)
        }
        BoundSelectedValueEntry::Object { path, object } => {
            let result = create_requirements_and_injection_from_object(builder, object)?;
            if let Some(path) = path {
                prepend_requirements_and_injection_with_path(builder, path, result)
            } else {
                result
            }
        }
        BoundSelectedValueEntry::List { path, list } => {
            let result = create_requirements_and_injection(builder, list.0)?;
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

fn create_requirements_and_injection_from_path(
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
                create_requirements_and_injection_from_path(builder, *next, rest);
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

fn create_requirements_and_injection_from_object(
    builder: &mut GraphBuilder<'_>,
    object: BoundSelectedObjectValue<InputValueDefinitionId>,
) -> Result<(FieldSetRecord, ValueInjection), String> {
    let mut field_set = FieldSetRecord::default();
    let mut key_value_injections = Vec::with_capacity(object.fields.len());
    let mut present_ids = Vec::with_capacity(object.fields.len());
    for field in object.fields {
        present_ids.push(field.id);
        let key_id = builder.graph[field.id].name_id;
        match field.value {
            SelectedValueOrField::Value(value) => {
                let (requires, value) = create_requirements_and_injection(builder, value)?;
                key_value_injections.push(KeyValueInjectionRecord { key_id, value });
                field_set = field_set.union(&requires);
            }
            SelectedValueOrField::Field(field_definition_id) => {
                let field_id = builder.selections.insert_field(SchemaFieldRecord {
                    definition_id: field_definition_id,
                    sorted_argument_ids: Default::default(),
                });
                key_value_injections.push(KeyValueInjectionRecord {
                    key_id,
                    value: ValueInjection::Select {
                        field_id,
                        next: builder.selections.push_injection(ValueInjection::Identity),
                    },
                });
                if !field_set.iter().any(|item| item.field_id == field_id) {
                    field_set.insert(FieldSetItemRecord {
                        field_id,
                        subselection_record: Default::default(),
                    });
                }
            }
            SelectedValueOrField::DefaultValue(id) => {
                key_value_injections.push(KeyValueInjectionRecord {
                    key_id,
                    value: ValueInjection::DefaultValue(id),
                });
            }
        }
    }

    Ok((
        field_set,
        ValueInjection::Object(builder.selections.push_key_value_injections(key_value_injections)),
    ))
}
