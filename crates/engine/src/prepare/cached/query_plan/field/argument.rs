use id_newtypes::IdRange;
use operation::{
    InputValueContext, QueryInputValueRecord, QueryOrSchemaInputValue, QueryOrSchemaInputValueId, Variables,
};
use schema::{ArgumentValueInjection, SchemaInputValueRecord};
use walker::Walk;

use crate::prepare::{CachedOperationContext, PartitionFieldArgument};

use super::PlanFieldArguments;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub(crate) enum PlanValueRecord {
    Value(QueryOrSchemaInputValueId),
    Injection(ArgumentValueInjection),
}

impl PlanValueRecord {
    pub fn as_schema_or_query_input_value(self) -> Option<QueryOrSchemaInputValueId> {
        match self {
            PlanValueRecord::Value(id) => Some(id),
            PlanValueRecord::Injection(_) => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, id_derives::Id, serde::Serialize, serde::Deserialize)]
pub struct PartitionFieldArgumentId(u16);

impl<'a> Walk<CachedOperationContext<'a>> for IdRange<PartitionFieldArgumentId> {
    type Walker<'w>
        = PlanFieldArguments<'w>
    where
        Self: 'w,
        'a: 'w;

    fn walk<'w>(self, ctx: impl Into<CachedOperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        let ctx: CachedOperationContext<'a> = ctx.into();
        PlanFieldArguments {
            ctx,
            records: &ctx.cached.query_plan[self],
        }
    }
}

impl<'ctx> PartitionFieldArgument<'ctx> {
    /// Used for GraphQL query generation to only include values in the query string that would be
    /// present after query sanitization.
    pub(crate) fn value_as_sanitized_query_const_value_str(&self) -> Option<&'ctx str> {
        match self.value_record {
            PlanValueRecord::Value(QueryOrSchemaInputValueId::Query(id)) => {
                Some(match &self.ctx.cached.operation.query_input_values[id] {
                    QueryInputValueRecord::EnumValue(id) => self.ctx.schema.walk(*id).name(),
                    QueryInputValueRecord::Boolean(b) => {
                        if *b {
                            "true"
                        } else {
                            "false"
                        }
                    }
                    QueryInputValueRecord::DefaultValue(id) => match &self.ctx.schema[*id] {
                        SchemaInputValueRecord::EnumValue(id) => self.ctx.schema.walk(*id).name(),
                        SchemaInputValueRecord::Boolean(b) => {
                            if *b {
                                "true"
                            } else {
                                "false"
                            }
                        }
                        _ => return None,
                    },
                    _ => return None,
                })
            }
            PlanValueRecord::Value(QueryOrSchemaInputValueId::Schema(id)) => Some(match &self.ctx.schema[id] {
                SchemaInputValueRecord::EnumValue(id) => self.ctx.schema.walk(*id).name(),
                SchemaInputValueRecord::Boolean(b) => {
                    if *b {
                        "true"
                    } else {
                        "false"
                    }
                }
                _ => return None,
            }),
            PlanValueRecord::Injection(_) => None,
        }
    }

    pub(crate) fn value<'v, 'out>(&self, variables: &'v Variables) -> Option<QueryOrSchemaInputValue<'out>>
    where
        'v: 'out,
        'ctx: 'out,
    {
        match self.value_record {
            PlanValueRecord::Value(QueryOrSchemaInputValueId::Query(id)) => {
                let ctx = InputValueContext {
                    schema: self.ctx.schema,
                    query_input_values: &self.ctx.cached.operation.query_input_values,
                    variables,
                };
                let value = id.walk(ctx);
                if value.is_undefined() {
                    self.definition().default_value().map(QueryOrSchemaInputValue::Schema)
                } else {
                    Some(QueryOrSchemaInputValue::Query(value))
                }
            }
            PlanValueRecord::Value(QueryOrSchemaInputValueId::Schema(id)) => {
                Some(QueryOrSchemaInputValue::Schema(id.walk(self.ctx.schema)))
            }
            PlanValueRecord::Injection(_) => None,
        }
    }
}
