use std::{cmp::Ordering, collections::VecDeque};

use crate::response::{ResponseListId, ResponseObjectField, ResponseObjectId, ResponseValue};

use super::ResponseBuilder;

impl ResponseBuilder {
    pub(super) fn recursive_merge_object(
        &mut self,
        object_id: ResponseObjectId,
        new_fields_sorted_by_edge: Vec<ResponseObjectField>,
    ) {
        let mut new_fields_sorted_by_edge = VecDeque::from(new_fields_sorted_by_edge);
        let Some(mut new_field) = new_fields_sorted_by_edge.pop_front() else {
            return;
        };

        let mut existing_fields = std::mem::take(&mut self.data_parts[object_id].fields_sorted_by_query_position);
        let n = existing_fields.len();
        let mut i = 0;
        loop {
            if i >= n {
                existing_fields.push(new_field);
                break;
            }
            // SAFETY we only push new elements and we compare against the initial size n. So i is
            // guaranteed to be within the array.
            let existing_field = unsafe { existing_fields.get_unchecked(i) };
            match existing_field.key.cmp(&new_field.key) {
                Ordering::Less => {
                    i += 1;
                }
                Ordering::Greater => {
                    // Adding at the end and will be sorted later.
                    existing_fields.push(new_field);
                    if let Some(next) = new_fields_sorted_by_edge.pop_front() {
                        new_field = next;
                    } else {
                        break;
                    }
                }
                Ordering::Equal => {
                    self.recursive_merge_value(existing_field.value.clone(), new_field.value);
                    i += 1;
                    if let Some(next) = new_fields_sorted_by_edge.pop_front() {
                        new_field = next;
                    } else {
                        break;
                    }
                }
            }
        }
        existing_fields.append(&mut Vec::from(new_fields_sorted_by_edge));
        existing_fields.sort_unstable_by(|a, b| a.key.cmp(&b.key));

        self.data_parts[object_id].fields_sorted_by_query_position = existing_fields;
    }

    fn recursive_merge_value(&mut self, existing: ResponseValue, new: ResponseValue) {
        match (existing, new) {
            (ResponseValue::Object { id: existing_id, .. }, ResponseValue::Object { id: new_id, .. }) => {
                let new_fields_sorted_by_edge =
                    std::mem::take(&mut self.data_parts[new_id].fields_sorted_by_query_position);
                self.recursive_merge_object(existing_id, new_fields_sorted_by_edge);
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
            _ => {
                // FIXME: Unlikely, but we should generate an error here.
                tracing::error!("Trying to merge something values that aren't a couple of objects/list");
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
