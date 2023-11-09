use engine_value::{Name, Value};
use grafbase_sql_ast::ast::{Aliasable, Column, Comparable, ConditionTree, Expression, Order, OrderDefinition};
use indexmap::IndexMap;
use postgres_connector_types::{
    cursor::{OrderDirection, SQLCursor},
    database_definition::{DatabaseDefinition, TableWalker},
};
use runtime::search::GraphqlCursor;
use serde::Deserialize;

use crate::{Error, SelectionField};

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
    before: Option<SQLCursor>,
    after: Option<SQLCursor>,
}

impl CollectionArgs {
    pub(crate) fn new(
        database_definition: &DatabaseDefinition,
        table: TableWalker<'_>,
        value: &SelectionField<'_>,
    ) -> Result<Self, Error> {
        let first = value.field.get_argument("first").and_then(|value| value.as_u64());
        let last = value.field.get_argument("last").and_then(|value| value.as_u64());

        match (first, last) {
            (Some(_), Some(_)) => {
                return Err(Error::new("first and last parameters can't be both defined"));
            }
            (None, None) => {
                return Err(Error::new(
                    "please limit your selection by setting either the first or last parameter",
                ));
            }
            _ => (),
        }

        let before = value
            .field
            .get_argument("before")
            .and_then(|value| value.node.clone().into_const());

        let before = match before {
            Some(before) => {
                let cursor = GraphqlCursor::deserialize(before)
                    .map_err(|error| Error::new(format!("invalid cursor: {error}")))
                    .and_then(|cursor| SQLCursor::try_from(cursor).map_err(|error| Error::new(error.to_string())))?;

                Some(cursor)
            }
            None => None,
        };

        let after = value
            .field
            .get_argument("after")
            .and_then(|value| value.node.clone().into_const());

        let after = match after {
            Some(after) => {
                let cursor = GraphqlCursor::deserialize(after)
                    .map_err(|error| Error::new(format!("invalid cursor: {error}")))
                    .and_then(|cursor| SQLCursor::try_from(cursor).map_err(|error| Error::new(error.to_string())))?;

                Some(cursor)
            }
            None => None,
        };

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

        for value in order_by_argument {
            let Some((field, value)) = value.as_object().and_then(IndexMap::first) else {
                continue;
            };
            let Some(direction) = value.as_str() else { continue };

            // For `last` to work, we must reverse the order of the inner query.
            let inner_direction = match direction {
                "DESC" if last.is_some() => Order::AscNullsFirst,
                "DESC" => Order::DescNullsFirst,
                _ if last.is_some() => Order::DescNullsFirst,
                _ => Order::AscNullsFirst,
            };

            // and then reverse the order again for the outer query.
            let outer_direction = match inner_direction {
                Order::DescNullsFirst if last.is_some() => Order::AscNullsFirst,
                Order::AscNullsFirst if last.is_some() => Order::DescNullsFirst,
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

        Ok(Self {
            first,
            last,
            order_by,
            extra_columns,
            before,
            after,
        })
    }

    /// A filter that allows before/after arguments to work correspondingly.
    pub(crate) fn pagination_filter(&self) -> Option<ConditionTree<'static>> {
        let cursor = match (self.before(), self.after()) {
            (Some(cursor), _) | (_, Some(cursor)) => cursor,
            _ => return None,
        };

        let mut fields = cursor.fields().collect::<Vec<_>>();
        let mut filters = Vec::new();

        while !fields.is_empty() {
            if let Some(filter) = generate_filter(&fields) {
                filters.push(filter);
            }

            fields.pop();
        }

        let filter = if filters.len() == 1 {
            ConditionTree::single(filters.pop().unwrap())
        } else {
            ConditionTree::Or(filters)
        };

        Some(filter)
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
    fn before(&self) -> Option<&SQLCursor> {
        self.before.as_ref()
    }

    /// Select the items after the given cursor. An example GraphQL definition:
    /// `userCollection(after: "asdf")`.
    fn after(&self) -> Option<&SQLCursor> {
        self.after.as_ref()
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

fn generate_filter(fields: &[(&str, &serde_json::Value, OrderDirection)]) -> Option<Expression<'static>> {
    let mut filters: Vec<Expression<'static>> = Vec::new();
    let max_id = fields.len() - 1;

    for (i, (column, value, direction)) in fields.iter().enumerate() {
        let column = Column::from((*column).to_string());

        if i == max_id {
            if value.is_null() {
                if let OrderDirection::Ascending = direction {
                    filters.push(column.is_not_null().into())
                }
            } else {
                match direction {
                    OrderDirection::Ascending => {
                        filters.push(column.greater_than((*value).clone()).into());
                    }
                    OrderDirection::Descending => {
                        let tree = ConditionTree::Or(vec![
                            column.clone().less_than((*value).clone()).into(),
                            column.is_null().into(),
                        ]);

                        filters.push(tree.into());
                    }
                }
            }
        } else {
            filters.push(column.equals((*value).clone()).into());
        }
    }

    if filters.is_empty() {
        None
    } else if filters.len() == 1 {
        Some(filters.pop().unwrap())
    } else {
        Some(ConditionTree::And(filters).into())
    }
}
