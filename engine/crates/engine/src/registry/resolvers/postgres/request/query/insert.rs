use grafbase_sql_ast::ast::{
    json_build_object, Aliasable, Column, CommonTableExpression, Insert, MultiRowInsert, Select, SingleRowInsert,
};

use crate::registry::resolvers::postgres::context::{InputItem, InputIterator, PostgresContext, TableSelection};

enum InsertType<'a> {
    Single(SingleRowInsert<'a>),
    Multi(MultiRowInsert<'a>),
}

pub fn build<'a>(
    ctx: &'a PostgresContext<'a>,
    input: impl IntoIterator<Item = InputIterator<'a>>,
) -> Result<Select<'a>, crate::Error> {
    let mut query = None;

    for input in input {
        match query.take() {
            None => {
                query = Some(InsertType::Single(create_insert(ctx, input)));
            }
            Some(InsertType::Single(previous_insert)) => {
                let combined = previous_insert
                    .merge(create_insert(ctx, input))
                    .map_err(|error| crate::Error::new(error.to_string()))?;

                query = Some(InsertType::Multi(combined));
            }
            Some(InsertType::Multi(mut previous_insert)) => {
                previous_insert
                    .extend(create_insert(ctx, input))
                    .map_err(|error| crate::Error::new(error.to_string()))?;
            }
        }
    }

    let insert_name = format!("{}_{}_insert", ctx.table().schema(), ctx.table().database_name());

    let mut returning = Vec::new();
    let mut selected_data = Vec::new();

    for selection in ctx.selection() {
        match selection? {
            TableSelection::Column(column) => {
                selected_data.push((
                    column.database_name(),
                    Column::from((insert_name.clone(), column.database_name())),
                ));

                returning.push(column.database_name());
            }
            // we will not have relations in the first phase
            TableSelection::JoinUnique(..) | TableSelection::JoinMany(..) => {
                todo!("we'll get back to this with nested inserts")
            }
        }
    }

    let mut insert = match query.expect("we must have at least one input document") {
        InsertType::Single(insert) => insert.build(),
        InsertType::Multi(insert) => insert.build(),
    };

    insert.returning(returning);

    let mut select = Select::from_table(insert_name.clone());
    select.with(CommonTableExpression::new(insert_name.clone(), insert));
    select.value(json_build_object(selected_data).alias("root"));

    Ok(select)
}

fn create_insert<'a>(ctx: &'a PostgresContext, input: InputIterator<'a>) -> SingleRowInsert<'a> {
    let mut insert = Insert::single_into(ctx.table().database_name());

    for input in input {
        match input {
            InputItem::Column(column, value) => insert.value(column.database_name(), value),
        }
    }

    insert
}
