use crate::telemetry::OpenTelemetryProviders;

#[cfg_attr(not(feature = "lambda"), allow(unused))]
#[derive(Clone)]
struct LambdaRuntime {
    telemetry: OpenTelemetryProviders,
}

impl federated_server::ServerRuntime for LambdaRuntime {
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

#[cfg_attr(not(feature = "lambda"), allow(unused_variables))]
pub(crate) fn build(telemetry: OpenTelemetryProviders) -> impl federated_server::ServerRuntime {
    cfg_if::cfg_if! {
        if #[cfg(feature = "lambda")] {
            LambdaRuntime { telemetry }
        } else {
        }
    }
}
