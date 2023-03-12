use serde_json::Value;
use thiserror::Error;

use grafbase_runtime::search;

use crate::{
    names::{
        INPUT_FIELD_FILTER_ALL, INPUT_FIELD_FILTER_ANY, INPUT_FIELD_FILTER_EQ,
        INPUT_FIELD_FILTER_GT, INPUT_FIELD_FILTER_GTE, INPUT_FIELD_FILTER_IN,
        INPUT_FIELD_FILTER_IS_NULL, INPUT_FIELD_FILTER_LT, INPUT_FIELD_FILTER_LTE,
        INPUT_FIELD_FILTER_NEQ, INPUT_FIELD_FILTER_NOT, INPUT_FIELD_FILTER_NOT_IN,
    },
    registry::scalars::{DateScalar, DateTimeScalar, IPAddressScalar, TimestampScalar},
    Error,
};

#[derive(Debug, Error)]
pub enum InvalidPagination {
    #[error("The '{0}' and '{0}' parameters are not supported together.")]
    UnsupportedCombination(&'static str, &'static str),
    #[error("Backwards pagination with 'last' without a 'before' cursor is not supported.")]
    UnsupporedBackwardsWithoutCursor,
    #[error("Either 'first' or 'last' must be specified.")]
    MissingDirection,
    #[error("Invalid cursor")]
    InvalidCursor,
}

pub fn parse_pagination(
    first: Option<usize>,
    before: Option<String>,
    last: Option<usize>,
    after: Option<String>,
) -> Result<search::Pagination, InvalidPagination> {
    match (first, after, last, before) {
        (Some(_), _, Some(_), _) => Err(InvalidPagination::UnsupportedCombination("first", "last")),
        (Some(_), _, _, Some(_)) => {
            Err(InvalidPagination::UnsupportedCombination("first", "before"))
        }
        (_, Some(_), Some(_), _) => Err(InvalidPagination::UnsupportedCombination("last", "after")),
        (Some(first), None, None, None) => Ok(search::Pagination::Forward {
            first: first as u64,
            after: None,
        }),
        (Some(first), Some(after), None, None) => Ok(search::Pagination::Forward {
            first: first as u64,
            after: Some(
                search::Cursor::try_from(after).map_err(|_| InvalidPagination::InvalidCursor)?,
            ),
        }),
        (None, None, Some(last), before) => {
            if let Some(before) = before {
                Ok(search::Pagination::Backward {
                    last: last as u64,
                    before: search::Cursor::try_from(before)
                        .map_err(|_| InvalidPagination::InvalidCursor)?,
                })
            } else {
                Err(InvalidPagination::UnsupporedBackwardsWithoutCursor)
            }
        }
        (None, _, None, _) => Err(InvalidPagination::MissingDirection),
    }
}

pub fn parse_filter(schema: &search::Schema, object: Value) -> Result<search::Filter, Error> {
    match object {
        Value::Object(filters) => Ok(search::Filter::All(
            filters
                .into_iter()
                .map(|(name, value)| {
                    (if let Some(field) = schema.fields.get(&name) {
                        parse_scalar_filter(&name, field, value)
                            .map_err(|err| Error::new(format!("Field '{name}': {err:?}")))
                    } else {
                        match name.as_str() {
                            INPUT_FIELD_FILTER_ALL => {
                                parse_filter_array(schema, value).map(search::Filter::All)
                            }
                            INPUT_FIELD_FILTER_ANY => {
                                parse_filter_array(schema, value).map(search::Filter::Any)
                            }
                            INPUT_FIELD_FILTER_NOT => parse_filter(schema, value)
                                .map(|f| search::Filter::Not(Box::new(f))),
                            _ => Err(Error::new("Unknown field".to_string())),
                        }
                    })
                    .map_err(|err| Error::new(format!("Field '{name}': {err:?}")))
                })
                .collect::<Result<_, _>>()?,
        )),
        _ => Err(Error::new("Expected an object of filters")),
    }
}

fn parse_filter_array(schema: &search::Schema, array: Value) -> Result<Vec<search::Filter>, Error> {
    match array {
        Value::Array(filters) => filters
            .into_iter()
            .map(|filters| parse_filter(schema, filters))
            .collect::<Result<Vec<_>, _>>(),
        _ => Err(Error::new("Expected an array of filters")),
    }
}

fn parse_scalar_filter(
    field_name: &str,
    field: &search::FieldEntry,
    filters: Value,
) -> Result<search::Filter, Error> {
    match filters {
        Value::Object(filters) => Ok(search::Filter::All(
            filters
                .into_iter()
                .map(|(name, value)| {
                    use search::Condition::*;
                    let condition = match name.as_str() {
                        INPUT_FIELD_FILTER_EQ => Eq(parse_scalar(field, value)?),
                        INPUT_FIELD_FILTER_NEQ => Neq(parse_scalar(field, value)?),
                        INPUT_FIELD_FILTER_GT => Gt(parse_scalar(field, value)?),
                        INPUT_FIELD_FILTER_GTE => Gte(parse_scalar(field, value)?),
                        INPUT_FIELD_FILTER_LTE => Lte(parse_scalar(field, value)?),
                        INPUT_FIELD_FILTER_LT => Lt(parse_scalar(field, value)?),
                        INPUT_FIELD_FILTER_IN => In(parse_scalar_array(field, value)?),
                        INPUT_FIELD_FILTER_NOT_IN => NotIn(parse_scalar_array(field, value)?),
                        INPUT_FIELD_FILTER_IS_NULL => IsNull(serde_json::from_value(value)?),
                        _ => return Err(Error::new(format!("Unknown field {name}"))),
                    };
                    Ok(search::Filter::FieldFilter {
                        field: field_name.to_string(),
                        condition,
                    })
                })
                .collect::<Result<_, Error>>()?,
        )),

        _ => Err(Error::new("Expected an object of filters")),
    }
}

fn parse_scalar(field: &search::FieldEntry, value: Value) -> Result<search::ScalarValue, Error> {
    use search::FieldType::*;
    use search::ScalarValue;
    Ok(match field.ty {
        URL { .. } => ScalarValue::URL(serde_json::from_value(value)?),
        Email { .. } => ScalarValue::Email(serde_json::from_value(value)?),
        PhoneNumber { .. } => ScalarValue::PhoneNumber(serde_json::from_value(value)?),
        String { .. } => ScalarValue::String(serde_json::from_value(value)?),
        Date { .. } => ScalarValue::Date(DateScalar::parse_value(value)?),
        DateTime { .. } => ScalarValue::DateTime(DateTimeScalar::parse_value(value)?),
        Timestamp { .. } => ScalarValue::Timestamp(TimestampScalar::parse_value(value)?),
        Int { .. } => ScalarValue::Int(serde_json::from_value(value)?),
        Float { .. } => ScalarValue::Float(serde_json::from_value(value)?),
        Boolean { .. } => ScalarValue::Boolean(serde_json::from_value(value)?),
        IPAddress { .. } => ScalarValue::IPAddress(IPAddressScalar::parse_value(value)?),
    })
}

fn parse_scalar_array(
    field: &search::FieldEntry,
    value: Value,
) -> Result<Vec<search::ScalarValue>, Error> {
    Ok(match value {
        Value::Array(values) => values
            .into_iter()
            .map(|value| parse_scalar(field, value))
            .collect::<Result<_, _>>()?,
        _ => vec![parse_scalar(field, value)?],
    })
}
