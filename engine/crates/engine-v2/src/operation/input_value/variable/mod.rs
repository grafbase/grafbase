mod de;
mod ser;

use id_derives::{Id, IndexImpls};
use id_newtypes::IdRange;
use schema::{EnumValueId, InputValue, InputValueDefinitionId, SchemaInputValue, SchemaInputValueId};

use crate::operation::OperationWalker;

#[derive(Default, IndexImpls)]
pub struct VariableInputValues {
    /// Individual input values and list values
    #[indexed_by(VariableInputValueId)]
    values: Vec<VariableInputValue>,

    /// InputObject's fields
    #[indexed_by(VariableInputObjectFieldValueId)]
    input_fields: Vec<(InputValueDefinitionId, VariableInputValue)>,

    /// Object's fields (for JSON)
    #[indexed_by(VariableInputKeyValueId)]
    key_values: Vec<(String, VariableInputValue)>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, Id)]
pub struct VariableInputValueId(std::num::NonZero<u32>);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, Id)]
pub struct VariableInputObjectFieldValueId(std::num::NonZero<u32>);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, Id)]
pub struct VariableInputKeyValueId(std::num::NonZero<u32>);

#[derive(Default)]
pub enum VariableInputValue {
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

impl VariableInputValues {
    pub fn push_value(&mut self, value: VariableInputValue) -> VariableInputValueId {
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
            self.values.push(VariableInputValue::Null);
        }
        (start..self.values.len()).into()
    }
    /// Reserve InputKeyValue slots for a map, avoiding the need for an intermediate
    /// Vec to hold values as we need them to be contiguous.
    pub fn reserve_map(&mut self, n: usize) -> IdRange<VariableInputKeyValueId> {
        let start = self.key_values.len();
        self.key_values.reserve(n);
        for _ in 0..n {
            self.key_values.push((String::new(), VariableInputValue::Null));
        }
        (start..self.key_values.len()).into()
    }

    pub fn append_input_object(
        &mut self,
        fields: &mut Vec<(InputValueDefinitionId, VariableInputValue)>,
    ) -> IdRange<VariableInputObjectFieldValueId> {
        let start = self.input_fields.len();
        self.input_fields.append(fields);
        (start..self.input_fields.len()).into()
    }
}

pub type VariableInputValueWalker<'a> = OperationWalker<'a, &'a VariableInputValue, ()>;

impl<'a> From<VariableInputValueWalker<'a>> for InputValue<'a> {
    fn from(walker: VariableInputValueWalker<'a>) -> Self {
        match walker.item {
            VariableInputValue::Null => InputValue::Null,
            VariableInputValue::String(s) => InputValue::String(s.as_str()),
            VariableInputValue::EnumValue(id) => InputValue::EnumValue(*id),
            VariableInputValue::Int(n) => InputValue::Int(*n),
            VariableInputValue::BigInt(n) => InputValue::BigInt(*n),
            VariableInputValue::Float(f) => InputValue::Float(*f),
            VariableInputValue::Boolean(b) => InputValue::Boolean(*b),
            VariableInputValue::InputObject(ids) => {
                let mut fields = Vec::with_capacity(ids.len());
                for (input_value_definition_id, value) in &walker.variables[*ids] {
                    fields.push((*input_value_definition_id, walker.walk(value).into()));
                }
                InputValue::InputObject(fields.into_boxed_slice())
            }
            VariableInputValue::List(ids) => {
                let mut values = Vec::with_capacity(ids.len());
                for id in *ids {
                    values.push(walker.walk(&walker.variables[id]).into());
                }
                InputValue::List(values.into_boxed_slice())
            }
            VariableInputValue::Map(ids) => {
                let mut key_values = Vec::with_capacity(ids.len());
                for (key, value) in &walker.variables[*ids] {
                    key_values.push((key.as_ref(), walker.walk(value).into()));
                }
                InputValue::Map(key_values.into_boxed_slice())
            }
            VariableInputValue::U64(n) => InputValue::U64(*n),
            VariableInputValue::DefaultValue(id) => {
                let value: &'a SchemaInputValue = &walker.schema_walker.as_ref()[*id];
                walker.schema_walker.walk(value).into()
            }
        }
    }
}

impl PartialEq<SchemaInputValue> for VariableInputValueWalker<'_> {
    fn eq(&self, other: &SchemaInputValue) -> bool {
        match (self.item, other) {
            (VariableInputValue::Null, SchemaInputValue::Null) => true,
            (VariableInputValue::String(l), SchemaInputValue::String(r)) => l == &self.schema_walker[*r],
            (VariableInputValue::EnumValue(l), SchemaInputValue::EnumValue(r)) => l == r,
            (VariableInputValue::Int(l), SchemaInputValue::Int(r)) => l == r,
            (VariableInputValue::BigInt(l), SchemaInputValue::BigInt(r)) => l == r,
            (VariableInputValue::U64(l), SchemaInputValue::U64(r)) => l == r,
            (VariableInputValue::Float(l), SchemaInputValue::Float(r)) => l == r,
            (VariableInputValue::Boolean(l), SchemaInputValue::Boolean(r)) => l == r,
            (VariableInputValue::InputObject(lids), SchemaInputValue::InputObject(rids)) => {
                let op_input_values = &self.variables[*lids];
                let schema_input_values = &self.schema_walker.as_ref()[*rids];

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
            (VariableInputValue::List(lids), SchemaInputValue::List(rids)) => {
                let left = &self.variables[*lids];
                let right = &self.schema_walker.as_ref()[*rids];
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
            (VariableInputValue::Map(ids), SchemaInputValue::Map(other_ids)) => {
                let op_kv = &self.variables[*ids];
                let schema_kv = &self.schema_walker[*other_ids];

                for (key, value) in op_kv {
                    let value = self.walk(value);
                    if let Ok(i) = schema_kv.binary_search_by(|probe| self.schema_walker[probe.0].cmp(key)) {
                        if !value.eq(&schema_kv[i].1) {
                            return false;
                        }
                    } else {
                        return false;
                    };
                }

                true
            }
            (VariableInputValue::DefaultValue(id), value) => self
                .schema_walker
                .walk(&self.schema_walker.as_ref()[*id])
                .eq(&self.schema_walker.walk(value)),
            // A bit tedious, but avoids missing a case
            (VariableInputValue::Null, _) => false,
            (VariableInputValue::String(_), _) => false,
            (VariableInputValue::EnumValue(_), _) => false,
            (VariableInputValue::Int(_), _) => false,
            (VariableInputValue::BigInt(_), _) => false,
            (VariableInputValue::U64(_), _) => false,
            (VariableInputValue::Float(_), _) => false,
            (VariableInputValue::Boolean(_), _) => false,
            (VariableInputValue::InputObject(_), _) => false,
            (VariableInputValue::List(_), _) => false,
            (VariableInputValue::Map(_), _) => false,
        }
    }
}

impl std::fmt::Debug for VariableInputValueWalker<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.item {
            VariableInputValue::Null => write!(f, "Null"),
            VariableInputValue::String(s) => s.fmt(f),
            VariableInputValue::EnumValue(id) => f
                .debug_tuple("EnumValue")
                .field(&self.schema_walker.walk(*id).name())
                .finish(),
            VariableInputValue::Int(n) => f.debug_tuple("Int").field(n).finish(),
            VariableInputValue::BigInt(n) => f.debug_tuple("BigInt").field(n).finish(),
            VariableInputValue::U64(n) => f.debug_tuple("U64").field(n).finish(),
            VariableInputValue::Float(n) => f.debug_tuple("Float").field(n).finish(),
            VariableInputValue::Boolean(b) => b.fmt(f),
            VariableInputValue::InputObject(ids) => {
                let mut map = f.debug_struct("InputObject");
                for (input_value_definition_id, value) in &self.variables[*ids] {
                    map.field(
                        self.schema_walker.walk(*input_value_definition_id).name(),
                        &self.walk(value),
                    );
                }
                map.finish()
            }
            VariableInputValue::List(ids) => {
                let mut seq = f.debug_list();
                for value in &self.variables[*ids] {
                    seq.entry(&self.walk(value));
                }
                seq.finish()
            }
            VariableInputValue::Map(ids) => {
                let mut map = f.debug_map();
                for (key, value) in &self.variables[*ids] {
                    map.entry(&key, &self.walk(value));
                }
                map.finish()
            }
            VariableInputValue::DefaultValue(id) => f
                .debug_tuple("DefaultValue")
                .field(&self.schema_walker.walk(&self.schema_walker.as_ref()[*id]))
                .finish(),
        }
    }
}
