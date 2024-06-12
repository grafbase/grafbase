use std::cmp::Ordering;

use crate::{FieldDefinitionId, InputValueDefinitionId, RequiredFieldSetArgumentsId, SchemaInputValueId};

pub(crate) static EMPTY: RequiredFieldSet = RequiredFieldSet(Vec::new());

#[derive(Default, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RequiredFieldSet(Vec<RequiredField>);

impl FromIterator<RequiredField> for RequiredFieldSet {
    fn from_iter<T: IntoIterator<Item = RequiredField>>(iter: T) -> Self {
        let mut fields = iter.into_iter().collect::<Vec<_>>();
        fields.sort_unstable();
        Self(fields)
    }
}

impl std::ops::Deref for RequiredFieldSet {
    type Target = [RequiredField];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> IntoIterator for &'a RequiredFieldSet {
    type Item = &'a RequiredField;
    type IntoIter = std::slice::Iter<'a, RequiredField>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl RequiredFieldSet {
    pub fn union_opt(left_set: Option<&Self>, right_set: Option<&Self>) -> Self {
        match (left_set, right_set) {
            (Some(left_set), Some(right_set)) => left_set.union(right_set),
            (Some(left_set), None) => left_set.clone(),
            (None, Some(right_set)) => right_set.clone(),
            (None, None) => Self::default(),
        }
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
            match left.cmp(right) {
                Ordering::Less => {
                    fields.push(left.clone());
                    l += 1;
                }
                Ordering::Greater => {
                    fields.push(right.clone());
                    r += 1;
                }
                Ordering::Equal => {
                    fields.push(RequiredField {
                        id: left.id,
                        definition_id: left.definition_id,
                        arguments_id: left.arguments_id,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct RequiredFieldId(u32);

impl From<u32> for RequiredFieldId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RequiredField {
    /// Unique id used during planning to associate a FieldId to a required field.
    pub id: RequiredFieldId,
    pub definition_id: FieldDefinitionId,
    pub arguments_id: Option<RequiredFieldSetArgumentsId>,
    pub subselection: RequiredFieldSet,
}

impl Ord for RequiredField {
    fn cmp(&self, other: &Self) -> Ordering {
        self.definition_id
            .cmp(&other.definition_id)
            // Arguments are deduplicated
            .then_with(|| self.arguments_id.cmp(&other.arguments_id))
    }
}

impl PartialEq for RequiredField {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Eq for RequiredField {}

impl PartialOrd for RequiredField {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// sorted by InputValueDefinitionId
#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct RequiredFieldArguments(pub(crate) Vec<(InputValueDefinitionId, SchemaInputValueId)>);

impl std::ops::Deref for RequiredFieldArguments {
    type Target = [(InputValueDefinitionId, SchemaInputValueId)];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
