use schema::SchemaInputValueRecord;

use crate::operation::QueryInputValueRecord;

use super::FieldArgument;

impl<'a> FieldArgument<'a> {
    /// Used for GraphQL query generation to only include values in the query string that would be
    /// present after query sanitization.
    pub(crate) fn value_as_sanitized_query_const_value_str(&self) -> Option<&'a str> {
        Some(match &self.ctx.operation.query_input_values[self.value_id] {
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
}
