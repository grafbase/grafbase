use std::{cmp::Ordering, collections::VecDeque};

use crate::response::{ResponseListId, ResponseObjectField, ResponseObjectId, ResponseValue};

use super::ResponseBuilder;

impl ResponseBuilder<'_> {
    pub(super) fn recursive_merge_with_default_object(
        &mut self,
        object_id: ResponseObjectId,
        default_fields_sorted_by_key: &[ResponseObjectField],
    ) {
        // When ingesting default fields, which we set to Null, we may encounter an actual
        // value in the case of shared roots. In this case we keep the old value.
        self.recursive_merge_object(object_id, default_fields_sorted_by_key.to_vec(), true);
    }

    pub(super) fn recursive_merge_shared_object(
        &mut self,
        object_id: ResponseObjectId,
        new_fields_sorted_by_key: Vec<ResponseObjectField>,
    ) {
        self.recursive_merge_object(object_id, new_fields_sorted_by_key, false);
    }

    fn recursive_merge_object(
        &mut self,
        object_id: ResponseObjectId,
        new_fields_sorted_by_key: Vec<ResponseObjectField>,
        skip_new_field_if_exists_already: bool,
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
                    if !skip_new_field_if_exists_already {
                        self.recursive_merge_value(existing_field.value.clone(), new_field.value);
                    }
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
                self.recursive_merge_object(existing_id, new_fields_sorted_by_key, false);
            }
            (ResponseValue::List { id: existing_id, .. }, ResponseValue::List { id: new_id, .. }) => {
                self.recursive_merge_list(existing_id, new_id)
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
                // FIXME: Unlikely, but we should generate an error here.
                tracing::error!(
                    "Trying to merge something values that aren't a couple of objects/list/couple of nulls {l:?} | {r:?}"
                );
            }
        }
    }

    fn recursive_merge_list(&mut self, existing_list_id: ResponseListId, new_list_id: ResponseListId) {
        let mut i = 0;
        while let Some((existing, new)) = self.data_parts[existing_list_id]
            .get(i)
            .zip(self.data_parts[new_list_id].get(i))
        {
            self.recursive_merge_value(existing.clone(), new.clone());
            i += 1;
        }
        // FIXME: Unlikely but should generate an error here.
    }
}
