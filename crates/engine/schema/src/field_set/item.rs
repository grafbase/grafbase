use walker::Walk;

use crate::{FieldSet, FieldSetRecord, Schema, SchemaField, SchemaFieldId};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FieldSetItemRecord {
    pub field_id: SchemaFieldId,
    pub subselection_record: FieldSetRecord,
}

impl<'a> Walk<&'a Schema> for &FieldSetItemRecord {
    type Walker<'w>
        = FieldSetItem<'w>
    where
        Self: 'w,
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        FieldSetItem {
            schema: schema.into(),
            ref_: self,
        }
    }
}

#[derive(Clone, Copy)]
pub struct FieldSetItem<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) ref_: &'a FieldSetItemRecord,
}

impl std::ops::Deref for FieldSetItem<'_> {
    type Target = FieldSetItemRecord;
    fn deref(&self) -> &Self::Target {
        self.ref_
    }
}

impl<'a> FieldSetItem<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a FieldSetItemRecord {
        self.ref_
    }
    pub fn field(&self) -> SchemaField<'a> {
        self.ref_.field_id.walk(self.schema)
    }
    pub fn subselection(&self) -> FieldSet<'a> {
        self.ref_.subselection_record.walk(self.schema)
    }
}

impl std::fmt::Debug for FieldSetItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldSetItem")
            .field("field", &self.field())
            .field("subselection", &self.subselection())
            .finish()
    }
}
