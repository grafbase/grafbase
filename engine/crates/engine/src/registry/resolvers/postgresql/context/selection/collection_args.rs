use crate::{registry::resolvers::postgresql::context::PostgresContext, SelectionField};
use engine_value::{Name, Value};
use grafbase_sql_ast::ast::{Aliasable, Column, Order, OrderDefinition};
use indexmap::IndexMap;
use postgresql_types::database_definition::TableWalker;
use std::borrow::Cow;

/// Argument defining a relay-style GraphQL collection.
#[derive(Debug, Clone)]
pub struct CollectionArgs {
    first: Option<u64>,
    last: Option<u64>,
    order_by: Option<(Vec<OrderDefinition<'static>>, Vec<OrderDefinition<'static>>)>,
    extra_columns: Vec<Column<'static>>,
    before: Option<String>,
    after: Option<String>,
}

impl CollectionArgs {
    pub(super) fn new(ctx: &PostgresContext<'_>, table: TableWalker<'_>, value: &SelectionField<'_>) -> Self {
        let first = value.field.get_argument("first").and_then(|value| value.as_u64());
        let last = value.field.get_argument("last").and_then(|value| value.as_u64());
        let before = value.field.get_argument("before").and_then(|value| value.as_string());
        let after = value.field.get_argument("after").and_then(|value| value.as_string());

        let mut order_by_argument = value
            .field
            .get_argument("orderBy")
            .and_then(|value| value.as_slice())
            .map(Cow::from);

        // For `last` to work, we need some known order for the inner query.
        // If no `orderBy` is set, we order the inner selection with the first unique constraint,
        // so we can select the `FIRST` n items from the reversed inner selection, and then
        // reverse again in the outer queries.
        //
        // We first try to order with the primary key, and if the table has no primary keys, we use the
        // first secondary key we find.
        if let (None, Some(_)) = (&order_by_argument, last) {
            let constraint = table
                .primary_key()
                .or_else(|| table.unique_constraints().next())
                .expect("tables at this point must have at least one unique constraint");

            let mut implicit_fields = Vec::with_capacity(constraint.columns().len());

            for column in constraint.columns() {
                let mut object = IndexMap::new();

                object.insert(
                    Name::new(column.table_column().database_name()),
                    Value::String("ASC".into()),
                );

                implicit_fields.push(Value::Object(object));
            }

            order_by_argument = Some(Cow::from(implicit_fields))
        };

        let order_by = order_by_argument.map(|slice| {
            // ordering the innermost query
            let mut inner_order = Vec::new();

            // the variables used for ordering the json queries
            let mut outer_order = Vec::new();

            // extra columns we have to select (based on ordering)
            let mut extra_columns = Vec::new();

            for value in slice.as_ref() {
                let Some((field, value)) = value.as_object().and_then(IndexMap::first) else { continue };
                let Some(direction) = value.as_str() else { continue };

                // For `last` to work, we must reverse the order of the inner query.
                let inner_direction = match direction {
                    "DESC" if last.is_some() => Order::Asc,
                    "DESC" => Order::Desc,
                    _ if last.is_some() => Order::Desc,
                    _ => Order::Asc,
                };

                // and then reverse the order again for the outer query.
                let outer_direction = match inner_direction {
                    Order::Desc if last.is_some() => Order::Asc,
                    Order::Asc if last.is_some() => Order::Desc,
                    _ => inner_direction,
                };

                let column = ctx
                    .database_definition
                    .find_column_for_client_field(&field, table.id())
                    .expect("ordering with non-existing column");

                let sql_column = Column::from((table.database_name().to_string(), column.database_name().to_string()));

                // We must name our order columns for them to be visible in the order by statement of the
                // outer queries.
                let order_alias = format!("{}_{}", table.database_name(), column.database_name());
                extra_columns.push(sql_column.clone().alias(order_alias.clone()));
                inner_order.push((sql_column.into(), Some(inner_direction)));

                let column = Column::from(order_alias);
                outer_order.push((column.into(), Some(outer_direction)));
            }

            (inner_order, outer_order, extra_columns)
        });

        let (order_by, extra_columns) = match order_by {
            Some((inner_order, outer_order, extra_columns)) => (Some((inner_order, outer_order)), extra_columns),
            None => (None, Vec::new()),
        };

        Self {
            first,
            last,
            order_by,
            extra_columns,
            before,
            after,
        }
    }

    /// Select the first N items. An example GraphQL definition: `userCollection(first: N)`.
    pub(crate) fn first(&self) -> Option<u64> {
        self.first
    }

    /// Select the last N items. An example GraphQL definition: `userCollection(last: N)`.
    pub(crate) fn last(&self) -> Option<u64> {
        self.last
    }

    /// Select the items before the given cursor. An example GraphQL definition:
    /// `userCollection(before: "asdf")`.
    pub(crate) fn before(&self) -> Option<&str> {
        self.before.as_deref()
    }

    /// Select the items after the given cursor. An example GraphQL definition:
    /// `userCollection(after: "asdf")`.
    pub(crate) fn after(&self) -> Option<&str> {
        self.after.as_deref()
    }

    /// Defines the ordering of the collection. The first item in a tuple is the ordering for the innermost
    /// query, and the second one of all the outer queries. An example GraphQL definition:
    /// `userCollection(orderBy: [{ name: DESC }])`.
    pub(crate) fn order_by(&self) -> Option<(&[OrderDefinition<'static>], &[OrderDefinition<'static>])> {
        self.order_by
            .as_ref()
            .map(|(left, right)| (left.as_slice(), right.as_slice()))
    }

    /// A set of extra columns needing to select in the collecting query. Needed to handle the ordering of the outer
    /// layers.
    pub(crate) fn extra_columns(&self) -> impl ExactSizeIterator<Item = Column<'static>> + '_ {
        self.extra_columns.clone().into_iter()
    }
}
