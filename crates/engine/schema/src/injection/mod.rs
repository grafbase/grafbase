use id_newtypes::IdRange;

use crate::{
    ArgumentInjectionId, ArgumentInjectionRecord, KeyValueInjectionId, KeyValueInjectionRecord, SchemaFieldArgumentId,
    SchemaFieldArgumentRecord, SchemaFieldId, SchemaFieldRecord, SchemaInputValueId, StringId,
};

#[derive(Clone, Default, id_derives::IndexedFields, serde::Serialize, serde::Deserialize)]
pub struct Selections {
    // deduplicated
    #[indexed_by(SchemaFieldId)]
    pub(crate) fields: Vec<SchemaFieldRecord>,
    #[indexed_by(SchemaFieldArgumentId)]
    pub(crate) arguments: Vec<SchemaFieldArgumentRecord>,
    #[indexed_by(ArgumentInjectionId)]
    pub(crate) argument_injections: Vec<ArgumentInjectionRecord>,
    #[indexed_by(ArgumentValueInjectionId)]
    pub(crate) argument_value_injections: Vec<ArgumentValueInjection>,
    #[indexed_by(ValueInjectionId)]
    pub(crate) injections: Vec<ValueInjection>,
    #[indexed_by(KeyValueInjectionId)]
    pub(crate) key_value_injections: Vec<KeyValueInjectionRecord>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, id_derives::Id, serde::Serialize, serde::Deserialize)]
pub struct ArgumentValueInjectionId(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub enum ArgumentValueInjection {
    Value(ValueInjection),
    Nested {
        key: StringId,
        value: ArgumentValueInjectionId,
    },
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, id_derives::Id, serde::Serialize, serde::Deserialize)]
pub struct ValueInjectionId(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub enum ValueInjection {
    Identity,
    DefaultValue(SchemaInputValueId),
    Select {
        field_id: SchemaFieldId,
        next: ValueInjectionId,
    },
    // sorted by field_id if it exists.
    Object(IdRange<KeyValueInjectionId>),
    // sorted by field_id if it exists.
    OneOf(IdRange<ValueInjectionId>),
}
