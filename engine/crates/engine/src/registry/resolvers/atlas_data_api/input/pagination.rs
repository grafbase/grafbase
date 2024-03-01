use graphql_cursor::GraphqlCursor;
use serde_json::{json, Value};

use crate::{
    registry::resolvers::atlas_data_api::{
        consts::{OP_AND, OP_EQ, OP_GT, OP_LT, OP_NE, OP_OR},
        cursor::{AtlasCursor, CursorField, OrderByDirection},
        JsonMap,
    },
    ContextField, ServerResult,
};

#[derive(Debug, Clone, Copy)]
enum CursorParameter {
    Before,
    After,
}

#[derive(Debug, Clone, Copy)]
enum ValueIncrement {
    Shrinking,
    Growing,
}

impl ValueIncrement {
    fn is_growing(self) -> bool {
        matches!(self, Self::Growing)
    }
}

pub(crate) fn before_definition(ctx: &ContextField) -> Option<GraphqlCursor> {
    ctx.input_by_name("before").ok()
}

pub(crate) fn after_definition(ctx: &ContextField) -> Option<GraphqlCursor> {
    ctx.input_by_name("after").ok()
}

pub(super) fn before(ctx: &ContextField<'_>) -> ServerResult<Option<JsonMap>> {
    let cursor = before_definition(ctx);

    let mut before = match cursor {
        Some(cursor) => AtlasCursor::try_from(cursor)?,
        None => return Ok(None),
    };

    let mut filters = Vec::new();

    while !before.fields.is_empty() {
        if let Some(filter) = fields_filter(&before.fields, CursorParameter::Before) {
            filters.push(filter);
        }

        before.fields.pop();
    }

    let filter = if filters.len() == 1 {
        filters.pop().unwrap()
    } else {
        let mut filter = JsonMap::new();

        filter.insert(
            OP_OR.to_string(),
            Value::Array(filters.into_iter().map(Value::from).collect()),
        );

        filter
    };

    Ok(Some(filter))
}

pub(super) fn after(ctx: &ContextField<'_>) -> ServerResult<Option<JsonMap>> {
    let cursor = after_definition(ctx);

    let mut after = match cursor {
        Some(cursor) => AtlasCursor::try_from(cursor)?,
        None => return Ok(None),
    };

    let mut filters = Vec::new();

    while !after.fields.is_empty() {
        if let Some(filter) = fields_filter(&after.fields, CursorParameter::After) {
            filters.push(filter);
        }

        after.fields.pop();
    }

    let filter = if filters.len() == 1 {
        filters.pop().unwrap()
    } else {
        let mut filter = JsonMap::new();

        filter.insert(
            OP_OR.to_string(),
            Value::Array(filters.into_iter().map(Value::from).collect()),
        );

        filter
    };

    Ok(Some(filter))
}

/// Sets a filter based on the usage of before/after and the fields used in order by.
/// There is always an order with Mongo, and if it's not explicitly set, the ordering is
/// by the id field in ascending order.
///
/// # Pagination example without ordering:
///
/// (id > "value in cursor set in after")
///
/// or
///
/// (id < "value in cursor set in before")
///
/// # Pagination example with ordering
///
/// If ordering with field "name", we must take nulls into account. Ascending order:
///
/// (name = "value in cursor" AND id > "value in cursor") OR (name > "value in cursor")
///
/// Descending order:
///
/// (name = "value in cursor" AND id > "value in cursor") OR (name < "value in cursor")
///
/// # Pagination example with ordering using more than one field
///
/// Ordering with "name" and "age", ascending.
///
/// (name = "value" AND age = "value" AND id > "value") OR (name = "value" age > "value") OR (name > "value")
///
/// Ordering with "name" and "age", descending.
///
/// (name = "value" AND age = "value" AND id > "value") OR (name = "value" AND (age < "value" OR age = null)) OR (name < "value" OR name = null)
///
/// An index is adviced to use in columns used in sorting and pagination.
fn fields_filter(fields: &[CursorField], cursor_parameter: CursorParameter) -> Option<JsonMap> {
    let max_id = fields.len() - 1;
    let mut filters = Vec::new();

    for (i, field) in fields.iter().enumerate() {
        // The direction where this field is going based on are we using before/after and is
        // the field ascending or descending in the sort.
        let increment = match (field.direction, cursor_parameter) {
            (OrderByDirection::Ascending, CursorParameter::Before)
            | (OrderByDirection::Descending, CursorParameter::After) => ValueIncrement::Shrinking,
            (OrderByDirection::Ascending, CursorParameter::After)
            | (OrderByDirection::Descending, CursorParameter::Before) => ValueIncrement::Growing,
        };

        if i == max_id {
            if field.value.is_null() {
                if increment.is_growing() {
                    let mut map = JsonMap::new();

                    map.insert(
                        field.name.clone(),
                        json!({
                            OP_NE: Value::Null
                        }),
                    );

                    filters.push(map);
                }
            } else if !field.value.is_id() {
                match increment {
                    ValueIncrement::Shrinking => {
                        let mut map = JsonMap::new();

                        map.insert(
                            OP_OR.to_string(),
                            json!([
                                {
                                    &field.name: {
                                        OP_LT: Value::from(field.value.clone())
                                    }
                                },
                                {
                                    &field.name: {
                                        OP_EQ: Value::Null
                                    }

                                }
                            ]),
                        );

                        filters.push(map);
                    }
                    ValueIncrement::Growing => {
                        let mut map = JsonMap::new();

                        map.insert(
                            field.name.clone(),
                            json!({
                                OP_GT: Value::from(field.value.clone()),
                            }),
                        );

                        filters.push(map);
                    }
                }
            } else {
                match increment {
                    ValueIncrement::Shrinking => {
                        let mut map = JsonMap::new();

                        map.insert(
                            field.name.clone(),
                            json!({
                                OP_LT: Value::from(field.value.clone())
                            }),
                        );

                        filters.push(map);
                    }
                    ValueIncrement::Growing => {
                        let mut map = JsonMap::new();

                        map.insert(
                            field.name.clone(),
                            json!({
                                OP_GT: Value::from(field.value.clone())
                            }),
                        );

                        filters.push(map);
                    }
                }
            }
        } else {
            let mut map = JsonMap::new();
            map.insert(
                field.name.clone(),
                json!({
                    OP_EQ: Value::from(field.value.clone())
                }),
            );

            filters.push(map);
        };
    }

    if filters.is_empty() {
        None
    } else if filters.len() == 1 {
        Some(filters.pop().unwrap())
    } else {
        let mut filter = JsonMap::new();

        filter.insert(
            OP_AND.to_string(),
            Value::Array(filters.into_iter().map(Value::from).collect()),
        );

        Some(filter)
    }
}
