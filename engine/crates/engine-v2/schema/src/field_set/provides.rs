use std::cmp::Ordering;

use crate::FieldDefinitionId;

static EMPTY: ProvidableFieldSet = ProvidableFieldSet(Vec::new());

impl ProvidableFieldSet {
    pub fn empty() -> &'static ProvidableFieldSet {
        &EMPTY
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProvidableFieldSet(Vec<ProvidableField>);

impl From<Vec<ProvidableField>> for ProvidableFieldSet {
    fn from(mut fields: Vec<ProvidableField>) -> Self {
        fields.sort_unstable_by_key(|field| field.id);
        Self(fields)
    }
}

impl FromIterator<ProvidableField> for ProvidableFieldSet {
    fn from_iter<T: IntoIterator<Item = ProvidableField>>(iter: T) -> Self {
        let mut fields = iter.into_iter().collect::<Vec<_>>();
        fields.sort_unstable_by_key(|field| field.id);
        Self(fields)
    }
}

impl<'a> IntoIterator for &'a ProvidableFieldSet {
    type Item = &'a ProvidableField;
    type IntoIter = std::slice::Iter<'a, ProvidableField>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl ProvidableFieldSet {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(&self, field: FieldDefinitionId) -> Option<&ProvidableField> {
        let index = self.0.binary_search_by_key(&field, |field| field.id).ok()?;
        Some(&self.0[index])
    }

    pub fn contains(&self, field: FieldDefinitionId) -> bool {
        self.0.binary_search_by_key(&field, |field| field.id).is_ok()
    }

    pub fn union_opt(
        left_set: Option<&ProvidableFieldSet>,
        right_set: Option<&ProvidableFieldSet>,
    ) -> ProvidableFieldSet {
        match (left_set, right_set) {
            (Some(left_set), Some(right_set)) => left_set.union(right_set),
            (Some(left_set), None) => left_set.clone(),
            (None, Some(right_set)) => right_set.clone(),
            (None, None) => ProvidableFieldSet::default(),
        }
    }

    pub fn update(&mut self, other: &ProvidableFieldSet) {
        self.0 = self.union(other).0;
    }

    pub fn union(&self, right_set: &ProvidableFieldSet) -> ProvidableFieldSet {
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
                    fields.push(ProvidableField {
                        id: left.id,
                        subselection: left.subselection.union(&right.subselection),
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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProvidableField {
    pub id: FieldDefinitionId,
    pub subselection: ProvidableFieldSet,
}
