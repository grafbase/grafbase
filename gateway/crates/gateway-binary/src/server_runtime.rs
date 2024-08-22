use crate::telemetry::OpenTelemetryProviders;

#[cfg_attr(feature = "lambda", allow(unused))]
#[derive(Clone)]
struct StandardRuntime {
    telemetry: OpenTelemetryProviders,
}

impl federated_server::ServerRuntime for StandardRuntime {
    async fn graceful_shutdown(&self) {
        use grafbase_telemetry::otel::opentelemetry::global::{shutdown_logger_provider, shutdown_tracer_provider};
        use tokio::task::spawn_blocking;

        let _ = tokio::join!(
            spawn_blocking(shutdown_tracer_provider),
            spawn_blocking(shutdown_logger_provider),
            async {
                if let Some(provider) = &self.telemetry.meter {
                    let _ = provider.shutdown().await;
                }
            }
        );
    }

    fn after_request(&self) {}
}

#[cfg_attr(not(feature = "lambda"), allow(unused))]
#[derive(Clone)]
struct LambdaRuntime {
    telemetry: OpenTelemetryProviders,
}

impl federated_server::ServerRuntime for LambdaRuntime {
    async fn graceful_shutdown(&self) {}

    fn after_request(&self) {
        // lambda must flush the trace events here, otherwise the
        // function might fall asleep and the events are pending until
        // the next wake-up.
        //
        // read more: https://github.com/open-telemetry/opentelemetry-lambda/blob/main/docs/design_proposal.md
        if let Some(ref tracer_provider) = self.telemetry.tracer {
            for result in tracer_provider.force_flush() {
                if let Err(e) = result {
                    println!("error flushing events: {e}");
                }
            }
        }
    }
}

pub(crate) fn build(telemetry: OpenTelemetryProviders) -> impl federated_server::ServerRuntime {
    cfg_if::cfg_if! {
        if #[cfg(feature = "lambda")] {
            LambdaRuntime { telemetry }
        } else {
            StandardRuntime { telemetry }
        }
    }
}
