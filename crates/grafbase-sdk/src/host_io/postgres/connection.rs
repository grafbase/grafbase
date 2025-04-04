use std::fmt;

use crate::{SdkError, wit};

use super::Query;

/// A Postgres database connection.
///
/// This represents a single connection to the Postgres database,
/// which can be used to execute queries or perform other database operations.
pub struct Connection(pub(crate) wit::PgConnection);

impl fmt::Debug for Connection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Connection { ... }")
    }
}

impl From<wit::PgConnection> for Connection {
    fn from(conn: wit::PgConnection) -> Self {
        Self(conn)
    }
}

impl Connection {
    /// Creates a new query with the given SQL statement.
    ///
    /// This method initializes a query builder for the specified SQL statement,
    /// allowing parameters to be bound and the query to be executed on this connection.
    ///
    /// # Parameters
    /// * `query` - The SQL query string to execute
    ///
    /// # Returns
    /// A query builder that can be used to bind parameters and execute the query
    pub fn query<'a>(&'a self, query: &'a str) -> Query<'a> {
        Query {
            connection: self.into(),
            query,
            values: Vec::new(),
            value_tree: wit::PgValueTree::new(),
        }
    }
}

/// A Postgres database transaction.
///
/// This represents an active database transaction that can be either committed
/// to make changes permanent, or rolled back to discard changes.
///
/// The transaction must be either commited or rolled back, otherwise it will be
/// automatically rolled back when dropped.
pub struct Transaction {
    pub(crate) inner: wit::PgTransaction,
    committed_or_rolled_back: bool,
}

impl From<wit::PgTransaction> for Transaction {
    fn from(inner: wit::PgTransaction) -> Self {
        Self {
            inner,
            committed_or_rolled_back: false,
        }
    }
}

impl fmt::Debug for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Transaction { ... }")
    }
}

impl Transaction {
    /// Creates a new query with the given SQL statement.
    ///
    /// This method initializes a query builder for the specified SQL statement,
    /// allowing parameters to be bound and the query to be executed on this connection.
    ///
    /// # Parameters
    /// * `query` - The SQL query string to execute
    ///
    /// # Returns
    /// A query builder that can be used to bind parameters and execute the query
    pub fn query<'a>(&'a self, query: &'a str) -> Query<'a> {
        Query {
            connection: self.into(),
            query,
            values: Vec::new(),
            value_tree: wit::PgValueTree::new(),
        }
    }

    /// Commits the transaction, making all changes permanent.
    ///
    /// # Returns
    /// `Ok(())` if the transaction was successfully committed, or an error message
    pub fn commit(mut self) -> Result<(), SdkError> {
        self.committed_or_rolled_back = true;

        self.inner.commit().map_err(SdkError::from)
    }

    /// Rolls back the transaction, discarding all changes.
    ///
    /// # Returns
    /// `Ok(())` if the transaction was successfully rolled back, or an error message
    pub fn rollback(mut self) -> Result<(), SdkError> {
        self.committed_or_rolled_back = true;

        self.inner.rollback().map_err(SdkError::from)
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        if !self.committed_or_rolled_back {
            self.inner.rollback().unwrap()
        }
    }
}
