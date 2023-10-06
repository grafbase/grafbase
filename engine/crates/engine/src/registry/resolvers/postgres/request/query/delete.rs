use crate::registry::resolvers::postgres::context::{FilterIterator, PostgresContext, TableSelection};
use grafbase_sql_ast::ast::{json_build_object, Aliasable, Column, ConditionTree, Delete, Table};

pub fn build<'a>(ctx: &'a PostgresContext<'a>, filter: FilterIterator<'a>) -> Result<Delete<'a>, crate::Error> {
    let sql_table = Table::from((ctx.table().schema(), ctx.table().database_name())).alias(ctx.table().database_name());
    let mut returning = Vec::new();

    for selection in ctx.selection() {
        match selection? {
            TableSelection::Column(column) => {
                returning.push((column.database_name(), Column::from(column.database_name())));
            }
            // our output type doesn't have relations, so this is never reachable
            TableSelection::JoinMany(..) | TableSelection::JoinUnique(..) => {
                unreachable!("we cannot join in a delete statement")
            }
        }
    }

    let mut query = Delete::from_table(sql_table);
    query.so_that(filter.fold(ConditionTree::NoCondition, ConditionTree::and));
    query.returning([json_build_object(returning).alias("root")]);

    Ok(query)
}
