use schema::{FieldDefinition, SchemaInputValueRecord};
use walker::Walk;

use super::{
    DataField, ExecutableDirectiveId, Field, FieldArgument, InputValueContext, Location, QueryInputValue,
    QueryInputValueRecord, ResponseKey, SelectionSet, SelectionSetRecord, TypenameField, Variables,
};

impl<'a> DataField<'a> {
    pub fn response_key_str(&self) -> &'a str {
        &self.ctx.operation.response_keys[self.response_key]
    }
}

impl<'a> TypenameField<'a> {
    pub fn response_key_str(&self) -> &'a str {
        &self.ctx.operation.response_keys[self.response_key]
    }
}

impl<'a> Field<'a> {
    pub fn response_key(&self) -> ResponseKey {
        match self {
            Field::Data(data) => data.response_key,
            Field::Typename(typename) => typename.response_key,
        }
    }
    pub fn response_key_str(&self) -> &'a str {
        match self {
            Field::Data(data) => data.response_key_str(),
            Field::Typename(typename) => typename.response_key_str(),
        }
    }
    pub fn location(&self) -> Location {
        match self {
            Field::Data(data) => data.location,
            Field::Typename(typename) => typename.location,
        }
    }
    pub fn definition(&self) -> Option<FieldDefinition<'a>> {
        match self {
            Field::Data(data) => Some(data.definition()),
            Field::Typename(_) => None,
        }
    }
    pub fn selection_set(&self) -> SelectionSet<'a> {
        match self {
            Field::Data(data) => data.selection_set(),
            Field::Typename(field) => SelectionSetRecord::empty().walk(field.ctx),
        }
    }
    pub fn directive_ids(&self) -> &'a [ExecutableDirectiveId] {
        match self {
            Field::Data(data) => data.as_ref().directive_ids.as_slice(),
            Field::Typename(typename) => typename.as_ref().directive_ids.as_slice(),
        }
    }
}

impl<'a> FieldArgument<'a> {
    pub fn value<'v>(&self, variables: &'v Variables) -> QueryInputValue<'v>
    where
        'a: 'v,
    {
        let ctx = InputValueContext {
            schema: self.ctx.schema,
            query_input_values: &self.ctx.operation.query_input_values,
            variables,
        };
        self.as_ref().value_id.walk(ctx)
    }

    /// Used for GraphQL query generation to only include values in the query string that would be
    /// present after query sanitization.
    pub fn value_as_sanitized_query_const_value_str(&self) -> Option<&'a str> {
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
