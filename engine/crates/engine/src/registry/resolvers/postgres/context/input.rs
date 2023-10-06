use std::collections::VecDeque;

use postgres_types::database_definition::{
    DatabaseDefinition, DatabaseType, EnumWalker, TableColumnWalker, TableWalker,
};
use serde_json::Value;

use crate::registry::type_kinds::InputType;

pub enum InputItem<'a> {
    /// Inserts a single column value.
    Column(TableColumnWalker<'a>, Value),
}

pub struct InputIterator<'a> {
    database_definition: &'a DatabaseDefinition,
    table: TableWalker<'a>,
    input: VecDeque<(String, Value)>,
}

impl<'a> InputIterator<'a> {
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
            input: VecDeque::from_iter(input),
        }
    }
}

impl<'a> Iterator for InputIterator<'a> {
    type Item = InputItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let (field, value) = self.input.pop_front()?;

        let column = self
            .database_definition
            .find_column_for_client_field(&field, self.table.id())
            .expect("column for client field not found");

        let value = match (value, column.database_type()) {
            (Value::String(value), DatabaseType::Enum(r#enum)) => rename_enum_variant(r#enum, &value),
            (Value::Array(values), DatabaseType::Enum(r#enum)) => {
                let values = values
                    .into_iter()
                    .map(|value| rename_enum_variant(r#enum, value.as_str().expect("must be a string")))
                    .collect();

                Value::Array(values)
            }
            (value, _) => value,
        };

        Some(InputItem::Column(column, value))
    }
}

fn rename_enum_variant(r#enum: EnumWalker<'_>, variant: &str) -> Value {
    let variant = r#enum
        .rename_variant(variant)
        .expect("invalid enum variant")
        .to_string();

    Value::String(variant)
}
