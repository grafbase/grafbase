use walker::Walk;

use crate::{FieldSet, FieldSetRecord, Schema, SchemaField, SchemaFieldId, StringId};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FieldSetItemRecord {
    /// If no alias is provided, it's set to field name
    pub alias_id: StringId,
    pub id: SchemaFieldId,
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

impl<'a> FieldSetItem<'a> {
    pub fn field(&self) -> SchemaField<'a> {
        self.ref_.id.walk(self.schema)
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
