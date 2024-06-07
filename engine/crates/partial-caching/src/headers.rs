pub struct RequestCacheControl {
    /// Whether the cache code should use the cache to provide responses
    ///
    /// Controlled by the `no-cache` directive in headers
    ///
    /// The MDN description of `no-cache` seems to say that you can fetch
    /// the cache but need to "revalidate" when this directive is present.
    /// But we currently have no revalidation mechanism so this seems the
    /// closest option
    pub should_read_from_cache: bool,

    /// Whether the cache code should store responses in the cache
    ///
    /// Controlled by the `no-store` directive in headers
    pub should_write_to_cache: bool,
}

impl From<headers::CacheControl> for RequestCacheControl {
    fn from(value: headers::CacheControl) -> Self {
        RequestCacheControl {
            should_read_from_cache: !value.no_cache(),
            should_write_to_cache: !value.no_store(),
        }
    }
}
