use crate::SelectionField;
use engine_value::{Name, Value};
use grafbase_sql_ast::ast::{Aliasable, Column, Order, OrderDefinition};
use indexmap::IndexMap;
use postgresql_types::database_definition::{DatabaseDefinition, TableWalker};

#[derive(Debug, Clone, Default)]
pub struct CollectionOrdering {
    inner: Vec<((String, String), Option<Order>)>,
    outer: Vec<(String, Option<Order>)>,
}

impl CollectionOrdering {
    pub fn raw_order(&self) -> impl ExactSizeIterator<Item = (&str, Option<Order>)> + '_ {
        self.inner.iter().map(|((_, column), order)| (column.as_str(), *order))
    }

    pub fn inner(&self) -> impl ExactSizeIterator<Item = OrderDefinition<'static>> + '_ {
        self.inner
            .iter()
            .map(|((table, column), order)| (Column::from((table.clone(), column.clone())).into(), *order))
    }

    pub fn outer(&self) -> impl ExactSizeIterator<Item = OrderDefinition<'static>> + '_ {
        self.outer.iter().map(|(column, order)| {
            let column = Column::from(column.clone());
            (column.into(), *order)
        })
    }
}

/// Argument defining a relay-style GraphQL collection.
#[derive(Debug, Clone)]
pub struct CollectionArgs {
    first: Option<u64>,
    last: Option<u64>,
    order_by: CollectionOrdering,
    extra_columns: Vec<Column<'static>>,
    before: Option<String>,
    after: Option<String>,
}

impl CollectionArgs {
    pub(crate) fn new(
        database_definition: &DatabaseDefinition,
        table: TableWalker<'_>,
        value: &SelectionField<'_>,
    ) -> Self {
        let first = value.field.get_argument("first").and_then(|value| value.as_u64());
        let last = value.field.get_argument("last").and_then(|value| value.as_u64());
        let before = value.field.get_argument("before").and_then(|value| value.as_string());
        let after = value.field.get_argument("after").and_then(|value| value.as_string());

        let mut order_by_argument = value
            .field
            .get_argument("orderBy")
            .and_then(|value| value.as_slice())
            .map(<[engine_value::Value]>::to_vec)
            .unwrap_or_default();

        let constraint = table
            .implicit_ordering_key()
            .expect("tables at this point must have at least one unique constraint");

        for column in constraint.columns() {
            if order_by_argument.iter().any(|value| match value {
                Value::Object(ref obj) => obj.keys().any(|key| key == column.table_column().client_name()),
                _ => false,
            }) {
                continue;
            }

            let mut object = IndexMap::new();

            object.insert(
                Name::new(column.table_column().client_name()),
                Value::String("ASC".into()),
            );

            order_by_argument.push(engine_value::Value::Object(object))
        }

        // ordering the innermost query
        let mut order_by = CollectionOrdering::default();

        // extra columns we have to select (based on ordering)
        let mut extra_columns = Vec::new();

        for value in order_by_argument.into_iter() {
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

            let column = database_definition
                .find_column_for_client_field(&field, table.id())
                .expect("ordering with non-existing column");

            let sql_column = Column::from((table.database_name().to_string(), column.database_name().to_string()));

            // We must name our order columns for them to be visible in the order by statement of the
            // outer queries.
            let alias = format!("{}_{}", table.database_name(), column.database_name());
            extra_columns.push(sql_column.clone().alias(alias.clone()));

            order_by.inner.push((
                (table.database_name().to_string(), column.database_name().to_string()),
                Some(inner_direction),
            ));

            order_by.outer.push((alias, Some(outer_direction)));
        }

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
    pub(crate) fn order_by(&self) -> &CollectionOrdering {
        &self.order_by
    }

    /// A set of extra columns needing to select in the collecting query. Needed to handle the ordering of the outer
    /// layers.
    pub(crate) fn extra_columns(&self) -> impl ExactSizeIterator<Item = Column<'static>> + '_ {
        self.extra_columns.clone().into_iter()
    }
}
