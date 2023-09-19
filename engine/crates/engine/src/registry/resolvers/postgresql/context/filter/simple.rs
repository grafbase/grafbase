use crate::registry::type_kinds::InputType;
use grafbase_sql_ast::ast::Comparable;
use postgresql_types::database_definition::DatabaseDefinition;
use serde_json::Value;
use std::{collections::VecDeque, iter::Iterator};

/// An iterator for a "simple" filter, e.g. a filter that's defined
/// as `by` argument from the client, and has at most one unique equality
/// check.
#[derive(Debug, Clone)]
pub struct ByFilterIterator<'a> {
    database_definition: &'a DatabaseDefinition,
    input_type: InputType<'a>,
    filter: VecDeque<(String, Value)>,
    nested: Option<Box<ByFilterIterator<'a>>>,
}

impl<'a> ByFilterIterator<'a> {
    pub fn new(
        database_definition: &'a DatabaseDefinition,
        input_type: InputType<'a>,
        filter: impl IntoIterator<Item = (String, Value)>,
    ) -> Self {
        Self {
            database_definition,
            input_type,
            filter: VecDeque::from_iter(filter),
            nested: None,
        }
    }
}

impl<'a> Iterator for ByFilterIterator<'a> {
    type Item = grafbase_sql_ast::ast::Compare<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // We are having a nested input type, which we iterate over.
        if let Some(item) = self.nested.as_mut().and_then(Iterator::next) {
            return Some(item);
        }

        let Some((field, value)) = self.filter.pop_front() else { return None };

        // If selecting an object, we don't care about the name of the object, but selecting the
        // fields defined in the input.
        //
        // E.g. in `user(by: { nameEmail: { name: "foo", email: "bar" }})`, we do not care about `nameEmail`,
        // but the nested values `name` and `email` are used in the query filters.
        if let Value::Object(map) = value {
            let mut nested = ByFilterIterator::new(self.database_definition, self.input_type, map);

            let item = nested.next();
            self.nested = Some(Box::new(nested));

            return item;
        };

        let table = self
            .database_definition
            .find_table_for_client_type(self.input_type.name())
            .expect("table for input type not found");

        let column = self
            .database_definition
            .find_column_for_client_field(&field, table.id())
            .expect("column for input field not found");

        match value {
            Value::Null => Some((table.database_name(), column.database_name()).is_null()),
            _ => Some((table.database_name(), column.database_name()).equals(value)),
        }
    }
}
