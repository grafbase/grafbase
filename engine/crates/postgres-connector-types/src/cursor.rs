use std::str::FromStr;

use graphql_cursor::GraphqlCursor;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::error::Error;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum OrderDirection {
    #[default]
    Ascending,
    Descending,
}

impl FromStr for OrderDirection {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let result = match s {
            "DESC" => Self::Descending,
            _ => Self::Ascending,
        };

        Ok(result)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct CursorField {
    pub(super) name: String,
    pub(super) value: Value,
    pub(super) direction: OrderDirection,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SQLCursor {
    fields: Vec<CursorField>,
}

impl SQLCursor {
    pub fn new(row: Map<String, Value>, order_by: Option<&[(String, Option<&'static str>)]>) -> Self {
        let mut fields = Vec::new();

        if let Some(order_by) = order_by {
            for (column, order) in order_by {
                let value = row
                    .get(column)
                    .expect("must select the field we're ordering with")
                    .clone();

                let direction = order.and_then(|order| order.parse().ok()).unwrap_or_default();

                fields.push(CursorField {
                    name: column.clone(),
                    value,
                    direction,
                });
            }
        };

        Self { fields }
    }

    pub fn fields(&self) -> impl ExactSizeIterator<Item = (&str, &Value, OrderDirection)> + '_ {
        self.fields
            .iter()
            .map(|field| (field.name.as_str(), &field.value, field.direction))
    }
}

impl TryFrom<SQLCursor> for GraphqlCursor {
    type Error = Error;

    fn try_from(value: SQLCursor) -> Result<Self, Self::Error> {
        let mut serializer = flexbuffers::FlexbufferSerializer::new();

        value
            .serialize(&mut serializer)
            .map_err(|error| Error::Internal(format!("invalid cursor: {error}")))?;

        Ok(GraphqlCursor::from_bytes(serializer.take_buffer()))
    }
}

impl TryFrom<GraphqlCursor> for SQLCursor {
    type Error = Error;

    fn try_from(value: GraphqlCursor) -> Result<Self, Self::Error> {
        let reader = flexbuffers::Reader::get_root(value.as_slice())
            .map_err(|error| Error::Internal(format!("invalid cursor: {error}")))?;

        Self::deserialize(reader).map_err(|error| Error::Internal(format!("invalid cursor: {error}")))
    }
}
