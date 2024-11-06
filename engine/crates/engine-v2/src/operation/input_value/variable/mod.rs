mod de;
mod ser;
mod walker;

use ::walker::Walk;
use id_derives::{Id, IndexedFields};
use id_newtypes::IdRange;
use schema::{
    EnumValueId, InputValue, InputValueDefinition, InputValueDefinitionId, SchemaInputValueId, SchemaInputValueRecord,
};

pub(crate) use walker::*;

use crate::operation::PreparedOperationWalker;

use super::InputValueContext;

#[derive(Default, IndexedFields)]
pub(crate) struct VariableInputValues {
    /// Individual input values and list values
    #[indexed_by(VariableInputValueId)]
    values: Vec<VariableInputValueRecord>,

    /// InputObject's fields
    #[indexed_by(VariableInputObjectFieldValueId)]
    input_fields: Vec<(InputValueDefinitionId, VariableInputValueRecord)>,

    /// Object's fields (for JSON)
    #[indexed_by(VariableInputKeyValueId)]
    key_values: Vec<(String, VariableInputValueRecord)>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, Id)]
pub(crate) struct VariableInputValueId(std::num::NonZero<u32>);

impl<'ctx> Walk<InputValueContext<'ctx>> for VariableInputValueId {
    type Walker<'w> = VariableInputValue<'w> where 'ctx: 'w;

    fn walk<'w>(self, ctx: InputValueContext<'ctx>) -> Self::Walker<'w>
    where
        Self: 'w,
        'ctx: 'w,
    {
        VariableInputValue {
            ctx,
            ref_: &ctx.variables[self],
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, Id)]
pub(crate) struct VariableInputObjectFieldValueId(std::num::NonZero<u32>);

impl<'ctx> Walk<InputValueContext<'ctx>> for VariableInputObjectFieldValueId {
    type Walker<'w> = (InputValueDefinition<'w>, VariableInputValue<'w>) where 'ctx: 'w;

    fn walk<'w>(self, ctx: InputValueContext<'ctx>) -> Self::Walker<'w>
    where
        Self: 'w,
        'ctx: 'w,
    {
        let (input_value_definition, value) = &ctx.variables[self];
        (input_value_definition.walk(ctx.schema), value.walk(ctx))
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, Id)]
pub(crate) struct VariableInputKeyValueId(std::num::NonZero<u32>);

impl<'ctx> Walk<InputValueContext<'ctx>> for VariableInputKeyValueId {
    type Walker<'w> = (&'w str, VariableInputValue<'w>) where 'ctx: 'w;

    fn walk<'w>(self, ctx: InputValueContext<'ctx>) -> Self::Walker<'w>
    where
        Self: 'w,
        'ctx: 'w,
    {
        let (key, value) = &ctx.variables[self];
        (key, value.walk(ctx))
    }
}

#[derive(Default)]
pub(crate) enum VariableInputValueRecord {
    #[default]
    Null,
    String(String),
    EnumValue(EnumValueId),
    Int(i32),
    BigInt(i64),
    Float(f64),
    Boolean(bool),
    InputObject(IdRange<VariableInputObjectFieldValueId>),
    List(IdRange<VariableInputValueId>),

    /// for JSON
    Map(IdRange<VariableInputKeyValueId>),
    U64(u64),

    /// Used to reference default values for operation input values. It's tricky without as default
    /// values also need to be taken into account for nested input object fields.
    DefaultValue(SchemaInputValueId),
}

impl<'ctx, 'value> Walk<InputValueContext<'ctx>> for &'value VariableInputValueRecord {
    type Walker<'w> = VariableInputValue<'w> where 'ctx: 'w, 'value: 'w;

    fn walk<'w>(self, ctx: InputValueContext<'ctx>) -> Self::Walker<'w>
    where
        'ctx: 'w,
        'value: 'w,
    {
        VariableInputValue { ctx, ref_: self }
    }
}

impl VariableInputValues {
    pub fn push_value(&mut self, value: VariableInputValueRecord) -> VariableInputValueId {
        let id = VariableInputValueId::from(self.values.len());
        self.values.push(value);
        id
    }

    /// Reserve InputValue slots for a list, avoiding the need for an intermediate
    /// Vec to hold values as we need them to be contiguous.
    pub fn reserve_list(&mut self, n: usize) -> IdRange<VariableInputValueId> {
        let start = self.values.len();
        self.values.reserve(n);
        for _ in 0..n {
            self.values.push(VariableInputValueRecord::Null);
        }
        (start..self.values.len()).into()
    }
    /// Reserve InputKeyValue slots for a map, avoiding the need for an intermediate
    /// Vec to hold values as we need them to be contiguous.
    pub fn reserve_map(&mut self, n: usize) -> IdRange<VariableInputKeyValueId> {
        let start = self.key_values.len();
        self.key_values.reserve(n);
        for _ in 0..n {
            self.key_values.push((String::new(), VariableInputValueRecord::Null));
        }
        (start..self.key_values.len()).into()
    }

    pub fn append_input_object(
        &mut self,
        fields: &mut Vec<(InputValueDefinitionId, VariableInputValueRecord)>,
    ) -> IdRange<VariableInputObjectFieldValueId> {
        let start = self.input_fields.len();
        self.input_fields.append(fields);
        (start..self.input_fields.len()).into()
    }
}

pub type VariableInputValueWalker<'a> = PreparedOperationWalker<'a, &'a VariableInputValueRecord>;

impl<'a> From<VariableInputValueWalker<'a>> for InputValue<'a> {
    fn from(walker: VariableInputValueWalker<'a>) -> Self {
        match walker.item {
            VariableInputValueRecord::Null => InputValue::Null,
            VariableInputValueRecord::String(s) => InputValue::String(s.as_str()),
            VariableInputValueRecord::EnumValue(id) => InputValue::EnumValue(id.walk(walker.schema)),
            VariableInputValueRecord::Int(n) => InputValue::Int(*n),
            VariableInputValueRecord::BigInt(n) => InputValue::BigInt(*n),
            VariableInputValueRecord::Float(f) => InputValue::Float(*f),
            VariableInputValueRecord::Boolean(b) => InputValue::Boolean(*b),
            VariableInputValueRecord::InputObject(ids) => {
                let mut fields = Vec::with_capacity(ids.len());
                for (input_value_definition_id, value) in &walker.variables[*ids] {
                    fields.push((input_value_definition_id.walk(walker.schema), walker.walk(value).into()));
                }
                InputValue::InputObject(fields)
            }
            VariableInputValueRecord::List(ids) => {
                let mut values = Vec::with_capacity(ids.len());
                for id in *ids {
                    values.push(walker.walk(&walker.variables[id]).into());
                }
                InputValue::List(values)
            }
            VariableInputValueRecord::Map(ids) => {
                let mut key_values = Vec::with_capacity(ids.len());
                for (key, value) in &walker.variables[*ids] {
                    key_values.push((key.as_ref(), walker.walk(value).into()));
                }
                InputValue::Map(key_values)
            }
            VariableInputValueRecord::U64(n) => InputValue::U64(*n),
            VariableInputValueRecord::DefaultValue(id) => id.walk(walker.schema).into(),
        }
    }
}

impl PartialEq<SchemaInputValueRecord> for VariableInputValueWalker<'_> {
    fn eq(&self, other: &SchemaInputValueRecord) -> bool {
        match (self.item, other) {
            (VariableInputValueRecord::Null, SchemaInputValueRecord::Null) => true,
            (VariableInputValueRecord::String(l), SchemaInputValueRecord::String(r)) => l == &self.schema[*r],
            (VariableInputValueRecord::EnumValue(l), SchemaInputValueRecord::EnumValue(r)) => l == r,
            (VariableInputValueRecord::Int(l), SchemaInputValueRecord::Int(r)) => l == r,
            (VariableInputValueRecord::BigInt(l), SchemaInputValueRecord::BigInt(r)) => l == r,
            (VariableInputValueRecord::U64(l), SchemaInputValueRecord::U64(r)) => l == r,
            (VariableInputValueRecord::Float(l), SchemaInputValueRecord::Float(r)) => l == r,
            (VariableInputValueRecord::Boolean(l), SchemaInputValueRecord::Boolean(r)) => l == r,
            (VariableInputValueRecord::InputObject(lids), SchemaInputValueRecord::InputObject(rids)) => {
                let op_input_values = &self.variables[*lids];
                let schema_input_values = &self.schema[*rids];

                if op_input_values.len() < schema_input_values.len() {
                    return false;
                }

                for (id, input_value) in op_input_values {
                    let input_value = self.walk(input_value);
                    if let Ok(i) = schema_input_values.binary_search_by(|probe| probe.0.cmp(id)) {
                        if !input_value.eq(&schema_input_values[i].1) {
                            return false;
                        }
                    } else {
                        return false;
                    };
                }

                true
            }
            (VariableInputValueRecord::List(lids), SchemaInputValueRecord::List(rids)) => {
                let left = &self.variables[*lids];
                let right = &self.schema[*rids];
                if left.len() != right.len() {
                    return false;
                }
                for (left_value, right_value) in left.iter().zip(right) {
                    if !self.walk(left_value).eq(right_value) {
                        return false;
                    }
                }
                true
            }
            (VariableInputValueRecord::Map(ids), SchemaInputValueRecord::Map(other_ids)) => {
                let op_kv = &self.variables[*ids];
                let schema_kv = &self.schema[*other_ids];

                for (key, value) in op_kv {
                    let value = self.walk(value);
                    if let Ok(i) = schema_kv.binary_search_by(|probe| self.schema[probe.0].cmp(key)) {
                        if !value.eq(&schema_kv[i].1) {
                            return false;
                        }
                    } else {
                        return false;
                    };
                }

                true
            }
            (VariableInputValueRecord::DefaultValue(id), value) => id.walk(self.schema).eq(&value.walk(self.schema)),
            // A bit tedious, but avoids missing a case
            (VariableInputValueRecord::Null, _) => false,
            (VariableInputValueRecord::String(_), _) => false,
            (VariableInputValueRecord::EnumValue(_), _) => false,
            (VariableInputValueRecord::Int(_), _) => false,
            (VariableInputValueRecord::BigInt(_), _) => false,
            (VariableInputValueRecord::U64(_), _) => false,
            (VariableInputValueRecord::Float(_), _) => false,
            (VariableInputValueRecord::Boolean(_), _) => false,
            (VariableInputValueRecord::InputObject(_), _) => false,
            (VariableInputValueRecord::List(_), _) => false,
            (VariableInputValueRecord::Map(_), _) => false,
        }
    }
}

impl std::fmt::Debug for VariableInputValueWalker<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.item {
            VariableInputValueRecord::Null => write!(f, "Null"),
            VariableInputValueRecord::String(s) => s.fmt(f),
            VariableInputValueRecord::EnumValue(id) => {
                f.debug_tuple("EnumValue").field(&self.schema.walk(*id).name()).finish()
            }
            VariableInputValueRecord::Int(n) => f.debug_tuple("Int").field(n).finish(),
            VariableInputValueRecord::BigInt(n) => f.debug_tuple("BigInt").field(n).finish(),
            VariableInputValueRecord::U64(n) => f.debug_tuple("U64").field(n).finish(),
            VariableInputValueRecord::Float(n) => f.debug_tuple("Float").field(n).finish(),
            VariableInputValueRecord::Boolean(b) => b.fmt(f),
            VariableInputValueRecord::InputObject(ids) => {
                let mut map = f.debug_struct("InputObject");
                for (input_value_definition_id, value) in &self.variables[*ids] {
                    map.field(self.schema.walk(*input_value_definition_id).name(), &self.walk(value));
                }
                map.finish()
            }
            VariableInputValueRecord::List(ids) => {
                let mut seq = f.debug_list();
                for value in &self.variables[*ids] {
                    seq.entry(&self.walk(value));
                }
                seq.finish()
            }
            VariableInputValueRecord::Map(ids) => {
                let mut map = f.debug_map();
                for (key, value) in &self.variables[*ids] {
                    map.entry(&key, &self.walk(value));
                }
                map.finish()
            }
            VariableInputValueRecord::DefaultValue(id) => {
                f.debug_tuple("DefaultValue").field(&id.walk(self.schema)).finish()
            }
        }
    }
}
