mod de;
mod debug;
mod ser;

use schema::{InputValueSet, SchemaInputValue, SchemaInputValueId, SchemaInputValueRecord};
use walker::Walk;

use crate::{InputValueContext, VariableDefinitionId};

use super::{QueryInputValueId, QueryInputValueRecord, QueryInputValueView, QueryOrSchemaInputValueView};

#[derive(Clone, Copy)]
pub struct QueryInputValue<'a> {
    pub(super) ctx: InputValueContext<'a>,
    pub(super) ref_: &'a QueryInputValueRecord,
}

impl<'a> QueryInputValue<'a> {
    /// Used for GraphQL query generation to only include values in the query string that would be
    /// present after query sanitization.
    pub fn to_sanitized_query_const_value_str(self) -> Option<&'a str> {
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

    pub fn is_undefined(&self) -> bool {
        match self.ref_ {
            QueryInputValueRecord::Variable(id) => {
                <VariableDefinitionId as Walk<InputValueContext<'a>>>::walk(*id, self.ctx).is_undefined()
            }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub enum QueryOrSchemaInputValueId {
    Query(QueryInputValueId),
    Schema(SchemaInputValueId),
}

impl From<QueryInputValueId> for QueryOrSchemaInputValueId {
    fn from(id: QueryInputValueId) -> Self {
        QueryOrSchemaInputValueId::Query(id)
    }
}

impl From<SchemaInputValueId> for QueryOrSchemaInputValueId {
    fn from(id: SchemaInputValueId) -> Self {
        QueryOrSchemaInputValueId::Schema(id)
    }
}

#[derive(Clone, Copy)]
pub enum QueryOrSchemaInputValue<'a> {
    Query(QueryInputValue<'a>),
    Schema(SchemaInputValue<'a>),
}

impl<'a> QueryOrSchemaInputValue<'a> {
    pub fn is_undefined(&self) -> bool {
        match self {
            QueryOrSchemaInputValue::Query(value) => value.is_undefined(),
            QueryOrSchemaInputValue::Schema(_) => false,
        }
    }

    pub fn with_selection_set<'s, 'w>(self, selection_set: &'s InputValueSet) -> QueryOrSchemaInputValueView<'w>
    where
        'a: 'w,
        's: 'w,
    {
        match self {
            QueryOrSchemaInputValue::Query(value) => {
                QueryOrSchemaInputValueView::Query(value.with_selection_set(selection_set))
            }
            QueryOrSchemaInputValue::Schema(value) => {
                QueryOrSchemaInputValueView::Schema(value.with_selection_set(selection_set))
            }
        }
    }
}

impl<'a> Walk<InputValueContext<'a>> for QueryOrSchemaInputValueId {
    type Walker<'w>
        = QueryOrSchemaInputValue<'w>
    where
        'a: 'w;

    fn walk<'w>(self, ctx: impl Into<InputValueContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        let ctx: InputValueContext<'a> = ctx.into();
        match self {
            QueryOrSchemaInputValueId::Query(id) => QueryOrSchemaInputValue::Query(id.walk(ctx)),
            QueryOrSchemaInputValueId::Schema(id) => QueryOrSchemaInputValue::Schema(id.walk(ctx)),
        }
    }
}
