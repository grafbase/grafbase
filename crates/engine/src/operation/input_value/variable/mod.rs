mod walker;

use ::walker::Walk;
use id_derives::{Id, IndexedFields};
use id_newtypes::IdRange;
use schema::{EnumValueId, InputValueDefinition, InputValueDefinitionId, SchemaInputValueId};

pub(crate) use walker::*;

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

    fn walk<'w>(self, ctx: impl Into<InputValueContext<'ctx>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'ctx: 'w,
    {
        let ctx = ctx.into();
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

    fn walk<'w>(self, ctx: impl Into<InputValueContext<'ctx>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'ctx: 'w,
    {
        let ctx: InputValueContext<'ctx> = ctx.into();
        let (input_value_definition, value) = &ctx.variables[self];
        (input_value_definition.walk(ctx.schema), value.walk(ctx))
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, Id)]
pub(crate) struct VariableInputKeyValueId(std::num::NonZero<u32>);

impl<'ctx> Walk<InputValueContext<'ctx>> for VariableInputKeyValueId {
    type Walker<'w> = (&'w str, VariableInputValue<'w>) where 'ctx: 'w;

    fn walk<'w>(self, ctx: impl Into<InputValueContext<'ctx>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'ctx: 'w,
    {
        let ctx: InputValueContext<'ctx> = ctx.into();
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

    fn walk<'w>(self, ctx: impl Into<InputValueContext<'ctx>>) -> Self::Walker<'w>
    where
        'ctx: 'w,
        'value: 'w,
    {
        let ctx = ctx.into();
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
