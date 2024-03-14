#[derive(Debug, PartialEq, Eq)]
pub enum CacheStatus {
    Hit,
    Miss,
    Stale,
    Bypass,
}

impl headers::Header for CacheStatus {
    fn name() -> &'static http::HeaderName {
        &super::X_GRAFBASE_CACHE
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i http::HeaderValue>,
    {
        values
            .filter_map(|value| match value.to_str() {
                Ok("HIT") => Some(CacheStatus::Hit),
                Ok("MISS") => Some(CacheStatus::Miss),
                Ok("STALE") => Some(CacheStatus::Stale),
                Ok("BYPASS") => Some(CacheStatus::Bypass),
                _ => None,
            })
            .last()
            .ok_or_else(headers::Error::invalid)
    }

    fn encode<E: Extend<http::HeaderValue>>(&self, values: &mut E) {
        values.extend(Some(http::HeaderValue::from_static(match self {
            CacheStatus::Hit => "HIT",
            CacheStatus::Miss => "MISS",
            CacheStatus::Stale => "STALE",
            CacheStatus::Bypass => "BYPASS",
        })));
    }
}
