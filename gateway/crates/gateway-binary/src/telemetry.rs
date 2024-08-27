use grafbase_telemetry::otel::opentelemetry_sdk::metrics::SdkMeterProvider;

use grafbase_telemetry::config::TelemetryConfig;
use grafbase_telemetry::otel::layer::OtelTelemetry;
use grafbase_telemetry::otel::opentelemetry_sdk::runtime::Tokio;
use grafbase_telemetry::otel::opentelemetry_sdk::trace::TracerProvider;
use tracing_subscriber::EnvFilter;

use crate::args::{Args, LogStyle};

#[derive(Default, Clone)]
pub(crate) struct OpenTelemetryProviders {
    pub meter: Option<SdkMeterProvider>,
    pub tracer: Option<TracerProvider>,
}

pub(crate) fn init(args: &impl Args, config: TelemetryConfig) -> anyhow::Result<OpenTelemetryProviders> {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let env_filter = EnvFilter::new(args.log_level().as_ref());

    init_propagators(&config.tracing);

    cfg_if::cfg_if! {
      if #[cfg(feature = "lambda")] {
            let id_generator = opentelemetry_aws::trace::XrayIdGenerator::default();
        } else {
            use grafbase_telemetry::otel::opentelemetry_sdk::trace::RandomIdGenerator;
            let id_generator = RandomIdGenerator::default();
        }
    }

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

    let registry = tracing_subscriber::registry()
        .with(tracer.map(|t| t.layer))
        .with(logger.map(|l| l.layer));

    let is_terminal = atty::is(atty::Stream::Stdout);
    match args.log_style() {
        // for interactive terminals we provide colored output
        LogStyle::Pretty => registry
            .with(
                tracing_subscriber::fmt::layer()
                    .pretty()
                    .with_ansi(is_terminal)
                    .with_target(false),
            )
            .with(env_filter)
            .init(),
        // for server logs, colors are off
        LogStyle::Text => registry
            .with(
                tracing_subscriber::fmt::layer()
                    .with_ansi(is_terminal)
                    .with_target(false),
            )
            .with(env_filter)
            .init(),
        LogStyle::Json => registry
            .with(tracing_subscriber::fmt::layer().json())
            .with(env_filter)
            .init(),
    };

    Ok(OpenTelemetryProviders {
        meter: meter_provider,
        tracer: tracer_provider,
    })
}

fn init_propagators(tracing_config: &gateway_config::TracingConfig) {
    use grafbase_telemetry::otel::opentelemetry::propagation::TextMapPropagator;
    use opentelemetry_aws::trace::XrayPropagator;

    let mut propagators: Vec<Box<dyn TextMapPropagator + Send + Sync>> = Vec::new();

    if tracing_config.propagation.trace_context {
        propagators.push(Box::new(
            grafbase_telemetry::otel::opentelemetry_sdk::propagation::TraceContextPropagator::new(),
        ));
    }

    if tracing_config.propagation.baggage {
        propagators.push(Box::new(
            grafbase_telemetry::otel::opentelemetry_sdk::propagation::BaggagePropagator::new(),
        ))
    }

    if tracing_config.propagation.aws_xray {
        propagators.push(Box::new(XrayPropagator::default()));
    }

    if !propagators.is_empty() {
        let propagator =
            grafbase_telemetry::otel::opentelemetry::propagation::TextMapCompositePropagator::new(propagators);

        grafbase_telemetry::otel::opentelemetry::global::set_text_map_propagator(propagator);
    }
}