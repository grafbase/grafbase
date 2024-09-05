use std::{borrow::Cow, cmp::Ordering};

use crate::{FieldDefinitionId, InputValueDefinitionId, RequiredFieldId, SchemaInputValueId};

pub(crate) static EMPTY: RequiredFieldSet = RequiredFieldSet(Vec::new());

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RequiredFieldSet(Vec<RequiredFieldSetItem>);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RequiredFieldSetItem {
    pub id: RequiredFieldId,
    pub subselection: RequiredFieldSet,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct RequiredField {
    pub definition_id: FieldDefinitionId,
    // sorted by InputValueDefinitionId
    pub arguments: Vec<(InputValueDefinitionId, SchemaInputValueId)>,
}

impl FromIterator<RequiredFieldSetItem> for RequiredFieldSet {
    fn from_iter<T: IntoIterator<Item = RequiredFieldSetItem>>(iter: T) -> Self {
        let mut fields = iter.into_iter().collect::<Vec<_>>();
        fields.sort_unstable_by_key(|field| field.id);
        Self(fields)
    }
}

impl std::ops::Deref for RequiredFieldSet {
    type Target = [RequiredFieldSetItem];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> IntoIterator for &'a RequiredFieldSet {
    type Item = &'a RequiredFieldSetItem;
    type IntoIter = std::slice::Iter<'a, RequiredFieldSetItem>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl RequiredFieldSet {
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
        let left_set = &self.0;
        let right_set = &right_set.0;
        // Allocating too much, but doesn't really matter. FieldSet will always be relatively small
        // anyway.
        let mut fields = Vec::with_capacity(left_set.len() + right_set.len());
        let mut l = 0;
        let mut r = 0;
        while l < left_set.len() && r < right_set.len() {
            let left = &left_set[l];
            let right = &right_set[r];
            match left.id.cmp(&right.id) {
                Ordering::Less => {
                    fields.push(left.clone());
                    l += 1;
                }
                Ordering::Greater => {
                    fields.push(right.clone());
                    r += 1;
                }
                Ordering::Equal => {
                    fields.push(RequiredFieldSetItem {
                        id: left.id,
                        subselection: if left.subselection.is_empty() {
                            right.subselection.clone()
                        } else if right.subselection.is_empty() {
                            left.subselection.clone()
                        } else {
                            left.subselection.union(&right.subselection)
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
