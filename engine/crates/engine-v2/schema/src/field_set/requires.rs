use walker::{Iter, Walk};

use crate::{RequiredField, RequiredFieldId, Schema, StringId};
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

impl<'a> Walk<&'a Schema> for RequiredFieldSetId {
    type Walker<'w> = RequiredFieldSet<'w> where 'a: 'w;
    fn walk<'w>(self, schema: &'a Schema) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        RequiredFieldSet {
            schema,
            ref_: &schema[self],
        }
    }
}

impl<'a> Walk<&'a Schema> for &RequiredFieldSetRecord {
    type Walker<'w> = RequiredFieldSet<'w> where Self: 'w, 'a: 'w;
    fn walk<'w>(self, schema: &'a Schema) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
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
    /// If no alias is provided, it's set to field name
    pub alias_id: StringId,
    pub id: RequiredFieldId,
    pub subselection: RequiredFieldSetRecord,
}

impl<'a> Walk<&'a Schema> for &RequiredFieldSetItemRecord {
    type Walker<'w> = RequiredFieldSetItem<'w> where Self: 'w, 'a: 'w;
    fn walk<'w>(self, schema: &'a Schema) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
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
        self.ref_.id.walk(self.schema)
    }
    pub fn subselection(&self) -> RequiredFieldSet<'a> {
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
        fields.sort_unstable_by_key(|field| field.id);
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
                    fields.push(RequiredFieldSetItemRecord {
                        alias_id: left.alias_id,
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
