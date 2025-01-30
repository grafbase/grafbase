use crate::{ChannelLogReceiver, ChannelLogSender};

mod extensions;
mod hooks;

fn create_log_channel() -> (ChannelLogSender, ChannelLogReceiver) {
    crate::create_log_channel(
        false,
        grafbase_telemetry::metrics::meter_from_global_provider()
            .i64_up_down_counter("grafbase.gateway.access_log.pending")
            .build(),
    )
}
