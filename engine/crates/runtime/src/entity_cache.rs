use std::{borrow::Cow, time::Duration};

use futures_util::future::BoxFuture;

/// A simplified cache trait with just enough features to handle entity caching
pub trait EntityCache: Send + Sync {
    fn get<'a>(&'a self, name: &'a str) -> BoxFuture<'a, anyhow::Result<Option<Vec<u8>>>>;

    /// Put an entry into the store, with an optional expiry TTL.
    fn put<'a>(
        &'a self,
        name: &'a str,
        bytes: Cow<'a, [u8]>,
        expiration_ttl: Duration,
    ) -> BoxFuture<'a, anyhow::Result<()>>;
}
