use crate::{
    FieldDefinitionId, FieldSetItemRecord, FieldSetRecord, InputValueDefinitionId, KeyValueInjectionRecord,
    SchemaFieldRecord, ValueInjection,
    builder::{
        BoundSelectedObjectValue, BoundSelectedValue, BoundSelectedValueEntry, GraphBuilder, SelectedValueOrField, sdl,
    },
};

pub(crate) struct InjectionsBuilder<'a, 'b> {
    pub builder: &'a mut GraphBuilder<'b>,
    pub sdl_definitions: &'a sdl::SdlDefinitions<'a>,
    pub subgraph_name: sdl::GraphName<'a>,
}

impl<'b> std::ops::Deref for InjectionsBuilder<'_, 'b> {
    type Target = GraphBuilder<'b>;
    fn deref(&self) -> &Self::Target {
        self.builder
    }
}

impl InjectionsBuilder<'_, '_> {
    #[allow(unused)]
    pub fn create_requirements_and_injection(
        &mut self,
        value: BoundSelectedValue<InputValueDefinitionId>,
    ) -> (FieldSetRecord, ValueInjection) {
        if value.alternatives.len() > 1 {
            let mut field_set = FieldSetRecord::default();
            let mut alternatives = Vec::new();
            for entry in value.alternatives {
                let (requirements, injection) = self.create_requirements_and_injection_for_entry(entry);
                field_set = field_set.union(&requirements);
                alternatives.push(injection);
            }
            let ids = self.builder.selections.push_injections(alternatives);
            (field_set, ValueInjection::OneOf(ids))
        } else {
            self.create_requirements_and_injection_for_entry(value.alternatives.into_iter().next().unwrap())
        }
    }

    fn create_requirements_and_injection_for_entry(
        &mut self,
        entry: BoundSelectedValueEntry<InputValueDefinitionId>,
    ) -> (FieldSetRecord, ValueInjection) {
        match entry {
            BoundSelectedValueEntry::Identity => (Default::default(), ValueInjection::Identity),
            BoundSelectedValueEntry::Path(path) => {
                let [head, rest @ ..] = &path[..] else {
                    unreachable!("Path must have at least one element");
                };
                self.create_requirement_and_injection_from_path(*head, rest)
            }
            BoundSelectedValueEntry::Object {
                path: Some(path),
                object,
            } => {
                let [head, rest @ ..] = &path[..] else {
                    unreachable!("Path must have at least one element");
                };
                self.create_requirement_and_injection_from_path_and_object(*head, rest, object)
            }
            BoundSelectedValueEntry::Object { path: None, object } => {
                self.create_requirement_and_injection_from_object(object)
            }
            BoundSelectedValueEntry::List { path: Some(path), list } => {
                let [head, rest @ ..] = &path[..] else {
                    unreachable!("Path must have at least one element");
                };
                self.create_requirement_and_injection_from_path_and_list(*head, rest, list.0)
            }
            BoundSelectedValueEntry::List { path: None, list } => self.create_requirements_and_injection(list.0),
        }
    }

    fn create_requirement_and_injection_from_path(
        &mut self,
        first: FieldDefinitionId,
        rest: &[FieldDefinitionId],
    ) -> (FieldSetRecord, ValueInjection) {
        let field_id = self.builder.selections.insert_field(SchemaFieldRecord {
            definition_id: first,
            sorted_argument_ids: Default::default(),
        });
        let (subselection_record, next_injection) = match rest {
            [] => (Default::default(), ValueInjection::Identity),
            [next, rest @ ..] => {
                let (subselection_record, next_injection) =
                    self.create_requirement_and_injection_from_path(*next, rest);
                (subselection_record, next_injection)
            }
        };
        let next = self.builder.selections.push_injection(next_injection);
        (
            FieldSetRecord::from_iter([FieldSetItemRecord {
                field_id,
                subselection_record,
            }]),
            ValueInjection::Select { field_id, next },
        )
    }

    fn create_requirement_and_injection_from_path_and_object(
        &mut self,
        first: FieldDefinitionId,
        rest: &[FieldDefinitionId],
        object: BoundSelectedObjectValue<InputValueDefinitionId>,
    ) -> (FieldSetRecord, ValueInjection) {
        let field_id = self.builder.selections.insert_field(SchemaFieldRecord {
            definition_id: first,
            sorted_argument_ids: Default::default(),
        });
        let (subselection_record, next_injection) = match rest {
            [] => self.create_requirement_and_injection_from_object(object),
            [next, rest @ ..] => self.create_requirement_and_injection_from_path_and_object(*next, rest, object),
        };
        let next = self.builder.selections.push_injection(next_injection);
        (
            FieldSetRecord::from_iter([FieldSetItemRecord {
                field_id,
                subselection_record,
            }]),
            ValueInjection::Select { field_id, next },
        )
    }

    fn create_requirement_and_injection_from_object(
        &mut self,
        object: BoundSelectedObjectValue<InputValueDefinitionId>,
    ) -> (FieldSetRecord, ValueInjection) {
        let mut field_set = FieldSetRecord::default();
        let mut key_value_injections = Vec::with_capacity(object.fields.len());
        for field in object.fields {
            let key_id = self.builder.graph[field.id].name_id;
            match field.value {
                SelectedValueOrField::Value(value) => {
                    let (requires, value) = self.create_requirements_and_injection(value);
                    key_value_injections.push(KeyValueInjectionRecord { key_id, value });
                    field_set = field_set.union(&requires);
                }
                SelectedValueOrField::Field(field_definition_id) => {
                    let field_id = self.builder.selections.insert_field(SchemaFieldRecord {
                        definition_id: field_definition_id,
                        sorted_argument_ids: Default::default(),
                    });
                    key_value_injections.push(KeyValueInjectionRecord {
                        key_id,
                        value: ValueInjection::Select {
                            field_id,
                            next: self.builder.selections.push_injection(ValueInjection::Identity),
                        },
                    });
                    if !field_set.iter().any(|item| item.field_id == field_id) {
                        field_set.insert(FieldSetItemRecord {
                            field_id,
                            subselection_record: Default::default(),
                        });
                    }
                }
            }
        }

        (
            field_set,
            ValueInjection::Object(self.builder.selections.push_key_value_injections(key_value_injections)),
        )
    }

    fn create_requirement_and_injection_from_path_and_list(
        &mut self,
        first: FieldDefinitionId,
        rest: &[FieldDefinitionId],
        list: BoundSelectedValue<InputValueDefinitionId>,
    ) -> (FieldSetRecord, ValueInjection) {
        let field_id = self.builder.selections.insert_field(SchemaFieldRecord {
            definition_id: first,
            sorted_argument_ids: Default::default(),
        });
        let (subselection_record, next_injection) = match rest {
            [] => self.create_requirements_and_injection(list),
            [next, rest @ ..] => self.create_requirement_and_injection_from_path_and_list(*next, rest, list),
        };
        let next = self.builder.selections.push_injection(next_injection);
        (
            FieldSetRecord::from_iter([FieldSetItemRecord {
                field_id,
                subselection_record,
            }]),
            ValueInjection::Select { field_id, next },
        )
    }
}
