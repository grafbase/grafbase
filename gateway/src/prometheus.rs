use prometheus::{
    register_gauge, register_histogram, register_int_counter, register_int_gauge,
    Gauge, Histogram, IntCounter, IntGauge,
};
use std::sync::Once;

// Define metrics
static INIT: Once = Once::new();

// Request metrics
pub static mut HTTP_REQUESTS_TOTAL: Option<IntCounter> = None;
pub static mut HTTP_REQUEST_DURATION_SECONDS: Option<Histogram> = None;
pub static mut ACTIVE_CONNECTIONS: Option<IntGauge> = None;

// GraphQL specific metrics
pub static mut GRAPHQL_QUERIES_TOTAL: Option<IntCounter> = None;
pub static mut GRAPHQL_MUTATIONS_TOTAL: Option<IntCounter> = None;
pub static mut GRAPHQL_QUERY_DURATION_SECONDS: Option<Histogram> = None;
pub static mut GRAPHQL_ERRORS_TOTAL: Option<IntCounter> = None;

// System metrics
pub static mut MEMORY_USAGE_BYTES: Option<Gauge> = None;
pub static mut CPU_USAGE_PERCENT: Option<Gauge> = None;

// Federation specific metrics
pub static mut FEDERATION_SUBGRAPH_REQUESTS_TOTAL: Option<IntCounter> = None;
pub static mut FEDERATION_SUBGRAPH_REQUEST_DURATION_SECONDS: Option<Histogram> = None;
pub static mut FEDERATION_SUBGRAPH_ERRORS_TOTAL: Option<IntCounter> = None;

pub fn init_metrics() {
    INIT.call_once(|| {
        // Safety: This is only called once during initialization
        unsafe {
            // HTTP metrics
            HTTP_REQUESTS_TOTAL = Some(
                register_int_counter!("grafbase_gateway_http_requests_total", "Total number of HTTP requests").unwrap(),
            );
            HTTP_REQUEST_DURATION_SECONDS = Some(
                register_histogram!("grafbase_gateway_http_request_duration_seconds", "HTTP request duration in seconds")
                    .unwrap(),
            );
            ACTIVE_CONNECTIONS = Some(
                register_int_gauge!("grafbase_gateway_active_connections", "Number of active connections").unwrap(),
            );

            // GraphQL metrics
            GRAPHQL_QUERIES_TOTAL = Some(
                register_int_counter!("grafbase_gateway_graphql_queries_total", "Total number of GraphQL queries")
                    .unwrap(),
            );
            GRAPHQL_MUTATIONS_TOTAL = Some(
                register_int_counter!("grafbase_gateway_graphql_mutations_total", "Total number of GraphQL mutations")
                    .unwrap(),
            );
            GRAPHQL_QUERY_DURATION_SECONDS = Some(
                register_histogram!(
                    "grafbase_gateway_graphql_query_duration_seconds",
                    "GraphQL query execution time in seconds"
                )
                .unwrap(),
            );
            GRAPHQL_ERRORS_TOTAL = Some(
                register_int_counter!("grafbase_gateway_graphql_errors_total", "Total number of GraphQL errors")
                    .unwrap(),
            );

            // System metrics
            MEMORY_USAGE_BYTES = Some(
                register_gauge!("grafbase_gateway_memory_usage_bytes", "Memory usage in bytes").unwrap(),
            );
            CPU_USAGE_PERCENT = Some(register_gauge!("grafbase_gateway_cpu_usage_percent", "CPU usage percentage").unwrap());

            // Federation metrics
            FEDERATION_SUBGRAPH_REQUESTS_TOTAL = Some(
                register_int_counter!(
                    "grafbase_gateway_federation_subgraph_requests_total",
                    "Total number of federation subgraph requests"
                )
                .unwrap(),
            );
            FEDERATION_SUBGRAPH_REQUEST_DURATION_SECONDS = Some(
                register_histogram!(
                    "grafbase_gateway_federation_subgraph_request_duration_seconds",
                    "Federation subgraph request duration in seconds"
                )
                .unwrap(),
            );
            FEDERATION_SUBGRAPH_ERRORS_TOTAL = Some(
                register_int_counter!(
                    "grafbase_gateway_federation_subgraph_errors_total",
                    "Total number of federation subgraph errors"
                )
                .unwrap(),
            );
        }
    });
}

// Helper functions to safely increment/observe metrics
pub fn increment_http_requests() {
    unsafe {
        if let Some(counter) = &HTTP_REQUESTS_TOTAL {
            counter.inc();
        }
    }
}

pub fn observe_http_request_duration(duration_secs: f64) {
    unsafe {
        if let Some(histogram) = &HTTP_REQUEST_DURATION_SECONDS {
            histogram.observe(duration_secs);
        }
    }
}

pub fn set_active_connections(count: i64) {
    unsafe {
        if let Some(gauge) = &ACTIVE_CONNECTIONS {
            gauge.set(count);
        }
    }
}

pub fn increment_graphql_queries() {
    unsafe {
        if let Some(counter) = &GRAPHQL_QUERIES_TOTAL {
            counter.inc();
        }
    }
}

pub fn increment_graphql_mutations() {
    unsafe {
        if let Some(counter) = &GRAPHQL_MUTATIONS_TOTAL {
            counter.inc();
        }
    }
}

pub fn observe_graphql_query_duration(duration_secs: f64) {
    unsafe {
        if let Some(histogram) = &GRAPHQL_QUERY_DURATION_SECONDS {
            histogram.observe(duration_secs);
        }
    }
}

pub fn increment_graphql_errors() {
    unsafe {
        if let Some(counter) = &GRAPHQL_ERRORS_TOTAL {
            counter.inc();
        }
    }
}

pub fn set_memory_usage(bytes: f64) {
    unsafe {
        if let Some(gauge) = &MEMORY_USAGE_BYTES {
            gauge.set(bytes);
        }
    }
}

pub fn set_cpu_usage(percent: f64) {
    unsafe {
        if let Some(gauge) = &CPU_USAGE_PERCENT {
            gauge.set(percent);
        }
    }
}

pub fn increment_federation_subgraph_requests() {
    unsafe {
        if let Some(counter) = &FEDERATION_SUBGRAPH_REQUESTS_TOTAL {
            counter.inc();
        }
    }
}

pub fn observe_federation_subgraph_request_duration(duration_secs: f64) {
    unsafe {
        if let Some(histogram) = &FEDERATION_SUBGRAPH_REQUEST_DURATION_SECONDS {
            histogram.observe(duration_secs);
        }
    }
}

pub fn increment_federation_subgraph_errors() {
    unsafe {
        if let Some(counter) = &FEDERATION_SUBGRAPH_ERRORS_TOTAL {
            counter.inc();
        }
    }
}