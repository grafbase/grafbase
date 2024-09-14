use walker::{Iter, Walk};

use crate::{RequiredField, RequiredFieldId, Schema};
use std::{borrow::Cow, cmp::Ordering};

static EMPTY: RequiredFieldSetRecord = RequiredFieldSetRecord(Vec::new());

impl RequiredFieldSetRecord {
    pub fn empty() -> &'static Self {
        &EMPTY
    }
}

//
// RequiredFieldSet
//
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RequiredFieldSetRecord(Vec<RequiredFieldSetItemRecord>);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct RequiredFieldSetId(std::num::NonZero<u32>);

impl Walk<Schema> for RequiredFieldSetId {
    type Walker<'a> = RequiredFieldSet<'a>;
    fn walk<'s>(self, schema: &'s Schema) -> Self::Walker<'s>
    where
        Self: 's,
    {
        RequiredFieldSet {
            schema,
            ref_: &schema[self],
        }
    }
}

impl Walk<Schema> for &RequiredFieldSetRecord {
    type Walker<'a> = RequiredFieldSet<'a> where Self: 'a;
    fn walk<'s>(self, schema: &'s Schema) -> Self::Walker<'s>
    where
        Self: 's,
    {
        RequiredFieldSet { schema, ref_: self }
    }
}

#[derive(Clone, Copy)]
pub struct RequiredFieldSet<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) ref_: &'a RequiredFieldSetRecord,
}

impl<'a> RequiredFieldSet<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a RequiredFieldSetRecord {
        self.ref_
    }

    pub fn items(&self) -> impl Iter<Item = RequiredFieldSetItem<'a>> + 'a {
        self.ref_.0.walk(self.schema)
    }
}

impl std::fmt::Debug for RequiredFieldSet<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.items()).finish()
    }
}

//
// RequiredFieldSetItem
//
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RequiredFieldSetItemRecord {
    pub field_id: RequiredFieldId,
    pub subselection: RequiredFieldSetRecord,
}

impl Walk<Schema> for &RequiredFieldSetItemRecord {
    type Walker<'a> = RequiredFieldSetItem<'a> where Self: 'a;
    fn walk<'s>(self, schema: &'s Schema) -> Self::Walker<'s>
    where
        Self: 's,
    {
        RequiredFieldSetItem { schema, ref_: self }
    }
}

#[derive(Clone, Copy)]
pub struct RequiredFieldSetItem<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) ref_: &'a RequiredFieldSetItemRecord,
}

impl<'a> RequiredFieldSetItem<'a> {
    pub fn field(&self) -> RequiredField<'a> {
        self.ref_.field_id.walk(self.schema)
    }
    pub fn subselection(&self) -> RequiredFieldSet<'_> {
        self.ref_.subselection.walk(self.schema)
    }
}

impl std::fmt::Debug for RequiredFieldSetItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RequiredFieldSetItem")
            .field("field", &self.field())
            .field("subselection", &self.subselection())
            .finish()
    }
}

//
// Utilities
//
impl FromIterator<RequiredFieldSetItemRecord> for RequiredFieldSetRecord {
    fn from_iter<T: IntoIterator<Item = RequiredFieldSetItemRecord>>(iter: T) -> Self {
        let mut fields = iter.into_iter().collect::<Vec<_>>();
        fields.sort_unstable_by_key(|field| field.field_id);
        Self(fields)
    }
}

impl std::ops::Deref for RequiredFieldSetRecord {
    type Target = [RequiredFieldSetItemRecord];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> IntoIterator for &'a RequiredFieldSetRecord {
    type Item = &'a RequiredFieldSetItemRecord;
    type IntoIter = std::slice::Iter<'a, RequiredFieldSetItemRecord>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl RequiredFieldSetRecord {
    /// Creates a union of two `RequiredFieldSetRecord`.
    ///
    /// If either `left` or `right` is empty, the other is returned. If neither is empty,
    /// a new `RequiredFieldSetRecord` is created containing unique items from both sets.
    ///
    /// # Parameters
    ///
    /// - `left`: A `Cow` representing the left set to union.
    /// - `right`: A `Cow` representing the right set to union.
    ///
    /// # Returns
    ///
    /// A `Cow` containing the union of the two sets.
    pub fn union_cow<'a>(left: Cow<'a, Self>, right: Cow<'a, Self>) -> Cow<'a, Self> {
        if left.is_empty() {
            return right;
        }
        if right.is_empty() {
            return left;
        }

        Cow::Owned(left.union(&right))
    }

    /// Creates a union of two `RequiredFieldSetRecord`.
    ///
    /// If either `self` or `right_set` is empty, the other is returned. If neither is empty,
    /// a new `RequiredFieldSetRecord` is created containing unique items from both sets.
    ///
    /// # Parameters
    ///
    /// - `right_set`: A reference to another `RequiredFieldSetRecord` to union with this one.
    ///
    /// # Returns
    ///
    /// A new `RequiredFieldSetRecord` containing the union of the two sets.
    pub fn union(&self, right_set: &Self) -> Self {
        let left_set = &self.0;
        let right_set = &right_set.0;

        // Allocating too much, but doesn't really matter. FieldSet will always be relatively small
        // anyway.
        let mut fields = Vec::with_capacity(left_set.len() + right_set.len());
        let mut left_index = 0;
        let mut right_index = 0;

        while left_index < left_set.len() && right_index < right_set.len() {
            let left = &left_set[left_index];
            let right = &right_set[right_index];
            match left.field_id.cmp(&right.field_id) {
                Ordering::Less => {
                    fields.push(left.clone());
                    left_index += 1;
                }
                Ordering::Greater => {
                    fields.push(right.clone());
                    right_index += 1;
                }
                Ordering::Equal => {
                    fields.push(RequiredFieldSetItemRecord {
                        field_id: left.field_id,
                        subselection: if left.subselection.is_empty() {
                            right.subselection.clone()
                        } else if right.subselection.is_empty() {
                            left.subselection.clone()
                        } else {
                            left.subselection.union(&right.subselection)
                        },
                    });
                    left_index += 1;
                    right_index += 1;
                }
            }
        }

        if left_index < left_set.len() {
            fields.extend_from_slice(&left_set[left_index..]);
        } else if right_index < right_set.len() {
            fields.extend_from_slice(&right_set[right_index..]);
        }

        Self(fields)
    }
}

impl std::fmt::Debug for RequiredField<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RequiredField")
            .field("name", &self.definition().name())
            .field("arguments", &ArgumentsDebug(self))
            .finish()
    }
}

struct ArgumentsDebug<'a>(&'a RequiredField<'a>);

impl std::fmt::Debug for ArgumentsDebug<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.0.arguments().map(|arg| (arg.definition().name(), arg.value())))
            .finish()
    }
}
