use federated_server::Config;
use grafbase_tracing::otel::{
    layer,
    opentelemetry_sdk::runtime::Tokio,
    tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter},
};

use crate::args::Args;

pub(super) fn init(config: &mut Config, args: &Args) -> Result<(), anyhow::Error> {
    let (otel_layer, filter) = match config.telemetry.take() {
        Some(config) => {
            let env_filter = EnvFilter::new(&config.tracing.filter);
            let otel_layer = layer::new_batched(&config.service_name, config.tracing, Tokio)?;

            (Some(otel_layer), env_filter)
        }
        None => (None, EnvFilter::builder().parse_lossy(args.log_filter())),
    };

    tracing_subscriber::registry()
        .with(otel_layer)
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();

    Ok(())
}
