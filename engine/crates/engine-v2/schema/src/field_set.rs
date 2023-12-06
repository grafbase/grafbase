use std::cmp::Ordering;

use crate::{FieldId, SchemaWalker};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldSet {
    // sorted by field id
    items: Box<[FieldSetItem]>,
}

impl FromIterator<FieldSetItem> for FieldSet {
    fn from_iter<T: IntoIterator<Item = FieldSetItem>>(iter: T) -> Self {
        let mut items = iter.into_iter().collect::<Vec<_>>();
        items.sort_unstable_by_key(|selection| selection.field_id);
        Self {
            items: items.into_boxed_slice(),
        }
    }
}

impl IntoIterator for FieldSet {
    type Item = FieldSetItem;

    type IntoIter = <Vec<FieldSetItem> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_vec().into_iter()
    }
}

impl<'a> IntoIterator for &'a FieldSet {
    type Item = &'a FieldSetItem;

    type IntoIter = <&'a Vec<FieldSetItem> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl Default for FieldSet {
    fn default() -> Self {
        Self {
            items: Vec::with_capacity(0).into_boxed_slice(),
        }
    }
}

impl FieldSet {
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &FieldSetItem> + '_ {
        self.items.iter()
    }

    pub fn get(&self, field: FieldId) -> Option<&FieldSetItem> {
        let index = self
            .items
            .binary_search_by_key(&field, |selection| selection.field_id)
            .ok()?;
        Some(&self.items[index])
    }

    pub fn contains(&self, field: FieldId) -> bool {
        self.items
            .binary_search_by_key(&field, |selection| selection.field_id)
            .is_ok()
    }

    pub fn merge_opt(left_set: Option<&FieldSet>, right_set: Option<&FieldSet>) -> FieldSet {
        match (left_set, right_set) {
            (Some(left_set), Some(right_set)) => Self::merge(left_set, right_set),
            (Some(left_set), None) => left_set.clone(),
            (None, Some(right_set)) => right_set.clone(),
            (None, None) => FieldSet::default(),
        }
    }

    pub fn merge(left_set: &FieldSet, right_set: &FieldSet) -> FieldSet {
        let mut items = vec![];
        let mut l = 0;
        let mut r = 0;
        while l < left_set.items.len() && r < right_set.items.len() {
            let left = &left_set.items[l];
            let right = &right_set.items[r];
            match left.field_id.cmp(&right.field_id) {
                Ordering::Less => {
                    items.push(left.clone());
                    l += 1;
                }
                Ordering::Greater => {
                    items.push(right.clone());
                    r += 1;
                }
                Ordering::Equal => {
                    items.push(FieldSetItem {
                        field_id: left.field_id,
                        selection_set: Self::merge(&left.selection_set, &right.selection_set),
                    });
                    l += 1;
                    r += 1;
                }
            }
        }
        if l < left_set.items.len() {
            items.extend_from_slice(&left_set.items[l..]);
        }
        if r < right_set.items.len() {
            items.extend_from_slice(&right_set.items[r..]);
        }
        FieldSet {
            items: items.into_boxed_slice(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldSetItem {
    pub field_id: FieldId,
    pub selection_set: FieldSet,
}

impl<'a> std::fmt::Debug for SchemaWalker<'a, &'a FieldSet> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FieldSet")
            .field(&self.inner.items.iter().map(|item| self.walk(item)).collect::<Vec<_>>())
            .finish()
    }
}

impl<'a> std::fmt::Debug for SchemaWalker<'a, &'a FieldSetItem> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.inner.selection_set.is_empty() {
            f.debug_struct("FieldSetItem")
                .field("name", &self.walk(self.inner.field_id).name())
                .field("selection_set", &self.walk(&self.inner.selection_set))
                .finish()
        } else {
            self.walk(self.inner.field_id).name().fmt(f)
        }
    }
}
