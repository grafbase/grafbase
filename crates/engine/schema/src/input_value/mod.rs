use std::num::NonZero;

use crate::{
    EnumValueId, IdRange, InputValueDefinition, InputValueDefinitionId, Schema, StringId, TypeSystemDirective,
};

mod error;
mod set;
#[cfg(test)]
mod tests;
mod value;
mod walker;

use ::walker::Walk;
pub use error::*;
pub use set::*;
pub use value::*;
pub use walker::*;

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

impl<'s> Walk<&'s Schema> for SchemaInputValueId {
    type Walker<'w> = SchemaInputValue<'w> where 's: 'w;

    fn walk<'w>(self, schema: impl Into<&'s Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        's: 'w,
    {
        let schema = schema.into();
        SchemaInputValue {
            schema,
            ref_: &schema[self],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct SchemaInputObjectFieldValueId(NonZero<u32>);

impl<'s> Walk<&'s Schema> for SchemaInputObjectFieldValueId {
    type Walker<'w> = (InputValueDefinition<'w>, SchemaInputValue<'w>) where 's: 'w;

    fn walk<'w>(self, schema: impl Into<&'s Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        's: 'w,
    {
        let schema = schema.into();
        let (input_value_definition, value) = &schema[self];
        (input_value_definition.walk(schema), value.walk(schema))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct SchemaInputKeyValueId(NonZero<u32>);

impl<'s> Walk<&'s Schema> for SchemaInputKeyValueId {
    type Walker<'w> = (&'w str, SchemaInputValue<'w>) where 's: 'w;

    fn walk<'w>(self, schema: impl Into<&'s Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        's: 'w,
    {
        let schema = schema.into();
        let (key, value) = &schema[self];
        (key.walk(schema), value.walk(schema))
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

    /// We may encounter unbound enum values within a scalar for which we have no definition. In
    /// this case we keep track of it.
    UnboundEnumValue(StringId),
}

impl<'s> Walk<&'s Schema> for &SchemaInputValueRecord {
    type Walker<'w> = SchemaInputValue<'w> where Self: 'w, 's: 'w;
    fn walk<'w>(self, schema: impl Into<&'s Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        's: 'w,
    {
        SchemaInputValue {
            schema: schema.into(),
            ref_: self,
        }
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
            SchemaInputValueRecord::UnboundEnumValue(_) => 11,
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
            self.key_values
                .push((StringId::from(0usize), SchemaInputValueRecord::Null));
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

impl<'a> InputValueDefinition<'a> {
    pub fn cost(&self) -> Option<i32> {
        self.directives().find_map(|directive| match directive {
            TypeSystemDirective::Cost(cost) => Some(cost.weight),
            _ => None,
        })
    }
}
