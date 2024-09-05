use std::num::NonZero;

use crate::{EnumValueId, IdRange, InputValueDefinition, InputValueDefinitionId, Schema, StringId};

mod error;
mod reader;
mod set;
#[cfg(test)]
mod tests;
mod value;

pub use error::*;
use readable::Readable;
pub use reader::*;
pub use set::*;
pub use value::*;

#[derive(Default, serde::Serialize, serde::Deserialize, id_derives::IndexedFields)]
pub struct SchemaInputValues {
    /// Individual input values and list values
    #[indexed_by(SchemaInputValueId)]
    values: Vec<SchemaInputValueRecord>,
    /// InputObject's fields
    #[indexed_by(SchemaInputObjectFieldValueId)]
    input_fields: Vec<(InputValueDefinitionId, SchemaInputValueRecord)>,
    /// Object's fields (for JSON)
    #[indexed_by(SchemaInputKeyValueId)]
    key_values: Vec<(StringId, SchemaInputValueRecord)>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct SchemaInputValueId(NonZero<u32>);

impl Readable<Schema> for SchemaInputValueId {
    type Reader<'a> = SchemaInputValue<'a>;

    fn read<'s>(self, schema: &'s Schema) -> Self::Reader<'s>
    where
        Self: 's,
    {
        SchemaInputValue {
            schema,
            value: &schema[self],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct SchemaInputObjectFieldValueId(NonZero<u32>);

impl Readable<Schema> for SchemaInputObjectFieldValueId {
    type Reader<'a> = (InputValueDefinition<'a>, SchemaInputValue<'a>);

    fn read<'s>(self, schema: &'s Schema) -> Self::Reader<'s>
    where
        Self: 's,
    {
        let (input_value_definition, value) = &schema[self];
        (input_value_definition.read(schema), value.read(schema))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct SchemaInputKeyValueId(NonZero<u32>);

impl Readable<Schema> for SchemaInputKeyValueId {
    type Reader<'a> = (&'a str, SchemaInputValue<'a>);

    fn read<'s>(self, schema: &'s Schema) -> Self::Reader<'s>
    where
        Self: 's,
    {
        let (key, value) = &schema[self];
        (key.read(schema), value.read(schema))
    }
}

/// Represents a default input value and @requires arguments.
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum SchemaInputValueRecord {
    Null,
    String(StringId),
    EnumValue(EnumValueId),
    Int(i32),
    BigInt(i64),
    Float(f64),
    Boolean(bool),
    // sorted by input_value_definition_id
    InputObject(IdRange<SchemaInputObjectFieldValueId>),
    List(IdRange<SchemaInputValueId>),

    // for JSON
    // sorted by the key (actual String, not the StringId)
    Map(IdRange<SchemaInputKeyValueId>),
    U64(u64),
}

impl Readable<Schema> for &SchemaInputValueRecord {
    type Reader<'a> = SchemaInputValue<'a> where Self: 'a;
    fn read<'s>(self, schema: &'s Schema) -> Self::Reader<'s>
    where
        Self: 's,
    {
        SchemaInputValue { schema, value: self }
    }
}

impl SchemaInputValueRecord {
    fn discriminant(&self) -> u8 {
        match self {
            SchemaInputValueRecord::Null => 0,
            SchemaInputValueRecord::String(_) => 1,
            SchemaInputValueRecord::EnumValue(_) => 2,
            SchemaInputValueRecord::Int(_) => 3,
            SchemaInputValueRecord::BigInt(_) => 4,
            SchemaInputValueRecord::Float(_) => 5,
            SchemaInputValueRecord::Boolean(_) => 6,
            SchemaInputValueRecord::InputObject(_) => 7,
            SchemaInputValueRecord::List(_) => 8,
            SchemaInputValueRecord::Map(_) => 9,
            SchemaInputValueRecord::U64(_) => 10,
        }
    }
}

impl SchemaInputValues {
    pub(crate) fn push_value(&mut self, value: SchemaInputValueRecord) -> SchemaInputValueId {
        let id = SchemaInputValueId::from(self.values.len());
        self.values.push(value);
        id
    }

    /// Reserve InputValue slots for a list, avoiding the need for an intermediate
    /// Vec to hold values as we need them to be contiguous.
    pub(crate) fn reserve_list(&mut self, n: usize) -> IdRange<SchemaInputValueId> {
        let start = self.values.len();
        self.values.reserve(n);
        for _ in 0..n {
            self.values.push(SchemaInputValueRecord::Null);
        }
        (start..self.values.len()).into()
    }

    /// Reserve InputKeyValue slots for a map, avoiding the need for an intermediate
    /// Vec to hold values as we need them to be contiguous.
    pub(crate) fn reserve_map(&mut self, n: usize) -> IdRange<SchemaInputKeyValueId> {
        let start = self.key_values.len();
        self.key_values.reserve(n);
        for _ in 0..n {
            self.key_values.push((StringId::from(0), SchemaInputValueRecord::Null));
        }
        (start..self.key_values.len()).into()
    }

    pub(crate) fn append_input_object(
        &mut self,
        fields: &mut Vec<(InputValueDefinitionId, SchemaInputValueRecord)>,
    ) -> IdRange<SchemaInputObjectFieldValueId> {
        let start = self.input_fields.len();
        self.input_fields.append(fields);
        (start..self.input_fields.len()).into()
    }
}

#[cfg(test)]
impl SchemaInputValues {
    pub fn push_list(&mut self, values: Vec<SchemaInputValueRecord>) -> IdRange<SchemaInputValueId> {
        let start = self.values.len();
        self.values.extend(values);
        (start..self.values.len()).into()
    }

    pub fn push_map(&mut self, fields: Vec<(StringId, SchemaInputValueRecord)>) -> IdRange<SchemaInputKeyValueId> {
        let start = self.key_values.len();
        self.key_values.extend(fields);
        (start..self.key_values.len()).into()
    }

    pub fn push_input_object(
        &mut self,
        fields: impl IntoIterator<Item = (InputValueDefinitionId, SchemaInputValueRecord)>,
    ) -> IdRange<SchemaInputObjectFieldValueId> {
        let start = self.input_fields.len();
        self.input_fields.extend(fields);
        (start..self.input_fields.len()).into()
    }
}
