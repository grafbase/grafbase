use std::{borrow::Cow, time::Duration};

use bytes::Bytes;
use futures_util::{future::BoxFuture, FutureExt};

/// A simplified cache trait with just enough features to handle entity caching
pub trait EntityCache: Send + Sync {
    fn get<'a>(&'a self, name: &'a str) -> BoxFuture<'a, anyhow::Result<Option<Bytes>>>;

    /// Put an entry into the store, with an optional expiry TTL.
    fn put<'a>(
        &'a self,
        name: &'a str,
        bytes: Cow<'a, [u8]>,
        expiration_ttl: Duration,
    ) -> BoxFuture<'a, anyhow::Result<()>>;
}

impl EntityCache for () {
    fn get<'a>(&'a self, _name: &'a str) -> BoxFuture<'a, anyhow::Result<Option<Bytes>>> {
        futures_util::future::ready(Ok(None)).boxed()
    }

    fn put<'a>(
        &'a self,
        _name: &'a str,
        _bytes: Cow<'a, [u8]>,
        _expiration_ttl: Duration,
    ) -> BoxFuture<'a, anyhow::Result<()>> {
        futures_util::future::ready(Ok(())).boxed()
    }
}
