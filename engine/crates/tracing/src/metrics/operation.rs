use opentelemetry::metrics::{Counter, Histogram, Meter};

#[derive(Clone)]
pub struct GraphqlOperationMetrics {
    count: Counter<u64>,
    latency: Histogram<u64>,
}

pub struct GraphqlOperationMetricsAttributes {
    pub id: String,
    pub ty: &'static str,
    pub name: Option<String>,
}

impl GraphqlOperationMetrics {
    pub fn build(meter: &Meter) -> Self {
        Self {
            count: meter.u64_counter("gql_operation_count").init(),
            latency: meter.u64_histogram("gql_operation_latency").init(),
        }
    }

    pub fn record(
        &self,
        GraphqlOperationMetricsAttributes { id, name, .. }: GraphqlOperationMetricsAttributes,
        latency: std::time::Duration,
    ) {
        let attributes = vec![
            opentelemetry::KeyValue::new("gql.operation.id", id),
            opentelemetry::KeyValue::new("gql.operation.name", name.unwrap_or_default()),
        ];
        self.count.add(1, &attributes);
        self.latency.record(latency.as_millis() as u64, &attributes);
    }
}
