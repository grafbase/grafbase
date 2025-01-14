mod eq;
mod view;
mod walker;

use ::walker::Walk;
use id_derives::{Id, IndexedFields};
use id_newtypes::IdRange;
use schema::{EnumValueId, InputValueDefinition, InputValueDefinitionId, SchemaInputValueId};

use crate::VariableDefinitionId;

pub use eq::*;
pub use view::*;
pub use walker::*;

use super::InputValueContext;

#[derive(Default, Clone, serde::Serialize, serde::Deserialize, IndexedFields)]
pub struct QueryInputValues {
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
pub struct QueryInputValueId(std::num::NonZero<u32>);

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
pub struct QueryInputObjectFieldValueId(std::num::NonZero<u32>);

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
pub struct QueryInputKeyValueId(std::num::NonZero<u32>);

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
pub enum QueryInputValueRecord {
    #[default]
    Null,
    String(String),
    EnumValue(EnumValueId),
    Int(i32),
    BigInt(i64),
    Float(f64),
    Boolean(bool),
    // Sorted by the input value definition id
    InputObject(IdRange<QueryInputObjectFieldValueId>),
    List(IdRange<QueryInputValueId>),

    /// for JSON
    Map(IdRange<QueryInputKeyValueId>),
    U64(u64),

    /// We may encounter unbound enum values within a scalar for which we have no definition. In
    /// this case we keep track of it.
    UnboundEnumValue(String),

    DefaultValue(SchemaInputValueId),
    Variable(VariableDefinitionId),
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
