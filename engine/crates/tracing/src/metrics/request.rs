use opentelemetry::{
    metrics::{Counter, Histogram, Meter},
    KeyValue,
};

#[derive(Clone)]
pub struct RequestMetrics {
    count: Counter<u64>,
    latency: Histogram<u64>,
}

pub struct RequestMetricsAttributes {
    pub status_code: u16,
    pub cache_status: Option<String>,
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
        }: RequestMetricsAttributes,
        latency: std::time::Duration,
    ) {
        let mut attributes = vec![KeyValue::new("http.response.status_code", status_code as i64)];
        if let Some(cache_status) = cache_status {
            attributes.push(KeyValue::new("http.response.headers.cache_status", cache_status));
        }
        self.count.add(1, &attributes);
        self.latency.record(latency.as_millis() as u64, &[]);
    }
}
