use std::collections::VecDeque;

use grafbase_sql_ast::ast::{Column, Expression, SqlOp};
use postgres_types::database_definition::{DatabaseDefinition, TableColumnWalker, TableWalker};
use serde_json::Value;

use crate::registry::type_kinds::InputType;

pub enum UpdateInputItem<'a> {
    /// Updates a single column value.
    Column(TableColumnWalker<'a>, Expression<'a>),
}

pub struct UpdateInputIterator<'a> {
    database_definition: &'a DatabaseDefinition,
    table: TableWalker<'a>,
    input: VecDeque<(String, Value)>,
}

impl<'a> UpdateInputIterator<'a> {
    pub fn new(
        database_definition: &'a DatabaseDefinition,
        input_type: InputType<'a>,
        input: impl IntoIterator<Item = (String, Value)>,
    ) -> Self {
        let table = database_definition
            .find_table_for_client_type(input_type.name())
            .expect("table for client type not found");

        Self {
            database_definition,
            table,
            input: input.into_iter().collect(),
        }
    }
}

impl<'a> Iterator for UpdateInputIterator<'a> {
    type Item = UpdateInputItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let (field, value) = self.input.pop_front()?;

        let column = self
            .database_definition
            .find_column_for_client_field(&field, self.table.id())
            .expect("column for client field not found");

        let sql_column = Column::from(column.database_name());

        let value = match value {
            Value::Object(value) => value,
            _ => unreachable!("our schema prevents non-object values here"),
        };

        // the type is oneOf, so we always have at most one operation in the object
        let expression = match value.into_iter().next() {
            Some((key, value)) if key == "set" => Expression::from(value),
            Some((key, value)) if key == "increment" => Expression::from(sql_column) + Expression::from(value),
            Some((key, value)) if key == "decrement" || key == "deleteKey" => {
                Expression::from(sql_column) - Expression::from(value)
            }
            Some((key, value)) if key == "multiply" => Expression::from(sql_column) * Expression::from(value),
            Some((key, value)) if key == "divide" => Expression::from(sql_column) / Expression::from(value),
            Some((key, value)) if key == "append" => {
                let value = if column.database_type().is_jsonb() {
                    Value::String(serde_json::to_string(&value).unwrap())
                } else {
                    value
                };

                let op = SqlOp::Append(Expression::from(sql_column), Expression::from(value));
                Expression::from(op)
            }
            Some((key, value)) if key == "prepend" => {
                let value = if column.database_type().is_jsonb() {
                    Value::String(serde_json::to_string(&value).unwrap())
                } else {
                    value
                };

                let op = SqlOp::Append(Expression::from(value), Expression::from(sql_column));
                Expression::from(op)
            }
            Some((key, value)) if key == "deleteAtPath" => {
                let op = SqlOp::JsonDeleteAtPath(Expression::from(sql_column), Expression::from(value));

                Expression::from(op)
            }
            Some((key, _)) => todo!("operation {key}"),
            None => unreachable!("oneOf type prevents this"),
        };

        Some(UpdateInputItem::Column(column, expression))
    }
}
