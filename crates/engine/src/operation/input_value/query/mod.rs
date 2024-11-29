mod view;
mod walker;

use ::walker::Walk;
use id_derives::{Id, IndexedFields};
use id_newtypes::IdRange;
use schema::{EnumValueId, InputValueDefinition, InputValueDefinitionId, SchemaInputValueId, SchemaInputValueRecord};

use crate::operation::{BoundVariableDefinitionId, OperationWalker};

pub(crate) use view::*;
pub(crate) use walker::*;

use super::InputValueContext;

#[derive(Default, Clone, serde::Serialize, serde::Deserialize, IndexedFields)]
pub(crate) struct QueryInputValues {
    /// Individual input values and list values
    #[indexed_by(QueryInputValueId)]
    values: Vec<QueryInputValueRecord>,

    /// InputObject's fields
    #[indexed_by(QueryInputObjectFieldValueId)]
    input_fields: Vec<(InputValueDefinitionId, QueryInputValueRecord)>,

    /// Object's fields (for JSON)
    #[indexed_by(QueryInputKeyValueId)]
    key_values: Vec<(String, QueryInputValueRecord)>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, Id)]
pub(crate) struct QueryInputValueId(std::num::NonZero<u32>);

impl<'ctx> Walk<InputValueContext<'ctx>> for QueryInputValueId {
    type Walker<'w>
        = QueryInputValue<'w>
    where
        'ctx: 'w;

    fn walk<'w>(self, ctx: impl Into<InputValueContext<'ctx>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'ctx: 'w,
    {
        let ctx: InputValueContext<'ctx> = ctx.into();
        QueryInputValue {
            ctx,
            ref_: &ctx.query_input_values[self],
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, Id)]
pub(crate) struct QueryInputObjectFieldValueId(std::num::NonZero<u32>);

impl<'ctx> Walk<InputValueContext<'ctx>> for QueryInputObjectFieldValueId {
    type Walker<'w>
        = (InputValueDefinition<'w>, QueryInputValue<'w>)
    where
        'ctx: 'w;

    fn walk<'w>(self, ctx: impl Into<InputValueContext<'ctx>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'ctx: 'w,
    {
        let ctx: InputValueContext<'ctx> = ctx.into();
        let (input_value_definition, value) = &ctx.query_input_values[self];
        (input_value_definition.walk(ctx.schema), value.walk(ctx))
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, Id)]
pub(crate) struct QueryInputKeyValueId(std::num::NonZero<u32>);

impl<'ctx> Walk<InputValueContext<'ctx>> for QueryInputKeyValueId {
    type Walker<'w>
        = (&'w str, QueryInputValue<'w>)
    where
        'ctx: 'w;

    fn walk<'w>(self, ctx: impl Into<InputValueContext<'ctx>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'ctx: 'w,
    {
        let ctx: InputValueContext<'ctx> = ctx.into();
        let (key, value) = &ctx.query_input_values[self];
        (key, value.walk(ctx))
    }
}

#[derive(Default, Clone, serde::Serialize, serde::Deserialize, Debug)]
pub(crate) enum QueryInputValueRecord {
    #[default]
    Null,
    String(String),
    EnumValue(EnumValueId),
    Int(i32),
    BigInt(i64),
    Float(f64),
    Boolean(bool),
    InputObject(IdRange<QueryInputObjectFieldValueId>),
    List(IdRange<QueryInputValueId>),

    /// for JSON
    Map(IdRange<QueryInputKeyValueId>),
    U64(u64),

    /// We may encounter unbound enum values within a scalar for which we have no definition. In
    /// this case we keep track of it.
    UnboundEnumValue(String),

    DefaultValue(SchemaInputValueId),
    Variable(BoundVariableDefinitionId),
}

impl<'ctx, 'value> Walk<InputValueContext<'ctx>> for &'value QueryInputValueRecord {
    type Walker<'w>
        = QueryInputValue<'w>
    where
        'ctx: 'w,
        'value: 'w;

    fn walk<'w>(self, ctx: impl Into<InputValueContext<'ctx>>) -> Self::Walker<'w>
    where
        'ctx: 'w,
        'value: 'w,
    {
        let ctx = ctx.into();
        QueryInputValue { ctx, ref_: self }
    }
}

impl QueryInputValues {
    pub fn push_value(&mut self, value: QueryInputValueRecord) -> QueryInputValueId {
        let id = QueryInputValueId::from(self.values.len());
        self.values.push(value);
        id
    }

    /// Reserve InputValue slots for a list, avoiding the need for an intermediate
    /// Vec to hold values as we need them to be contiguous.
    pub fn reserve_list(&mut self, n: usize) -> IdRange<QueryInputValueId> {
        let start = self.values.len();
        self.values.reserve(n);
        for _ in 0..n {
            self.values.push(QueryInputValueRecord::Null);
        }
        (start..self.values.len()).into()
    }
    /// Reserve InputKeyValue slots for a map, avoiding the need for an intermediate
    /// Vec to hold values as we need them to be contiguous.
    pub fn reserve_map(&mut self, n: usize) -> IdRange<QueryInputKeyValueId> {
        let start = self.key_values.len();
        self.key_values.reserve(n);
        for _ in 0..n {
            self.key_values.push((String::new(), QueryInputValueRecord::Null));
        }
        (start..self.key_values.len()).into()
    }

    pub fn append_input_object(
        &mut self,
        fields: &mut Vec<(InputValueDefinitionId, QueryInputValueRecord)>,
    ) -> IdRange<QueryInputObjectFieldValueId> {
        let start = self.input_fields.len();
        self.input_fields.append(fields);
        (start..self.input_fields.len()).into()
    }
}

impl PartialEq<SchemaInputValueRecord> for OperationWalker<'_, &QueryInputValueRecord> {
    fn eq(&self, other: &SchemaInputValueRecord) -> bool {
        let input_values = &self.operation.query_input_values;
        match (self.item, other) {
            (QueryInputValueRecord::Null, SchemaInputValueRecord::Null) => true,
            (QueryInputValueRecord::String(l), SchemaInputValueRecord::String(r)) => l == &self.schema[*r],
            (QueryInputValueRecord::EnumValue(l), SchemaInputValueRecord::EnumValue(r)) => l == r,
            (QueryInputValueRecord::UnboundEnumValue(l), SchemaInputValueRecord::UnboundEnumValue(r)) => {
                l == &self.schema[*r]
            }
            (QueryInputValueRecord::Int(l), SchemaInputValueRecord::Int(r)) => l == r,
            (QueryInputValueRecord::BigInt(l), SchemaInputValueRecord::BigInt(r)) => l == r,
            (QueryInputValueRecord::U64(l), SchemaInputValueRecord::U64(r)) => l == r,
            (QueryInputValueRecord::Float(l), SchemaInputValueRecord::Float(r)) => l == r,
            (QueryInputValueRecord::Boolean(l), SchemaInputValueRecord::Boolean(r)) => l == r,
            (QueryInputValueRecord::InputObject(lids), SchemaInputValueRecord::InputObject(rids)) => {
                let op_input_values = &input_values[*lids];
                let schema_input_values = &self.schema[*rids];

                if op_input_values.len() != schema_input_values.len() {
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
            (QueryInputValueRecord::List(lids), SchemaInputValueRecord::List(rids)) => {
                let left = &input_values[*lids];
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
            (QueryInputValueRecord::Map(ids), SchemaInputValueRecord::Map(other_ids)) => {
                let op_kv = &input_values[*ids];
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
            (QueryInputValueRecord::DefaultValue(id), value) => id.walk(self.schema).eq(&value.walk(self.schema)),
            (QueryInputValueRecord::Variable(_), _) => false,
            // A bit tedious, but avoids missing a case
            (QueryInputValueRecord::Null, _) => false,
            (QueryInputValueRecord::String(_), _) => false,
            (QueryInputValueRecord::EnumValue(_), _) => false,
            (QueryInputValueRecord::UnboundEnumValue(_), _) => false,
            (QueryInputValueRecord::Int(_), _) => false,
            (QueryInputValueRecord::BigInt(_), _) => false,
            (QueryInputValueRecord::U64(_), _) => false,
            (QueryInputValueRecord::Float(_), _) => false,
            (QueryInputValueRecord::Boolean(_), _) => false,
            (QueryInputValueRecord::InputObject(_), _) => false,
            (QueryInputValueRecord::List(_), _) => false,
            (QueryInputValueRecord::Map(_), _) => false,
        }
    }
}
