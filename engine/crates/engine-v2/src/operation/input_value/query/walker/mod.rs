use schema::Schema;
use walker::Walk;

use crate::{operation::Variables, plan::PlanContext};

use super::{QueryInputValueId, QueryInputValueRecord, QueryInputValues};

impl<'a> Walk<PlanContext<'a>> for QueryInputValueId {
    type Walker<'w> = QueryInputValue<'w> where 'a: 'w;

    fn walk<'w>(self, ctx: PlanContext<'a>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        QueryInputValue {
            schema: ctx.schema,
            input_values: &ctx.operation_plan.query_input_values,
            variables: ctx.variables,
            value: &ctx.operation_plan.query_input_values[self],
        }
    }
}

impl<'a, 'b> Walk<PlanContext<'a>> for &'b QueryInputValueRecord {
    type Walker<'w> = QueryInputValue<'w> where 'a: 'w, 'b: 'w;

    fn walk<'w>(self, ctx: PlanContext<'a>) -> Self::Walker<'w>
    where
        'a: 'w,
        'b: 'w,
    {
        QueryInputValue {
            schema: ctx.schema,
            input_values: &ctx.operation_plan.query_input_values,
            variables: ctx.variables,
            value: self,
        }
    }
}

pub(crate) struct QueryInputValue<'a> {
    input_values: &'a QueryInputValues,
    schema: &'a Schema,
    variables: &'a Variables,
    value: &'a QueryInputValueRecord,
}

impl<'a> QueryInputValue<'a> {
    pub fn walk(&self, value: &'a QueryInputValueRecord) -> Self {
        QueryInputValue {
            schema: self.schema,
            input_values: self.input_values,
            variables: self.variables,
            value,
        }
    }
}

impl std::fmt::Debug for QueryInputValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value {
            QueryInputValueRecord::Null => write!(f, "Null"),
            QueryInputValueRecord::String(s) => s.fmt(f),
            QueryInputValueRecord::EnumValue(id) => {
                f.debug_tuple("EnumValue").field(&self.schema.walk(*id).name()).finish()
            }
            QueryInputValueRecord::UnboundEnumValue(s) => f.debug_tuple("UnboundEnumValue").field(&s).finish(),
            QueryInputValueRecord::Int(n) => f.debug_tuple("Int").field(n).finish(),
            QueryInputValueRecord::BigInt(n) => f.debug_tuple("BigInt").field(n).finish(),
            QueryInputValueRecord::U64(n) => f.debug_tuple("U64").field(n).finish(),
            QueryInputValueRecord::Float(n) => f.debug_tuple("Float").field(n).finish(),
            QueryInputValueRecord::Boolean(b) => b.fmt(f),
            QueryInputValueRecord::InputObject(ids) => {
                let mut map = f.debug_struct("InputObject");
                for (input_value_definition_id, value) in &self.input_values[*ids] {
                    map.field(self.schema.walk(*input_value_definition_id).name(), &self.walk(value));
                }
                map.finish()
            }
            QueryInputValueRecord::List(ids) => {
                let mut seq = f.debug_list();
                for value in &self.input_values[*ids] {
                    seq.entry(&self.walk(value));
                }
                seq.finish()
            }
            QueryInputValueRecord::Map(ids) => {
                let mut map = f.debug_map();
                for (key, value) in &self.input_values[*ids] {
                    map.entry(&key, &self.walk(value));
                }
                map.finish()
            }
            QueryInputValueRecord::DefaultValue(id) => {
                f.debug_tuple("DefaultValue").field(&id.walk(self.schema)).finish()
            }
            // TODO: Add missing variable debug impl
            QueryInputValueRecord::Variable(_id) => f.debug_struct("Variable").finish_non_exhaustive(),
        }
    }
}
