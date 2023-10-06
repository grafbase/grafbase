use std::collections::VecDeque;

use grafbase_sql_ast::ast::{Column, Comparable, ConditionTree, Expression, Row, Select};
use postgres_types::database_definition::TableColumnWalker;
use serde_json::{Map, Value};

use crate::registry::{resolvers::postgres::context::PostgresContext, type_kinds::InputType};

#[derive(Clone)]
pub struct ComplexFilterIterator<'a> {
    context: &'a PostgresContext<'a>,
    input_type: InputType<'a>,
    filter: VecDeque<(String, Value)>,
}

impl<'a> ComplexFilterIterator<'a> {
    pub fn new(
        context: &'a PostgresContext<'a>,
        input_type: InputType<'a>,
        filter: impl IntoIterator<Item = (String, Value)>,
    ) -> Self {
        Self {
            context,
            input_type,
            filter: VecDeque::from_iter(filter),
        }
    }
}

impl<'a> Iterator for ComplexFilterIterator<'a> {
    type Item = ConditionTree<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some((field, value)) = self.filter.pop_front() else { return None };

        let table = self
            .context
            .database_definition
            .find_table_for_client_type(self.input_type.name())
            .expect("table for input type not found");

        // filtering from a related table.
        if let Some(relation) = self
            .context
            .database_definition
            .find_relation_for_client_field(&field, table.id())
        {
            let input_type: InputType<'_> = self
                .input_type
                .field(&field)
                .and_then(|field| self.context.registry().lookup(&field.ty).ok())
                .unwrap();

            let object = if !relation.is_referenced_row_unique() {
                match value {
                    Value::Object(mut object) => match object.remove("contains") {
                        Some(Value::Object(object)) => object,
                        _ => unreachable!("nested filters must be objects"),
                    },
                    _ => unreachable!("nested filters must be objects"),
                }
            } else {
                match value {
                    Value::Object(object) => object,
                    _ => unreachable!("nested filters must be objects"),
                }
            };

            let mut conditions = Vec::new();

            for (referenced, referencing) in relation.referenced_columns().zip(relation.referencing_columns()) {
                let referencing = Column::from((referencing.table().database_name(), referencing.database_name()));
                conditions.push(Expression::from(referenced.database_name().equals(referencing)));
            }

            let nested = Self::new(self.context, input_type, object);

            for condition in nested {
                conditions.push(Expression::from(condition))
            }

            let table = relation.referenced_table();

            let mut select = Select::from_table((table.schema(), table.database_name()));
            select.value(1);
            select.so_that(ConditionTree::And(conditions));

            return Some(ConditionTree::exists(select));
        }

        let operations = match value {
            Value::Object(operations) => operations,
            Value::Array(values) => {
                let mut operations = Vec::with_capacity(values.len());

                for operation in values.into_iter().filter_map(|operation| match operation {
                    Value::Object(obj) => Some(obj),
                    _ => None,
                }) {
                    let nested = Self::new(self.context, self.input_type, operation);

                    for operation in nested {
                        operations.push(Expression::from(operation));
                    }
                }

                let tree = match field.as_str() {
                    "ALL" => ConditionTree::And(operations),
                    "ANY" => ConditionTree::Or(operations),
                    "NONE" => ConditionTree::not(ConditionTree::Or(operations)),
                    _ => unreachable!(),
                };

                return Some(tree);
            }
            _ => return None,
        };

        let column = self
            .context
            .database_definition
            .find_column_for_client_field(&field, table.id())
            .expect("column for input field not found");

        Some(generate_conditions(operations, column))
    }
}

fn generate_conditions(operations: Map<String, Value>, column: TableColumnWalker<'_>) -> ConditionTree<'_> {
    let mut compares = Vec::with_capacity(operations.len());

    for (key, value) in operations {
        let table_column = (column.table().database_name(), column.database_name());

        let compare = match key.as_str() {
            "eq" => match value {
                Value::Null => table_column.is_null(),
                value => table_column.equals(value),
            },
            "ne" => match value {
                Value::Null => table_column.is_not_null(),
                value => table_column.not_equals(value),
            },
            "gt" => table_column.greater_than(value),
            "lt" => table_column.less_than(value),
            "gte" => table_column.greater_than_or_equals(value),
            "lte" => table_column.less_than_or_equals(value),
            "in" => table_column.in_selection(Row::from(value)),
            "nin" => table_column.not_in_selection(Row::from(value)),
            "contains" => table_column.array_contains(value),
            "contained" => table_column.array_contained(value),
            "overlaps" => table_column.array_overlaps(value),
            "not" => {
                let operations = match value {
                    Value::Object(obj) => obj,
                    _ => unreachable!("non-object not filter"),
                };

                let expression = Expression::from(ConditionTree::not(generate_conditions(operations, column)));
                compares.push(expression);

                continue;
            }
            _ => todo!(),
        };

        compares.push(Expression::from(compare));
    }

    ConditionTree::And(compares)
}
