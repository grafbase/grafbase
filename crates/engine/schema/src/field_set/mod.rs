mod field;
mod item;
mod union;

use walker::{Iter, Walk};

pub use item::*;

use crate::Schema;

static EMPTY: FieldSetRecord = FieldSetRecord(Vec::new());

impl FieldSetRecord {
    pub fn empty() -> &'static Self {
        &EMPTY
    }
}

#[derive(Default, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone)]
pub struct FieldSetRecord(Vec<FieldSetItemRecord>);

impl FieldSetRecord {
    pub(crate) fn insert(&mut self, item: FieldSetItemRecord) {
        match self.0.binary_search_by_key(&item.field_id, |i| i.field_id) {
            Ok(idx) => {
                self.0[idx].subselection_record = self.0[idx].subselection_record.union(&item.subselection_record);
            }
            Err(idx) => self.0.insert(idx, item),
        }
    }
}

impl std::ops::Deref for FieldSetRecord {
    type Target = [FieldSetItemRecord];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromIterator<FieldSetItemRecord> for FieldSetRecord {
    fn from_iter<T: IntoIterator<Item = FieldSetItemRecord>>(iter: T) -> Self {
        iter.into_iter().collect::<Vec<_>>().into()
    }
}

impl From<Vec<FieldSetItemRecord>> for FieldSetRecord {
    fn from(mut items: Vec<FieldSetItemRecord>) -> Self {
        items.sort_unstable_by(|a, b| a.field_id.cmp(&b.field_id));
        Self(items)
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
    type Walker<'w>
        = FieldSet<'w>
    where
        Self: 'w,
        'a: 'w;
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

impl std::fmt::Debug for FieldSet<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FieldSet").field(&self.items()).finish()
    }
}
