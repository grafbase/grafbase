mod de;
mod debug;
mod ser;

use schema::{InputValueSet, SchemaInputValue, SchemaInputValueRecord};
use walker::Walk;

use crate::operation::InputValueContext;

use super::{QueryInputValueRecord, QueryInputValueView};

#[derive(Clone, Copy)]
pub(crate) struct QueryInputValue<'a> {
    pub(super) ctx: InputValueContext<'a>,
    pub(super) ref_: &'a QueryInputValueRecord,
}

impl<'a> QueryInputValue<'a> {
    /// Used for GraphQL query generation to only include values in the query string that would be
    /// present after query normalization.
    #[allow(unused)]
    pub fn to_normalized_query_const_value_str(self) -> Option<&'a str> {
        Some(match self.ref_ {
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

    pub fn as_usize(&self) -> Option<usize> {
        match self.ref_ {
            QueryInputValueRecord::Int(value) => Some(*value as usize),
            QueryInputValueRecord::BigInt(value) => Some(*value as usize),
            QueryInputValueRecord::DefaultValue(id) => self.ctx.schema.walk(*id).as_usize(),
            QueryInputValueRecord::Variable(id) => id.walk(self.ctx).as_usize(),
            _ => None,
        }
    }

    pub fn is_undefined(&self) -> bool {
        match self.ref_ {
            QueryInputValueRecord::Variable(id) => id.walk(self.ctx).is_undefined(),
            _ => false,
        }
    }

    pub fn with_selection_set<'s, 'w>(self, selection_set: &'s InputValueSet) -> QueryInputValueView<'w>
    where
        'a: 'w,
        's: 'w,
    {
        QueryInputValueView {
            value: self,
            selection_set,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum QueryOrSchemaInputValue<'a> {
    Query(QueryInputValue<'a>),
    Schema(SchemaInputValue<'a>),
}
