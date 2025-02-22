use crate::{AccessLogReceiver, AccessLogSender, resources::SharedResources};

mod extensions;
mod hooks;

fn create_test_access_log() -> (AccessLogSender, AccessLogReceiver) {
    crate::create_access_log_channel(
        false,
        grafbase_telemetry::metrics::meter_from_global_provider()
            .i64_up_down_counter("grafbase.gateway.access_log.pending")
            .build(),
    )
}

fn create_shared_resources() -> (SharedResources, AccessLogReceiver) {
    let (access_log, access_log_receiver) = create_test_access_log();
    (SharedResources { access_log }, access_log_receiver)
}
