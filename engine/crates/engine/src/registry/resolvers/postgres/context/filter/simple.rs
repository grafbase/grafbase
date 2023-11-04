use crate::registry::type_kinds::InputType;
use grafbase_sql_ast::ast::{Comparable, Compare};
use indexmap::IndexSet;
use postgres_connector_types::database_definition::{DatabaseDefinition, TableColumnId};
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
    constrained_columns: IndexSet<TableColumnId>,
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
            constrained_columns: IndexSet::new(),
        }
    }

    fn push_constrained_column(&mut self, column_id: TableColumnId) {
        self.constrained_columns.insert(column_id);
    }
}

impl<'a> Iterator for ByFilterIterator<'a> {
    type Item = Compare<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // We are having a nested input type, which we iterate over.
        if let Some(item) = self.nested.as_mut().and_then(Iterator::next) {
            return Some(item);
        }

        let table = self
            .database_definition
            .find_table_for_client_type(self.input_type.name())
            .expect("table for input type not found");

        let Some((field, value)) = self.filter.pop_front() else {
            // solves the issue where user emits a value for a nullable composite unique.
            return self.constrained_columns.pop().map(|column_id| {
                let column = self.database_definition.walk(column_id);
                (table.database_name(), column.database_name()).is_null()
            });
        };

        // If selecting an object, we don't care about the name of the object, but selecting the
        // fields defined in the input.
        //
        // E.g. in `user(by: { nameEmail: { name: "foo", email: "bar" }})`, we do not care about `nameEmail`,
        // but the nested values `name` and `email` are used in the query filters.
        if let Value::Object(map) = value {
            let mut nested = ByFilterIterator::new(self.database_definition, self.input_type, map);

            let constraint = self
                .database_definition
                .find_unique_constraint_for_client_field(&field, table.id())
                .expect("constraint for input field not found");

            for column in constraint.columns() {
                nested.push_constrained_column(column.table_column().id());
            }

            let item = nested.next();
            self.nested = Some(Box::new(nested));

            return item;
        };

        let column = self
            .database_definition
            .find_column_for_client_field(&field, table.id())
            .expect("column for input field not found");

        self.constrained_columns.remove(&column.id());

        match value {
            Value::Null => Some((table.database_name(), column.database_name()).is_null()),
            _ => Some((table.database_name(), column.database_name()).equals(value)),
        }
    }
}
