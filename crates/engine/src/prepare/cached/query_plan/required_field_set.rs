use std::cmp::Ordering;

use schema::SchemaFieldId;
use walker::Walk;

use crate::prepare::CachedOperationContext;

use super::{PartitionDataField, PartitionDataFieldId};

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct RequiredFieldSetRecord(Vec<RequiredFieldSetItemRecord>);

impl std::ops::Deref for RequiredFieldSetRecord {
    type Target = [RequiredFieldSetItemRecord];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vec<RequiredFieldSetItemRecord>> for RequiredFieldSetRecord {
    fn from(mut items: Vec<RequiredFieldSetItemRecord>) -> Self {
        items.sort_unstable_by(|a, b| a.matching_field_id.cmp(&b.matching_field_id));
        Self(items)
    }
}

impl FromIterator<RequiredFieldSetItemRecord> for RequiredFieldSetRecord {
    fn from_iter<T: IntoIterator<Item = RequiredFieldSetItemRecord>>(iter: T) -> Self {
        iter.into_iter().collect::<Vec<_>>().into()
    }
}

#[derive(Clone, Copy)]
pub(crate) struct RequiredFieldSet<'a> {
    ctx: CachedOperationContext<'a>,
    ref_: &'a RequiredFieldSetRecord,
}

impl<'a> RequiredFieldSet<'a> {
    pub fn len(&self) -> usize {
        self.ref_.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ref_.0.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<RequiredFieldSetItem<'a>> {
        self.ref_.0.get(index).map(|item| item.walk(self.ctx))
    }

    pub fn iter(&self) -> impl Iterator<Item = RequiredFieldSetItem<'a>> + 'a {
        let ctx = self.ctx;
        self.ref_.0.iter().map(move |field| field.walk(ctx))
    }
}

impl<'a> Walk<CachedOperationContext<'a>> for &RequiredFieldSetRecord {
    type Walker<'w>
        = RequiredFieldSet<'w>
    where
        Self: 'w,
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<CachedOperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        RequiredFieldSet {
            ctx: ctx.into(),
            ref_: self,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct RequiredFieldSetItemRecord {
    pub data_field_id: PartitionDataFieldId,
    pub matching_field_id: SchemaFieldId,
    pub subselection_record: RequiredFieldSetRecord,
}

#[derive(Clone, Copy)]
pub(crate) struct RequiredFieldSetItem<'a> {
    ctx: CachedOperationContext<'a>,
    ref_: &'a RequiredFieldSetItemRecord,
}

impl std::ops::Deref for RequiredFieldSetItem<'_> {
    type Target = RequiredFieldSetItemRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> RequiredFieldSetItem<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a RequiredFieldSetItemRecord {
        self.ref_
    }
    pub fn data_field(&self) -> PartitionDataField<'a> {
        self.ref_.data_field_id.walk(self.ctx)
    }
    pub fn subselection(&self) -> RequiredFieldSet<'a> {
        self.ref_.subselection_record.walk(self.ctx)
    }
}

impl<'a> Walk<CachedOperationContext<'a>> for &RequiredFieldSetItemRecord {
    type Walker<'w>
        = RequiredFieldSetItem<'w>
    where
        Self: 'w,
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<CachedOperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        RequiredFieldSetItem {
            ctx: ctx.into(),
            ref_: self,
        }
    }
}

impl std::fmt::Debug for RequiredFieldSetItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Field")
            .field(
                "field",
                &&self.ctx.cached.operation.response_keys[self.data_field().response_key],
            )
            .field("subselection", &self.subselection_record.walk(self.ctx))
            .finish()
    }
}

impl std::fmt::Debug for RequiredFieldSet<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FieldSet").field(&self.ref_.0.walk(self.ctx)).finish()
    }
}

impl RequiredFieldSetRecord {
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
            match left.matching_field_id.cmp(&right.matching_field_id) {
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
                        data_field_id: left.data_field_id,
                        matching_field_id: left.matching_field_id,
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
