use crate::FieldSetRecord;
use std::{borrow::Cow, cmp::Ordering};

use super::FieldSetItemRecord;

impl FieldSetRecord {
    pub fn union_cow<'a>(left: Cow<'a, Self>, right: Cow<'a, Self>) -> Cow<'a, Self> {
        if left.is_empty() {
            return right;
        }
        if right.is_empty() {
            return left;
        }
        Cow::Owned(left.union(&right))
    }

    pub fn union(&self, right_set: &Self) -> Self {
        let left_set = &self;
        let right_set = &right_set;

        // Allocating too much, but doesn't really matter. FieldSet will always be relatively small
        // anyway.
        let mut fields = Vec::with_capacity(left_set.len() + right_set.len());
        let mut l = 0;
        let mut r = 0;
        while l < left_set.len() && r < right_set.len() {
            let left = &left_set[l];
            let right = &right_set[r];
            match left.alias_id.cmp(&right.alias_id).then(left.id.cmp(&right.id)) {
                Ordering::Less => {
                    fields.push(left.clone());
                    l += 1;
                }
                Ordering::Greater => {
                    fields.push(right.clone());
                    r += 1;
                }
                Ordering::Equal => {
                    fields.push(FieldSetItemRecord {
                        alias_id: left.alias_id,
                        id: left.id,
                        subselection_record: if left.subselection_record.is_empty() {
                            right.subselection_record.clone()
                        } else {
                            left.subselection_record.union(&right.subselection_record)
                        },
                    });
                    l += 1;
                    r += 1;
                }
            }
        }
        if l < left_set.len() {
            fields.extend_from_slice(&left_set[l..]);
        } else if r < right_set.len() {
            fields.extend_from_slice(&right_set[r..]);
        }

        Self(fields)
    }
}
