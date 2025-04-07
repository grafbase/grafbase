use std::fmt;

use crate::{SdkError, wit};

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

/// Represents either a [`Connection`] or a [`Transaction`].
///
/// This enum is used in functions that can operate on either a regular
/// connection or within an active transaction, allowing for flexibility
/// in how database operations are performed.
#[derive(Clone, Copy, Debug)]
pub enum ConnectionLike<'a> {
    /// A regular database connection, not part of an explicit transaction.
    Connection(&'a Connection),
    /// An active database transaction. Operations performed using this variant
    /// will be part of the ongoing transaction.
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
    pub(crate) fn query<'a>(
        &'a self,
        query: &'a str,
        params: (&[wit::PgBoundValue], &[wit::PgValue]),
    ) -> Result<Vec<wit::PgRow>, SdkError> {
        match self {
            ConnectionLike::Connection(connection) => connection.0.query(query, params).map_err(SdkError::from),
            ConnectionLike::Transaction(transaction) => transaction.inner.query(query, params).map_err(SdkError::from),
        }
    }

    pub(crate) fn execute<'a>(
        &'a self,
        query: &'a str,
        params: (&[wit::PgBoundValue], &[wit::PgValue]),
    ) -> Result<u64, SdkError> {
        match self {
            ConnectionLike::Connection(connection) => connection.0.execute(query, params).map_err(SdkError::from),
            ConnectionLike::Transaction(transaction) => {
                transaction.inner.execute(query, params).map_err(SdkError::from)
            }
        }
    }
}
