use std::num::NonZero;

use id_newtypes::IdRange;
use walker::Walk;

use crate::{
    InputValueDefinitionId, Schema, SchemaFieldArgumentId, SchemaFieldArgumentRecord, SchemaFieldId, SchemaFieldRecord,
    SchemaInputValueId,
};

#[derive(Default, id_derives::IndexedFields, serde::Serialize, serde::Deserialize)]
pub struct Selections {
    // deduplicated
    #[indexed_by(SchemaFieldId)]
    pub(crate) fields: Vec<SchemaFieldRecord>,
    #[indexed_by(SchemaFieldArgumentId)]
    pub(crate) arguments: Vec<SchemaFieldArgumentRecord>,
    #[indexed_by(InputValueInjectionId)]
    pub(crate) injections: Vec<InputValueInjection>,
    #[indexed_by(ValueInjectionId)]
    pub(crate) mapping: Vec<ValueInjection>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, id_derives::Id, serde::Serialize, serde::Deserialize)]
pub struct InputValueInjectionId(u32);

impl<'a> Walk<&'a Schema> for InputValueInjectionId {
    type Walker<'w>
        = &'w InputValueInjection
    where
        Self: 'w,
        'a: 'w;

    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        let schema: &'a Schema = schema.into();
        &schema[self]
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, id_derives::Id, serde::Serialize, serde::Deserialize)]
pub struct ValueInjectionId(NonZero<u32>);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub enum ValueInjection {
    Const(SchemaInputValueId),
    Select {
        field_id: SchemaFieldId,
        next: Option<ValueInjectionId>,
    },
    // sorted by field_id if it exists.
    Object(IdRange<InputValueInjectionId>),
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct InputValueInjection {
    pub definition_id: InputValueDefinitionId,
    pub injection: ValueInjection,
}
