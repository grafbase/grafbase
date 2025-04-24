use std::fmt::{self, Write};

use serde::Deserialize;
use zerocopy::TryFromBytes;

use crate::{SdkError, wit};

use super::{
    ConnectionLike,
    types::{DatabaseType, DatabaseValue},
};

/// A query builder for constructing and executing SQL queries.
///
/// This struct provides a fluent interface for building SQL queries with
/// parameter binding, and methods to execute those queries or fetch their results.
#[derive(Clone, Debug)]
pub struct Query {
    pub(crate) query: String,
    pub(crate) values: Vec<wit::PgBoundValue>,
    pub(crate) value_tree: wit::PgValueTree,
}

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.query)
    }
}

impl Query {
    /// Creates a new `QueryBuilder` instance.
    ///
    /// This is the entry point for constructing a new query using the builder pattern.
    pub fn builder() -> QueryBuilder {
        QueryBuilder::default()
    }

    /// Executes the SQL query with the bound parameters.
    ///
    /// This method sends the query to the database server and returns the number of rows affected.
    /// For INSERT, UPDATE, or DELETE statements, this represents the number of rows modified.
    /// For other statements, the meaning of the return value depends on the specific operation.
    ///
    /// # Returns
    /// The number of rows affected by the query, or an error message if the execution failed
    pub fn execute<'a>(self, connection: impl Into<ConnectionLike<'a>>) -> Result<u64, SdkError> {
        connection
            .into()
            .execute(&self.query, (self.values.as_slice(), &self.value_tree))
    }

    /// Executes the SQL query and fetches all rows from the result.
    ///
    /// This method sends the query to the database and returns all rows in the result set.
    /// It's useful for SELECT queries where you want to process multiple results.
    ///
    /// # Returns
    /// A vector containing all rows in the result set, or an error message if the execution failed
    pub fn fetch<'a>(
        self,
        connection: impl Into<ConnectionLike<'a>>,
    ) -> Result<impl Iterator<Item = ColumnIterator>, SdkError> {
        let rows = connection
            .into()
            .query(&self.query, (self.values.as_slice(), &self.value_tree))?;

        let rows = rows.into_iter().map(|row| ColumnIterator {
            position: 0,
            length: row.len() as usize,
            row,
        });

        Ok(rows)
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
    type Item = Result<RowValue, SdkError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.length {
            let value = match self.row.as_bytes(self.position as u64) {
                Ok(value) => value,
                Err(err) => return Some(Err(SdkError::from(err))),
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

    /// Consumes the `RowValue` and returns the underlying raw binary data.
    ///
    /// This method transfers ownership of the byte vector. If the value
    /// is NULL, it returns `None`.
    ///
    /// # Returns
    /// An owned `Vec<u8>` if the value is not NULL, or `None` if the value is NULL.
    pub fn into_bytes(self) -> Option<Vec<u8>> {
        self.value
    }

    /// This method attempts to interpret the binary data as a UTF-8 encoded string.
    ///
    /// # Returns
    /// * `Ok(Some(str))` if the value is not NULL and was successfully converted to a string
    /// * `Ok(None)` if the value is NULL
    /// * `Err` with a message if the value is not valid UTF-8
    pub fn as_str(&self) -> Result<Option<&str>, SdkError> {
        self.value
            .as_deref()
            .map(|value| {
                std::str::from_utf8(value)
                    .map_err(|e| SdkError::from(format!("Failed to convert bytes to string: {}", e)))
            })
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
    pub fn as_value<T>(&self) -> Result<Option<T>, SdkError>
    where
        T: TryFromBytes,
    {
        self.value
            .as_deref()
            .map(|value| {
                T::try_read_from_bytes(value)
                    .map_err(|e| SdkError::from(format!("Failed to convert bytes to primitive: {e:?}")))
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
    pub fn as_json<T>(&self) -> Result<Option<T>, SdkError>
    where
        T: for<'a> Deserialize<'a>,
    {
        match self.value {
            Some(ref value) => serde_json::from_slice(value).map_err(SdkError::from),
            None => Ok(None),
        }
    }
}

/// A builder for constructing SQL queries with bound parameters.
///
/// This struct facilitates the creation of `Query` objects by allowing
/// the gradual binding of parameters before finalizing the query for execution.
#[derive(Debug, Default)]
pub struct QueryBuilder {
    /// The SQL query string being built.
    query: String,
    /// A list of bound parameter values for the query.
    values: Vec<wit::PgBoundValue>,
    /// A tree structure holding nested values, primarily used for arrays.
    value_tree: wit::PgValueTree,
}

impl QueryBuilder {
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
    pub fn bind(&mut self, value: impl DatabaseType) {
        let value = value.into_bound_value(self.value_tree.len() as u64);
        self.bind_value(value);
    }

    /// Binds a pre-constructed `DatabaseValue` to the query as a parameter.
    ///
    /// This method is similar to `bind()` but accepts a `DatabaseValue` that has already been
    /// created, which can be useful when you need more control over how values are bound.
    ///
    /// # Parameters
    /// * `value` - A `DatabaseValue` instance to bind to the query
    pub fn bind_value(&mut self, value: DatabaseValue) {
        let DatabaseValue {
            value: bound_value,
            array_values,
        } = value;

        let wit::PgBoundValue {
            mut value,
            type_,
            is_array,
        } = bound_value;

        // If the value is an array, adjust the indices based on the current size of the value_tree
        if let wit::PgValue::Array(items) = &mut value {
            let offset = self.value_tree.len() as u64;
            items.iter_mut().for_each(|x| *x += offset);
        }

        // Add the potentially modified value to the list of bound values
        self.values.push(wit::PgBoundValue { value, type_, is_array });

        // If there are associated array values (nested arrays), extend the value_tree
        if let Some(array_values) = array_values {
            self.value_tree.extend(array_values);
        }
    }

    /// Finalizes the query construction process.
    ///
    /// This method takes the built query string and bound parameters from the `QueryBuilder`
    /// and combines them with a database connection or transaction to create a `Query` object.
    /// The returned `Query` object is ready for execution.
    ///
    /// # Parameters
    /// * `connection` - A database connection (`&Connection`) or transaction (`&Transaction`)
    ///   where the query will be executed. This parameter accepts any type that can be converted
    ///   into a `ConnectionLike` enum, typically a reference to a `Connection` or `Transaction`.
    ///
    /// # Returns
    /// A `Query` instance containing the finalized SQL query, bound parameters, and the connection,
    /// ready to be executed or fetched.
    pub fn finalize(self) -> Query {
        let query = self.query;
        let values = self.values;
        let value_tree = self.value_tree;

        Query {
            query,
            values,
            value_tree,
        }
    }

    /// Returns the number of values currently bound to the query builder.
    ///
    /// This can be useful for generating parameter placeholders (e.g., `$1`, `$2`)
    /// dynamically while building the query string.
    pub fn bound_values(&self) -> usize {
        self.values.len()
    }
}

impl Write for QueryBuilder {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.query.write_str(s)
    }
}
