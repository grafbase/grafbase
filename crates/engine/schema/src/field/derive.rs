use walker::Walk;

use crate::{DeriveObject, DeriveObjectRecord, DeriveScalarAsField, DeriveScalarAsFieldRecord, Schema};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum DeriveMappingRecord {
    Object(DeriveObjectRecord),
    ScalarAsField(DeriveScalarAsFieldRecord),
}

pub enum DeriveMapping<'a> {
    Object(DeriveObject<'a>),
    ScalarAsField(DeriveScalarAsField<'a>),
}

impl<'a> Walk<&'a Schema> for &'a DeriveMappingRecord {
    type Walker<'w>
        = DeriveMapping<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        match self {
            DeriveMappingRecord::Object(record) => DeriveMapping::Object(record.walk(schema)),
            DeriveMappingRecord::ScalarAsField(record) => DeriveMapping::ScalarAsField(record.walk(schema)),
        }
    }
}

impl std::fmt::Debug for DeriveMapping<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeriveMapping::Object(obj) => f.debug_tuple("DeriveMapping::Object").field(obj).finish(),
            DeriveMapping::ScalarAsField(scalar_as_field) => f
                .debug_tuple("DeriveMapping::ScalarAsField")
                .field(scalar_as_field)
                .finish(),
        }
    }
}
