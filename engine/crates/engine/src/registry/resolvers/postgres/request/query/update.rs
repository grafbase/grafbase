use grafbase_sql_ast::ast::{
    json_build_object, Aliasable, Column, CommonTableExpression, ConditionTree, Select, Update,
};

use crate::registry::resolvers::postgres::context::{FilterIterator, PostgresContext, TableSelection, UpdateInputItem};

pub fn build<'a>(ctx: &'a PostgresContext<'a>, filter: FilterIterator<'a>) -> Result<Select<'a>, crate::Error> {
    let mut update = Update::table(ctx.table().database_name());
    update.so_that(filter.fold(ConditionTree::NoCondition, ConditionTree::and));

    for item in ctx.update_input()? {
        match item {
            UpdateInputItem::Column(column, expression) => update.set(column.database_name(), expression),
        }
    }

    let update_name = format!("{}_{}_update", ctx.table().schema(), ctx.table().database_name());

    let mut returning = Vec::new();
    let mut selected_data = Vec::new();

    for selection in ctx.selection() {
        match selection? {
            TableSelection::Column(column) => {
                selected_data.push((
                    column.database_name(),
                    Column::from((update_name.clone(), column.database_name())),
                ));

                returning.push(column.database_name());
            }
            // we will not have relations in the first phase
            TableSelection::JoinUnique(..) | TableSelection::JoinMany(..) => {
                todo!("we'll get back to this with nested updates")
            }
        }
    }

    update.returning(returning);

    let mut select = Select::from_table(update_name.clone());
    select.with(CommonTableExpression::new(update_name, update));
    select.value(json_build_object(selected_data).alias("root"));

    Ok(select)
}
