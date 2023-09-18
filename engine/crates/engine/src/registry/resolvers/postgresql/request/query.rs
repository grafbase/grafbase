mod builder;

pub use builder::SelectBuilder;
use grafbase_sql_ast::ast::{
    coalesce, json_agg, raw, row_to_json, Aliasable, Column, Comparable, ConditionTree, Expression, Joinable, Ordering,
    Select, Table,
};

use crate::registry::resolvers::postgresql::context::TableSelection;

/// Builds the outermost query of the selection. Gathers all the data from the nested
/// queries into a JSON array, which is serialized in the database.
///
/// [example query](https://gist.github.com/pimeys/a7535acb0922fa432562539f5d8123c3)
pub fn build<'a>(builder: SelectBuilder<'a>) -> Select<'a> {
    // The innermost query of the select. All filters, ordering, limits etc. are defined here.
    let sql_table =
        Table::from((builder.table().schema(), builder.table().database_name())).alias(builder.table().database_name());

    let mut inner_nested = Select::from_table(sql_table);

    if let Some(filters) = builder.filter() {
        for filter in filters.clone() {
            inner_nested.and_where(filter);
        }
    }

    if let Some(ref args) = builder.collection_args() {
        if let Some((order_by, _)) = args.order_by() {
            for ordering in order_by {
                inner_nested.order_by(ordering.clone());
            }
        }

        if let Some(limit) = args.first() {
            inner_nested.limit(limit as u32);
        }

        // There's no `LAST` in PostgreSQL, so we limit the inner selection which is ordered in an opposite way,
        // and re-order it in the outer query.
        if let Some(limit) = args.last() {
            inner_nested.limit(limit as u32);
        }

        if args.before().is_some() {
            todo!("not yet done");
        }

        if args.after().is_some() {
            todo!("not yet done");
        }
    }

    if let Some(relation) = builder.relation() {
        for (left, right) in relation.referencing_columns().zip(relation.referenced_columns()) {
            let left_column = Column::from((left.table().client_name(), left.database_name()));
            let right_column = Column::from((right.table().database_name(), right.database_name()));

            inner_nested.and_where(left_column.equals(right_column));
        }
    }

    // The middle query of the selection. Collects nested data from joins, and combines it with the main
    // query. Returns all rows as JSON objects.
    let mut collecting_select = Select::from_table(Table::from(inner_nested).alias(builder.table().client_name()));

    for selection in builder.selection() {
        match selection {
            TableSelection::Column(column) => {
                collecting_select.column((builder.table().client_name(), column.database_name()));
            }
            // m:1, 1:1
            TableSelection::JoinUnique(relation, selection) => {
                let client_field_name = relation.client_field_name();
                collecting_select.column(client_field_name.clone());

                let mut builder = SelectBuilder::new(relation.referenced_table(), selection, client_field_name.clone());
                builder.set_relation(relation);

                // recurse
                let mut join_data = Table::from(build(builder))
                    .alias(client_field_name)
                    .on(ConditionTree::single(raw("true")));

                join_data.lateral();
                collecting_select.left_join(join_data);
            }
            // 1:m
            TableSelection::JoinMany(relation, selection, args) => {
                let client_field_name = relation.client_field_name();
                collecting_select.column(client_field_name.clone());

                let mut builder = SelectBuilder::new(relation.referenced_table(), selection, client_field_name.clone());
                builder.set_collection_args(args);
                builder.set_relation(relation);

                // recurse
                let mut join_data = Table::from(build(builder))
                    .alias(client_field_name)
                    .on(ConditionTree::single(raw("true")));

                join_data.lateral();
                collecting_select.left_join(join_data);
            }
        }
    }

    let mut json_select = Select::from_table(Table::from(collecting_select).alias(builder.table().database_name()));
    json_select.value(row_to_json(builder.table().database_name(), false).alias(builder.field_name().to_string()));

    if let Some(args) = builder.collection_args() {
        for column in args.extra_columns() {
            json_select.column(column.clone());
        }
    }

    match builder.collection_args() {
        Some(args) => {
            // SQL doesn't guarantee ordering if it's not defined in the query.
            // we'll reuse the nested ordering here.
            if let Some((_, order_by)) = args.order_by() {
                for ordering in order_by {
                    json_select.order_by(ordering.clone());
                }
            }

            let mut json_aggregation =
                Select::from_table(Table::from(json_select).alias(builder.table().database_name().to_string()));

            let column = Column::from((builder.table().database_name(), builder.field_name().to_string()));

            // SQL doesn't guarantee ordering if it's not defined in the query.
            // we'll reuse the nested ordering here.
            let json_agg = match args.order_by() {
                Some((_, order_by)) => {
                    let mut ordering = Ordering::default();

                    for order in order_by {
                        ordering.append(order.clone());
                    }

                    json_agg(column, Some(ordering), false)
                }
                None => json_agg(column, None, false),
            };

            let json_value = coalesce([Expression::from(json_agg), raw("'[]'")]);
            json_aggregation.value(json_value.alias(builder.field_name().to_string()));

            json_aggregation
        }
        None => json_select,
    }
}
