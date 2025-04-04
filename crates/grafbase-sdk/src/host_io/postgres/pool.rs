use std::time::Duration;

use crate::wit;

use super::{Connection, Transaction};

/// Configuration options for a Postgres connection pool.
///
/// This struct provides various configuration options for controlling the behavior
/// of a Postgres connection pool, such as connection limits and timeouts.
pub struct PoolOptions(wit::PgPoolOptions);

impl Default for PoolOptions {
    fn default() -> Self {
        Self(wit::PgPoolOptions {
            max_connections: None,
            min_connections: None,
            idle_timeout_ms: None,
            acquisition_timeout_ms: None,
            max_lifetime_ms: None,
        })
    }
}

impl PoolOptions {
    /// Creates a new `PoolOptions` instance with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum number of connections in the pool.
    ///
    /// This limits how many database connections the pool can open simultaneously.
    pub fn max_connections(mut self, max_connections: u32) -> Self {
        self.0.max_connections = Some(max_connections);
        self
    }

    /// Sets the minimum number of connections the pool will maintain.
    ///
    /// The pool will attempt to maintain this many idle connections at all times.
    pub fn min_connections(mut self, min_connections: u32) -> Self {
        self.0.min_connections = Some(min_connections);
        self
    }

    /// Sets the idle timeout for connections in the pool.
    ///
    /// Connections that remain idle for longer than this duration may be closed.
    pub fn idle_timeout(mut self, idle_timeout: Duration) -> Self {
        self.0.idle_timeout_ms = Some(idle_timeout.as_millis() as u64);
        self
    }

    /// Sets the maximum time to wait when acquiring a connection from the pool.
    ///
    /// If a connection cannot be acquired within this time, the acquisition attempt fails.
    pub fn acquire_timeout(mut self, acquire_timeout: Duration) -> Self {
        self.0.acquisition_timeout_ms = Some(acquire_timeout.as_millis() as u64);
        self
    }

    /// Sets the maximum lifetime of connections in the pool.
    ///
    /// Connections will be closed after this duration regardless of whether they're idle.
    pub fn max_lifetime(mut self, max_lifetime: Duration) -> Self {
        self.0.max_lifetime_ms = Some(max_lifetime.as_millis() as u64);
        self
    }
}

/// A Postgres connection pool.
///
/// This pool manages a set of database connections that can be reused across
/// different operations, improving performance by avoiding the overhead of
/// establishing new connections for each database operation.
pub struct Pool(wit::PgPool);

impl Pool {
    /// Creates a new connection pool with default options.
    ///
    /// # Parameters
    /// * `identifier` - A unique identifier for the pool
    /// * `url` - The Postgres connection URL
    ///
    /// # Returns
    /// A new connection pool or an error if the connection fails
    pub fn connect(identifier: &str, url: &str) -> Result<Self, String> {
        Self::connect_with_options(identifier, url, Default::default())
    }

    /// Creates a new connection pool with custom options.
    ///
    /// # Parameters
    /// * `identifier` - A unique identifier for the pool
    /// * `url` - The Postgres connection URL
    /// * `options` - Configuration options for the connection pool
    ///
    /// # Returns
    /// A new connection pool or an error if the connection fails
    pub fn connect_with_options(identifier: &str, url: &str, options: PoolOptions) -> Result<Self, String> {
        let pool = wit::PgPool::connect(identifier, url, options.0)
            .map_err(|e| format!("Failed to connect to Postgres: {}", e))?;

        Ok(Self(pool))
    }

    /// Acquires a connection from the pool.
    ///
    /// This function will wait until a connection is available or until
    /// the configured acquisition timeout is reached.
    ///
    /// # Returns
    /// A database connection or an error if no connection could be acquired
    pub fn acquire(&self) -> Result<Connection, String> {
        self.0
            .acquire()
            .map(Into::into)
            .map_err(|e| format!("Failed to acquire connection: {}", e))
    }

    /// Begins a new database transaction.
    ///
    /// This is a convenience method that acquires a connection and starts
    /// a transaction on it.
    ///
    /// # Returns
    /// A transaction object or an error if the transaction couldn't be started
    pub fn begin_transaction(&self) -> Result<Transaction, String> {
        self.0
            .begin_transaction()
            .map(Into::into)
            .map_err(|e| format!("Failed to begin transaction: {}", e))
    }
}
