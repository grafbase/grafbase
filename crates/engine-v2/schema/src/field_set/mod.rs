mod field;
mod item;
mod union;

use walker::{Iter, Walk};

pub use item::*;

use crate::{Schema, MAX_ID};

static EMPTY: FieldSetRecord = FieldSetRecord(Vec::new());

impl FieldSetRecord {
    pub fn empty() -> &'static Self {
        &EMPTY
    }
}

#[derive(Default, Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct FieldSetRecord(Vec<FieldSetItemRecord>);

impl std::ops::Deref for FieldSetRecord {
    type Target = [FieldSetItemRecord];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromIterator<FieldSetItemRecord> for FieldSetRecord {
    fn from_iter<T: IntoIterator<Item = FieldSetItemRecord>>(iter: T) -> Self {
        let mut items_record = iter.into_iter().collect::<Vec<_>>();
        items_record.sort_unstable_by_key(|field| field.id);
        Self(items_record)
    }
}

impl<'a> IntoIterator for &'a FieldSetRecord {
    type Item = &'a FieldSetItemRecord;
    type IntoIter = std::slice::Iter<'a, FieldSetItemRecord>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[derive(Clone, Copy)]
pub struct FieldSet<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) ref_: &'a FieldSetRecord,
}

impl std::ops::Deref for FieldSet<'_> {
    type Target = FieldSetRecord;
    fn deref(&self) -> &Self::Target {
        self.ref_
    }
}

impl<'a> FieldSet<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a FieldSetRecord {
        self.ref_
    }
    pub fn items(&self) -> impl Iter<Item = FieldSetItem<'a>> + 'a {
        self.as_ref().0.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for &FieldSetRecord {
    type Walker<'w> = FieldSet<'w> where Self : 'w , 'a: 'w ;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        FieldSet {
            schema: schema.into(),
            ref_: self,
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
#[max(MAX_ID)]
pub struct FieldSetId(std::num::NonZero<u32>);

impl<'a> Walk<&'a Schema> for FieldSetId {
    type Walker<'w> = FieldSet<'w> where Self : 'w , 'a: 'w ;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        let schema = schema.into();
        FieldSet {
            schema,
            ref_: &schema[self],
        }
    }
}

impl std::fmt::Debug for FieldSet<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FieldSet").field(&self.items()).finish()
    }
}
