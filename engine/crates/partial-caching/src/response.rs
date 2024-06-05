use std::time::Duration;

use graph_entities::QueryResponse;
use headers::HeaderMapExt;
use runtime::cache::X_GRAFBASE_CACHE;

pub struct Response {
    pub body: QueryResponse,
    pub headers: http::HeaderMap,
}

impl Response {
    pub(crate) fn hit(body: QueryResponse, max_age: MaxAge) -> Self {
        Response {
            body,
            headers: headers("HIT", max_age),
        }
    }

    pub(crate) fn partial_hit(body: QueryResponse, max_age: MaxAge) -> Self {
        Response {
            body,
            headers: headers("PARTIAL_HIT", max_age),
        }
    }

    pub(crate) fn miss(body: QueryResponse, max_age: MaxAge) -> Self {
        Response {
            body,
            headers: headers("MISS", max_age),
        }
    }
}

fn headers(grafbase_cache: &'static str, max_age: MaxAge) -> http::HeaderMap {
    let mut headers = http::HeaderMap::new();
    headers.insert(
        http::HeaderName::from_static(X_GRAFBASE_CACHE),
        http::HeaderValue::from_static(grafbase_cache),
    );

    if let Some(max_age) = max_age.into_duration() {
        headers.typed_insert(headers::CacheControl::new().with_public().with_max_age(max_age));
    }

    headers
}

/// The maximum age that will be returned in headers
#[derive(Clone, Copy, Debug, Default)]
pub(crate) enum MaxAge {
    #[default]
    /// A max age has not yet been set
    Unknown,

    Known(Duration),

    /// Max age has explicitly been set to none
    None,
}

impl MaxAge {
    pub fn merge(&mut self, other: Duration) {
        // Clamp to seconds, because that's what the maxAge header is measured in
        let other = Duration::from_secs(other.as_secs());

        *self = match self {
            MaxAge::Unknown => MaxAge::Known(other),
            MaxAge::Known(this) => MaxAge::Known(std::cmp::min(*this, other)),
            MaxAge::None => MaxAge::None,
        };
    }

    /// This can be used to ensure no max age will be sent
    pub fn set_none(&mut self) {
        *self = MaxAge::None;
    }

    fn into_duration(self) -> Option<Duration> {
        match self {
            MaxAge::Unknown | MaxAge::None => None,
            MaxAge::Known(duration) if duration.is_zero() => None,
            MaxAge::Known(duration) => Some(duration),
        }
    }
}
