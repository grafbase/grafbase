use grafbase_telemetry::otel::opentelemetry_sdk::metrics::SdkMeterProvider;
use tracing_subscriber::EnvFilter;

use grafbase_telemetry::config::TelemetryConfig;
use grafbase_telemetry::otel::layer::OtelTelemetry;
use grafbase_telemetry::otel::opentelemetry_sdk::runtime::Tokio;
use grafbase_telemetry::otel::opentelemetry_sdk::trace::TracerProvider;

use crate::args::Args;

#[derive(Default, Clone)]
pub(crate) struct OpenTelemetryProviders {
    pub meter: Option<SdkMeterProvider>,
    pub tracer: Option<TracerProvider>,
}

pub(crate) fn init(args: &impl Args, config: TelemetryConfig) -> anyhow::Result<OpenTelemetryProviders> {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let filter = args.log_level().map(|l| l.as_filter_str()).unwrap_or("info");
    let env_filter = EnvFilter::new(filter);
    let id_generator = {
        cfg_if::cfg_if! {
            if #[cfg(feature = "lambda")] {
                use opentelemetry_aws::trace::{XrayIdGenerator, XrayPropagator};
                grafbase_telemetry::otel::opentelemetry::global::set_text_map_propagator(XrayPropagator::default());

                XrayIdGenerator::default()
            } else {
                use grafbase_telemetry::otel::opentelemetry_sdk::trace::RandomIdGenerator;

                RandomIdGenerator::default()
            }
        }
    };

    let OtelTelemetry {
        tracer,
        meter_provider,
        logger,
    } = grafbase_telemetry::otel::layer::build(config, id_generator, Tokio)?;

    if let Some(ref meter_provider) = meter_provider {
        grafbase_telemetry::otel::opentelemetry::global::set_meter_provider(meter_provider.clone());
    }

    if let Some(ref tracer) = tracer {
        grafbase_telemetry::otel::opentelemetry::global::set_tracer_provider(tracer.provider.clone());
    }
    let tracer_provider = tracer.as_ref().map(|t| t.provider.clone());

    match (logger, tracer) {
        (None, None) => {
            tracing_subscriber::registry()
                .with(args.log_format())
                .with(env_filter)
                .init();
        }
        (None, Some(tracer)) => {
            tracing_subscriber::registry()
                .with(tracer.layer)
                .with(args.log_format())
                .with(env_filter)
                .init();
        }
        (Some(logger), None) => {
            tracing_subscriber::registry()
                .with(logger.layer)
                .with(args.log_format())
                .with(env_filter)
                .init();
        }
        (Some(logger), Some(tracer)) => {
            tracing_subscriber::registry()
                .with(tracer.layer)
                .with(logger.layer)
                .with(args.log_format())
                .with(env_filter)
                .init();
        }
    }

    Ok(OpenTelemetryProviders {
        meter: meter_provider,
        tracer: tracer_provider,
    })
}
