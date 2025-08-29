use std::{cmp::Ordering, collections::VecDeque, mem::take};

use crate::{
    prepare::DefaultFieldShape,
    response::{
        DataPartId, DataParts, ResponseField, ResponseFieldsSortedByKey, ResponseListId, ResponseObjectId,
        ResponseValue,
    },
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

        let ResponseFieldsSortedByKey::Owned { fields_id } = self.data_parts[object_id].fields_sorted_by_key else {
            unreachable!(
                "We're merging default fields into an existing object, which means it was tracked and thus must be owned."
            );
        };

        let mut existing_fields = take(&mut self.data_parts[object_id.part_id][fields_id]);
        let mut needs_sorting = false;
        let n = existing_fields.len();

        {
            let mut i = 0;
            let mut key = default_field.key();
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
                        needs_sorting = true;
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
        }

        if needs_sorting {
            existing_fields.sort_unstable_by_key(|field| field.key);
        }

        for field in default_fields_sorted_by_key {
            existing_fields.push(ResponseField {
                key: field.key(),
                value: field.value.into(),
            });
        }

        self.data_parts[object_id.part_id][fields_id] = existing_fields;
    }

    pub(super) fn recursive_merge_object_in_place(
        &mut self,
        existing_object_id: ResponseObjectId,
        new_part_id: DataPartId,
        new_fields: ResponseFieldsSortedByKey,
    ) {
        let existing_fields_id = match self.data_parts[existing_object_id].fields_sorted_by_key {
            ResponseFieldsSortedByKey::Slice {
                fields_id,
                offset,
                limit,
            } => {
                // It's a slice, we need to copy the fields into an owned vec before we can modify
                // them. This should be a rare operation only with nested shared root fields.
                let mut owned_fields = Vec::with_capacity(limit as usize);
                let fields_slice = &mut self.data_parts[existing_object_id.part_id][fields_id];
                for field in &mut fields_slice[offset as usize..(offset as usize + limit as usize)] {
                    owned_fields.push(std::mem::replace(field, ResponseField::null()));
                }
                let fields_id =
                    self.data_parts[existing_object_id.part_id].push_owned_sorted_fields_by_key(owned_fields);
                self.data_parts[existing_object_id].fields_sorted_by_key = fields_id.into();
                fields_id
            }
            ResponseFieldsSortedByKey::Owned { fields_id } => fields_id,
        };

        let mut existing_fields = take(&mut self.data_parts[existing_object_id.part_id][existing_fields_id]);
        let mut needs_sorting = false;
        let n = existing_fields.len();

        match new_fields {
            ResponseFieldsSortedByKey::Slice {
                fields_id: new_fields_id,
                offset,
                limit,
            } => {
                let offset = offset as usize;
                let mut next_new_index = 0;
                let mut next_new_field = |data_parts: &mut DataParts| -> Option<ResponseField> {
                    if next_new_index < limit {
                        let field = std::mem::replace(
                            &mut data_parts[new_part_id][new_fields_id][offset + next_new_index as usize],
                            ResponseField::null(),
                        );
                        next_new_index += 1;
                        Some(field)
                    } else {
                        None
                    }
                };
                if let Some(mut new_field) = next_new_field(&mut self.data_parts) {
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
                                needs_sorting = true;
                                existing_fields.push(new_field);
                                if let Some(next) = next_new_field(&mut self.data_parts) {
                                    new_field = next;
                                } else {
                                    break;
                                }
                            }
                            Ordering::Equal => {
                                self.recursive_merge_value_in_place(&existing_field.value, new_field.value);
                                i += 1;
                                if let Some(next) = next_new_field(&mut self.data_parts) {
                                    new_field = next;
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                    if needs_sorting {
                        existing_fields.sort_unstable_by_key(|field| field.key);
                    }
                    if next_new_index < limit {
                        let new_fields = &mut self.data_parts[new_part_id][new_fields_id];
                        let start = offset + next_new_index as usize;
                        let end = offset + limit as usize;
                        for new_field in &mut new_fields[start..end] {
                            existing_fields.push(std::mem::replace(new_field, ResponseField::null()));
                        }
                    }
                }
            }
            ResponseFieldsSortedByKey::Owned {
                fields_id: new_fields_id,
            } => {
                let mut new_fields = VecDeque::from(take(&mut self.data_parts[new_part_id][new_fields_id]));
                if let Some(mut new_field) = new_fields.pop_front() {
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
                                needs_sorting = true;
                                existing_fields.push(new_field);
                                if let Some(next) = new_fields.pop_front() {
                                    new_field = next;
                                } else {
                                    break;
                                }
                            }
                            Ordering::Equal => {
                                self.recursive_merge_value_in_place(&existing_field.value, new_field.value);
                                i += 1;
                                if let Some(next) = new_fields.pop_front() {
                                    new_field = next;
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                    if needs_sorting {
                        existing_fields.sort_unstable_by_key(|field| field.key);
                    }
                    existing_fields.append(&mut Vec::from(new_fields));
                };
            }
        }

        self.data_parts[existing_object_id.part_id][existing_fields_id] = existing_fields;
    }

    fn recursive_merge_value_in_place(&mut self, existing: &ResponseValue, new: ResponseValue) {
        match (existing, new) {
            (&ResponseValue::Object { id: existing_id, .. }, ResponseValue::Object { id: new_id, .. }) => {
                self.recursive_merge_object_in_place(
                    existing_id,
                    new_id.part_id,
                    self.data_parts[new_id].fields_sorted_by_key,
                );
            }
            (
                &ResponseValue::List {
                    id: existing_list_id,
                    offset: existing_offset,
                    limit: existing_limit,
                },
                ResponseValue::List {
                    id: new_list_id,
                    offset: new_offset,
                    limit: new_limit,
                },
            ) => {
                assert_eq!(existing_limit, new_limit, "Trying to merge lists with different sizes");
                self.recursive_merge_list_in_place(
                    existing_list_id,
                    existing_offset,
                    new_list_id,
                    new_offset,
                    existing_limit,
                )
            }
            (&ResponseValue::Inaccessible { id: existing_id }, ResponseValue::Inaccessible { id: new_id }) => {
                let existing = take(&mut self.data_parts[existing_id]);
                let new_value = take(&mut self.data_parts[new_id]);
                self.recursive_merge_value_in_place(&existing, new_value);
                self.data_parts[existing_id] = existing;
            }
            (&ResponseValue::Inaccessible { id }, new) => {
                let existing = take(&mut self.data_parts[id]);
                self.recursive_merge_value_in_place(&existing, new);
                self.data_parts[id] = existing;
            }
            (existing, ResponseValue::Inaccessible { id: new_id }) => {
                let new_value = take(&mut self.data_parts[new_id]);
                self.recursive_merge_value_in_place(existing, new_value);
            }
            (ResponseValue::Null, ResponseValue::Null) => {}
            (l, r) => {
                unreachable!(
                    "Trying to merge something values that aren't a couple of objects/list/couple of nulls {l:?} | {r:?}"
                );
            }
        }
    }

    fn recursive_merge_list_in_place(
        &mut self,
        existing_list_id: ResponseListId,
        existing_offset: u32,
        new_list_id: ResponseListId,
        new_offset: u32,
        length: u32,
    ) {
        for i in 0..length as usize {
            let existing = take(
                self.data_parts[existing_list_id]
                    .get_mut(existing_offset as usize + i)
                    .expect("List length mismatch during merge"),
            );
            let new = take(
                self.data_parts[new_list_id]
                    .get_mut(new_offset as usize + i)
                    .expect("List length mismatch during merge"),
            );
            self.recursive_merge_value_in_place(&existing, new);
            self.data_parts[existing_list_id][existing_offset as usize + i] = existing;
        }
    }
}
