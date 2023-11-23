use std::cmp::Ordering;

use crate::FieldId;

#[derive(Debug, Default, Clone)]
pub struct FieldSet {
    // sorted by field id
    items: Vec<FieldSetItem>,
}

impl FromIterator<FieldSetItem> for FieldSet {
    fn from_iter<T: IntoIterator<Item = FieldSetItem>>(iter: T) -> Self {
        let mut items = iter.into_iter().collect::<Vec<_>>();
        items.sort_unstable_by_key(|selection| selection.field);
        Self { items }
    }
}

impl IntoIterator for FieldSet {
    type Item = FieldSetItem;

    type IntoIter = <Vec<FieldSetItem> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a> IntoIterator for &'a FieldSet {
    type Item = &'a FieldSetItem;

    type IntoIter = <&'a Vec<FieldSetItem> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
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
            .binary_search_by_key(&field, |selection| selection.field)
            .ok()?;
        Some(&self.items[index])
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
            match left.field.cmp(&right.field) {
                Ordering::Less => {
                    items.push(left.clone());
                    l += 1;
                }
                Ordering::Equal => {
                    items.push(right.clone());
                    r += 1;
                }
                Ordering::Greater => {
                    items.push(FieldSetItem {
                        field: left.field,
                        selection_set: Self::merge(&left.selection_set, &right.selection_set),
                    });
                    l += 1;
                    r += 1;
                }
            }
        }
        FieldSet { items }
    }
}

#[derive(Debug, Clone)]
pub struct FieldSetItem {
    pub field: FieldId,
    pub selection_set: FieldSet,
}
