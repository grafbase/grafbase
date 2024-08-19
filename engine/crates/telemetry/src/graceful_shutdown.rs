pub fn graceful_shutdown() {
    opentelemetry::global::shutdown_tracer_provider();
}
