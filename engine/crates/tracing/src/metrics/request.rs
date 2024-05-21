use opentelemetry::{
    metrics::{Counter, Histogram, Meter},
    KeyValue,
};

use crate::grafbase_client::Client;

static X_GRAFBASE_HAS_GRAPHQL_ERRORS: http::HeaderName = http::HeaderName::from_static("x-grafbase-graphql-errors");

pub struct HasGraphqlErrors;

impl HasGraphqlErrors {
    pub fn header_name() -> &'static http::HeaderName {
        &X_GRAFBASE_HAS_GRAPHQL_ERRORS
    }
}

impl headers::Header for HasGraphqlErrors {
    fn name() -> &'static http::HeaderName {
        &X_GRAFBASE_HAS_GRAPHQL_ERRORS
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i http::HeaderValue>,
    {
        values
            .next()
            .map(|_| HasGraphqlErrors)
            .ok_or_else(headers::Error::invalid)
    }

    fn encode<E: Extend<http::HeaderValue>>(&self, values: &mut E) {
        values.extend(Some(http::HeaderValue::from_static("t")))
    }
}

#[derive(Clone)]
pub struct RequestMetrics {
    count: Counter<u64>,
    latency: Histogram<u64>,
}

pub struct RequestMetricsAttributes {
    pub status_code: u16,
    pub cache_status: Option<String>,
    pub has_graphql_errors: bool,
    pub client: Option<Client>,
}

impl RequestMetricsAttributes {
    pub fn server_error() -> RequestMetricsAttributes {
        RequestMetricsAttributes {
            status_code: 500,
            cache_status: None,
            has_graphql_errors: false,
            client: None,
        }
    }
}

impl RequestMetrics {
    pub fn build(meter: &Meter) -> Self {
        Self {
            count: meter.u64_counter("request_count").init(),
            latency: meter.u64_histogram("request_latency").init(),
        }
    }

    pub fn record(
        &self,
        RequestMetricsAttributes {
            status_code,
            cache_status,
            has_graphql_errors,
            client,
        }: RequestMetricsAttributes,
        latency: std::time::Duration,
    ) {
        let mut attributes = vec![KeyValue::new("http.response.status_code", status_code as i64)];
        if let Some(cache_status) = cache_status {
            attributes.push(KeyValue::new("http.response.headers.cache_status", cache_status));
        }
        if let Some(client) = client {
            attributes.push(KeyValue::new("http.headers.x-grafbase-client-name", client.name));
            if let Some(version) = client.version {
                attributes.push(KeyValue::new("http.headers.x-grafbase-client-version", version));
            }
        }
        if has_graphql_errors {
            attributes.push(KeyValue::new("gql.response.has_errors", "true"));
        }
        self.count.add(1, &attributes);
        self.latency.record(latency.as_millis() as u64, &attributes);
    }
}
