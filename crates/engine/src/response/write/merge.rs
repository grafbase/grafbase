use std::{cmp::Ordering, collections::VecDeque};

use crate::{
    prepare::DefaultFieldShape,
    response::{ResponseField, ResponseListId, ResponseObjectId, ResponseValue},
};

use super::ResponseBuilder;

impl<'ctx> ResponseBuilder<'ctx> {
    pub(super) fn merge_with_default_object(
        &mut self,
        object_id: ResponseObjectId,
        mut default_fields_sorted_by_key: impl Iterator<Item = DefaultFieldShape<'ctx>>,
    ) {
        let Some(mut default_field) = default_fields_sorted_by_key.next() else {
            return;
        };
        let mut key = default_field.key();

        let mut existing_fields = std::mem::take(&mut self.data_parts[object_id].fields_sorted_by_key);
        let n = existing_fields.len();
        let mut i = 0;
        loop {
            if i >= n {
                existing_fields.push(ResponseField {
                    key,
                    value: default_field.value.into(),
                });
                break;
            }
            let existing_field = &existing_fields[i];
            match existing_field.key.cmp(&key) {
                Ordering::Less => {
                    i += 1;
                }
                Ordering::Greater => {
                    // Adding at the end and will be sorted later.
                    existing_fields.push(ResponseField {
                        key,
                        value: default_field.value.into(),
                    });
                    if let Some(next) = default_fields_sorted_by_key.next() {
                        default_field = next;
                        key = default_field.key();
                    } else {
                        break;
                    }
                }
                // When ingesting default fields, which we set to Null, we may encounter an actual
                // value in the case of shared roots. In this case we keep the old value.
                Ordering::Equal => {
                    i += 1;
                    if let Some(next) = default_fields_sorted_by_key.next() {
                        default_field = next;
                        key = default_field.key();
                    } else {
                        break;
                    }
                }
            }
        }
        for field in default_fields_sorted_by_key {
            existing_fields.push(ResponseField {
                key,
                value: field.value.into(),
            });
        }
        existing_fields.sort_unstable_by(|a, b| a.key.cmp(&b.key));

        self.data_parts[object_id].fields_sorted_by_key = existing_fields;
    }

    pub(super) fn recursive_merge_object(
        &mut self,
        object_id: ResponseObjectId,
        new_fields_sorted_by_key: Vec<ResponseField>,
    ) {
        let mut new_fields_sorted_by_key = VecDeque::from(new_fields_sorted_by_key);
        let Some(mut new_field) = new_fields_sorted_by_key.pop_front() else {
            return;
        };

        let mut existing_fields = std::mem::take(&mut self.data_parts[object_id].fields_sorted_by_key);
        let n = existing_fields.len();
        let mut i = 0;
        loop {
            if i >= n {
                existing_fields.push(new_field);
                break;
            }
            let existing_field = &existing_fields[i];
            match existing_field.key.cmp(&new_field.key) {
                Ordering::Less => {
                    i += 1;
                }
                Ordering::Greater => {
                    // Adding at the end and will be sorted later.
                    existing_fields.push(new_field);
                    if let Some(next) = new_fields_sorted_by_key.pop_front() {
                        new_field = next;
                    } else {
                        break;
                    }
                }
                Ordering::Equal => {
                    self.recursive_merge_value(existing_field.value.clone(), new_field.value);
                    i += 1;
                    if let Some(next) = new_fields_sorted_by_key.pop_front() {
                        new_field = next;
                    } else {
                        break;
                    }
                }
            }
        }
        existing_fields.append(&mut Vec::from(new_fields_sorted_by_key));
        existing_fields.sort_unstable_by(|a, b| a.key.cmp(&b.key));

        self.data_parts[object_id].fields_sorted_by_key = existing_fields;
    }

    fn recursive_merge_value(&mut self, existing: ResponseValue, new: ResponseValue) {
        match (existing, new) {
            (ResponseValue::Object { id: existing_id, .. }, ResponseValue::Object { id: new_id, .. }) => {
                let new_fields_sorted_by_key = std::mem::take(&mut self.data_parts[new_id].fields_sorted_by_key);
                self.recursive_merge_object(existing_id, new_fields_sorted_by_key);
            }
            (
                ResponseValue::List {
                    id: existing_id,
                    offset: existing_offset,
                    length: existing_length,
                },
                ResponseValue::List {
                    id: new_id,
                    offset: new_offset,
                    length: new_length,
                },
            ) => {
                assert!(existing_length == new_length);
                self.recursive_merge_list(existing_id, existing_offset, new_id, new_offset, existing_length)
            }
            (ResponseValue::Inaccessible { id: existing_id }, ResponseValue::Inaccessible { id: new_id }) => {
                self.recursive_merge_value(self.data_parts[existing_id].clone(), self.data_parts[new_id].clone());
            }
            (ResponseValue::Inaccessible { id }, new) => {
                self.recursive_merge_value(self.data_parts[id].clone(), new);
            }
            (existing, ResponseValue::Inaccessible { id }) => {
                self.recursive_merge_value(existing, self.data_parts[id].clone());
            }
            (ResponseValue::Null, ResponseValue::Null) => {}
            (l, r) => {
                unreachable!(
                    "Trying to merge something values that aren't a couple of objects/list/couple of nulls {l:?} | {r:?}"
                );
            }
        }
    }

    fn recursive_merge_list(
        &mut self,
        existing_list_id: ResponseListId,
        existing_offset: u32,
        new_list_id: ResponseListId,
        new_offset: u32,
        length: u32,
    ) {
        let mut i = 0;
        let length = length as usize;
        while i < length
            && let Some((existing, new)) = self.data_parts[existing_list_id]
                .get(existing_offset as usize + i)
                .zip(self.data_parts[new_list_id].get(new_offset as usize + i))
        {
            self.recursive_merge_value(existing.clone(), new.clone());
            i += 1;
        }
        // FIXME: Unlikely but should generate an error here.
    }
}
