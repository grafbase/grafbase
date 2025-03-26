use prometheus::{
    Encoder, Gauge, Histogram, IntCounter, IntGauge, register_gauge, register_histogram, register_int_counter,
    register_int_gauge,
};
use std::io::Read;
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
                register_histogram!(
                    "grafbase_gateway_http_request_duration_seconds",
                    "HTTP request duration in seconds"
                )
                .unwrap(),
            );
            ACTIVE_CONNECTIONS = Some(
                register_int_gauge!("grafbase_gateway_active_connections", "Number of active connections").unwrap(),
            );

            // GraphQL metrics
            GRAPHQL_QUERIES_TOTAL = Some(
                register_int_counter!(
                    "grafbase_gateway_graphql_queries_total",
                    "Total number of GraphQL queries"
                )
                .unwrap(),
            );
            GRAPHQL_MUTATIONS_TOTAL = Some(
                register_int_counter!(
                    "grafbase_gateway_graphql_mutations_total",
                    "Total number of GraphQL mutations"
                )
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
                register_int_counter!(
                    "grafbase_gateway_graphql_errors_total",
                    "Total number of GraphQL errors"
                )
                .unwrap(),
            );

            // System metrics
            MEMORY_USAGE_BYTES =
                Some(register_gauge!("grafbase_gateway_memory_usage_bytes", "Memory usage in bytes").unwrap());
            CPU_USAGE_PERCENT =
                Some(register_gauge!("grafbase_gateway_cpu_usage_percent", "CPU usage percentage").unwrap());

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
#[allow(unused)]
pub fn increment_http_requests() {
    unsafe {
        if let Some(ref counter) = HTTP_REQUESTS_TOTAL {
            counter.inc();
        }
    }
}

#[allow(unused)]
pub fn observe_http_request_duration(duration_secs: f64) {
    unsafe {
        if let Some(ref histogram) = HTTP_REQUEST_DURATION_SECONDS {
            histogram.observe(duration_secs);
        }
    }
}

#[allow(unused)]
pub fn set_active_connections(count: i64) {
    unsafe {
        if let Some(ref gauge) = ACTIVE_CONNECTIONS {
            gauge.set(count);
        }
    }
}

#[allow(unused)]
pub fn increment_graphql_queries() {
    unsafe {
        if let Some(ref counter) = GRAPHQL_QUERIES_TOTAL {
            counter.inc();
        }
    }
}

#[allow(unused)]
pub fn increment_graphql_mutations() {
    unsafe {
        if let Some(ref counter) = GRAPHQL_MUTATIONS_TOTAL {
            counter.inc();
        }
    }
}

#[allow(unused)]
pub fn observe_graphql_query_duration(duration_secs: f64) {
    unsafe {
        if let Some(ref histogram) = GRAPHQL_QUERY_DURATION_SECONDS {
            histogram.observe(duration_secs);
        }
    }
}

#[allow(unused)]
pub fn increment_graphql_errors() {
    unsafe {
        if let Some(ref counter) = GRAPHQL_ERRORS_TOTAL {
            counter.inc();
        }
    }
}

#[allow(unused)]
pub fn set_memory_usage(bytes: f64) {
    unsafe {
        if let Some(ref gauge) = MEMORY_USAGE_BYTES {
            gauge.set(bytes);
        }
    }
}

#[allow(unused)]
pub fn set_cpu_usage(percent: f64) {
    unsafe {
        if let Some(ref gauge) = CPU_USAGE_PERCENT {
            gauge.set(percent);
        }
    }
}

#[allow(unused)]
pub fn increment_federation_subgraph_requests() {
    unsafe {
        if let Some(ref counter) = FEDERATION_SUBGRAPH_REQUESTS_TOTAL {
            counter.inc();
        }
    }
}

#[allow(unused)]
pub fn observe_federation_subgraph_request_duration(duration_secs: f64) {
    unsafe {
        if let Some(ref histogram) = FEDERATION_SUBGRAPH_REQUEST_DURATION_SECONDS {
            histogram.observe(duration_secs);
        }
    }
}

#[allow(unused)]
pub fn increment_federation_subgraph_errors() {
    unsafe {
        if let Some(ref counter) = FEDERATION_SUBGRAPH_ERRORS_TOTAL {
            counter.inc();
        }
    }
}

/// Start a Prometheus metrics server if enabled
pub fn maybe_start_metrics_server(config: &gateway_config::TelemetryConfig) {
    if let Some(metrics_config) = &config.metrics {
        if let Some(prometheus_config) = &metrics_config.prometheus {
            if prometheus_config.enabled {
                let addr = prometheus_config.listen_address.unwrap_or_else(|| {
                    // Default to 0.0.0.0:9090 if not specified
                    std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)), 9090)
                });
                start_metrics_server(addr);
            }
        }
    }
}

fn start_metrics_server(addr: std::net::SocketAddr) {
    std::thread::spawn(move || {
        let metrics_server = std::net::TcpListener::bind(addr).expect("Failed to bind metrics server");
        tracing::info!(
            "Prometheus metrics available at http://{}:{}/metrics",
            addr.ip(),
            addr.port()
        );

        for stream in metrics_server.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut buffer = [0u8; 4096];
                    let bytes_read = match stream.read(&mut buffer) {
                        Ok(n) => n,
                        Err(e) => {
                            tracing::error!("Error reading from HTTP stream: {}", e);
                            continue;
                        }
                    };

                    if bytes_read == 0 {
                        continue;
                    }

                    let request = match std::str::from_utf8(&buffer[0..bytes_read]) {
                        Ok(s) => s,
                        Err(e) => {
                            tracing::error!("Error parsing HTTP request: {}", e);
                            continue;
                        }
                    };

                    // Check if this is a GET request for /metrics
                    if !request.starts_with("GET /metrics") && !request.starts_with("GET / ") {
                        let response = "HTTP/1.1 404 Not Found\r\n\r\nNot Found";
                        if let Err(e) = std::io::Write::write_all(&mut stream, response.as_bytes()) {
                            tracing::error!("Error writing 404 response: {}", e);
                        }
                        continue;
                    }

                    let encoder = prometheus::TextEncoder::new();
                    let metric_families = prometheus::gather();
                    let mut output_buffer = Vec::new();
                    if let Err(e) = encoder.encode(&metric_families, &mut output_buffer) {
                        tracing::error!("Failed to encode metrics: {}", e);
                        continue;
                    }

                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
                        encoder.format_type(),
                        output_buffer.len(),
                        String::from_utf8(output_buffer).expect("Failed to convert metrics to string")
                    );

                    if let Err(e) = std::io::Write::write_all(&mut stream, response.as_bytes()) {
                        tracing::error!("Error writing metrics response: {}", e);
                    }
                }
                Err(e) => tracing::error!("Error accepting metrics connection: {}", e),
            }
        }
    });
}
