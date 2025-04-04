use serde::de::DeserializeOwned;
use zerocopy::TryFromBytes;

use crate::wit;

use super::{
    Connection, Transaction,
    types::{DatabaseType, DatabaseValue},
};

#[derive(Clone, Copy, Debug)]
pub(crate) enum ConnectionLike<'a> {
    Connection(&'a Connection),
    Transaction(&'a Transaction),
}

impl<'a> From<&'a Connection> for ConnectionLike<'a> {
    fn from(connection: &'a Connection) -> Self {
        ConnectionLike::Connection(connection)
    }
}

impl<'a> From<&'a Transaction> for ConnectionLike<'a> {
    fn from(transaction: &'a Transaction) -> Self {
        ConnectionLike::Transaction(transaction)
    }
}

impl ConnectionLike<'_> {
    pub fn query<'a>(
        &'a self,
        query: &'a str,
        params: (&[wit::PgBoundValue], &[wit::PgValue]),
    ) -> Result<Vec<wit::PgRow>, String> {
        match self {
            ConnectionLike::Connection(connection) => connection.0.query(query, params),
            ConnectionLike::Transaction(transaction) => transaction.inner.query(query, params),
        }
    }

    pub fn execute<'a>(
        &'a self,
        query: &'a str,
        params: (&[wit::PgBoundValue], &[wit::PgValue]),
    ) -> Result<u64, String> {
        match self {
            ConnectionLike::Connection(connection) => connection.0.execute(query, params),
            ConnectionLike::Transaction(transaction) => transaction.inner.execute(query, params),
        }
    }
}

/// A query builder for constructing and executing SQL queries.
///
/// This struct provides a fluent interface for building SQL queries with
/// parameter binding, and methods to execute those queries or fetch their results.
#[derive(Clone, Debug)]
pub struct Query<'a> {
    pub(crate) connection: ConnectionLike<'a>,
    pub(crate) query: &'a str,
    pub(crate) values: Vec<wit::PgBoundValue>,
    pub(crate) value_tree: wit::PgValueTree,
}

impl Query<'_> {
    /// Binds a value to the query as a parameter.
    ///
    /// This method adds a value to be used as a parameter in the SQL query.
    /// The value will be properly escaped and converted to the appropriate PostgreSQL type.
    ///
    /// # Parameters
    /// * `value` - Any value that implements the `DatabaseType` trait
    ///
    /// # Returns
    /// The query builder for method chaining
    pub fn bind(mut self, value: impl DatabaseType) -> Self {
        let DatabaseValue { value, array_values } = value.into_bound_value();

        self.values.push(value);

        if let Some(array_values) = array_values {
            self.value_tree.extend(array_values);
        }

        self
    }

    /// Executes the SQL query with the bound parameters.
    ///
    /// This method sends the query to the database server and returns the number of rows affected.
    /// For INSERT, UPDATE, or DELETE statements, this represents the number of rows modified.
    /// For other statements, the meaning of the return value depends on the specific operation.
    ///
    /// # Returns
    /// The number of rows affected by the query, or an error message if the execution failed
    pub fn execute(self) -> Result<u64, String> {
        self.connection
            .execute(self.query, (self.values.as_slice(), &self.value_tree))
    }

    /// Executes the SQL query and fetches all rows from the result.
    ///
    /// This method sends the query to the database and returns all rows in the result set.
    /// It's useful for SELECT queries where you want to process multiple results.
    ///
    /// # Returns
    /// A vector containing all rows in the result set, or an error message if the execution failed
    pub fn fetch(self) -> Result<impl IntoIterator<Item = ColumnIterator>, String> {
        match self
            .connection
            .query(self.query, (self.values.as_slice(), &self.value_tree))
        {
            Ok(rows) => {
                let rows = rows.into_iter().map(|row| ColumnIterator {
                    position: 0,
                    length: row.len() as usize,
                    row,
                });

                Ok(rows)
            }
            Err(e) => Err(e.to_string()),
        }
    }
}

/// An iterator over the columns in a database row.
///
/// This iterator yields each column value in the row as a `RowValue`
/// which contains both the column name and the value data.
pub struct ColumnIterator {
    position: usize,
    length: usize,
    row: wit::PgRow,
}

impl Iterator for ColumnIterator {
    type Item = Result<RowValue, String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.length {
            let value = match self.row.as_bytes(self.position as u64) {
                Ok(value) => value,
                Err(err) => return Some(Err(err)),
            };

            self.position += 1;

            Some(Ok(RowValue { value }))
        } else {
            None
        }
    }
}

/// A value from a database row.
///
/// This struct represents a single column value from a database row. It includes
/// both the column name and the raw binary data for the value, which can be
/// accessed or converted to other types as needed.
pub struct RowValue {
    value: Option<Vec<u8>>,
}

impl RowValue {
    /// Returns the raw binary data for this value.
    ///
    /// # Returns
    /// A slice of bytes if the value is not NULL, or None if the value is NULL.
    pub fn bytes(&self) -> Option<&[u8]> {
        self.value.as_deref()
    }

    /// This method attempts to interpret the binary data as a UTF-8 encoded string.
    ///
    /// # Returns
    /// * `Ok(Some(str))` if the value is not NULL and was successfully converted to a string
    /// * `Ok(None)` if the value is NULL
    /// * `Err` with a message if the value is not valid UTF-8
    pub fn as_str(&self) -> Result<Option<&str>, String> {
        self.value
            .as_deref()
            .map(|value| std::str::from_utf8(value).map_err(|e| format!("Failed to convert bytes to string: {}", e)))
            .transpose()
    }

    /// Converts the binary data to a value that implements `TryFromBytes`.
    ///
    /// This method attempts to interpret the binary data as a value of the specified type.
    /// It's particularly useful for converting database values to Rust values.
    ///
    /// # Type Parameters
    /// * `T` - The type to convert the binary data to, must implement `TryFromBytes`
    ///
    /// # Returns
    /// * `Ok(Some(T))` if the value is not NULL and was successfully converted
    /// * `Ok(None)` if the value is NULL
    /// * `Err` with a message if conversion failed
    pub fn as_value<T>(&self) -> Result<Option<T>, String>
    where
        T: TryFromBytes,
    {
        self.value
            .as_deref()
            .map(|value| {
                T::try_read_from_bytes(value).map_err(|e| format!("Failed to convert bytes to primitive: {e:?}"))
            })
            .transpose()
    }

    /// Deserializes this value as JSON into the specified type.
    ///
    /// This method parses the binary data as JSON and converts it to the
    /// requested type.
    ///
    /// # Type Parameters
    /// * `T` - The type to deserialize the JSON into
    ///
    /// # Returns
    /// * `Ok(Some(T))` if the value is not NULL and was successfully deserialized
    /// * `Ok(None)` if the value is NULL
    /// * `Err` with a message if deserialization failed
    pub fn as_json<T>(&self) -> Result<Option<T>, String>
    where
        T: DeserializeOwned,
    {
        match self.value {
            Some(ref value) => serde_json::from_slice(value).map_err(|e| format!("Failed to deserialize JSON: {}", e)),
            None => Ok(None),
        }
    }
}
