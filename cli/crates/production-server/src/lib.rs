//! Implements the self-hosted Grafbase gateway. It can run in a hybrid mode,
//! where we contact the schema registry in the API to fetch the latest schema
//! and send tracing and metrics to either our own or a 3rd party collector.

#![deny(missing_docs)]

use std::{fs, net::SocketAddr, path::Path, thread};
use tokio::runtime;
use tokio::runtime::Handle;
use tracing::log::{debug, error, warn};
use tracing::Subscriber;

pub use error::Error;
use grafbase_tracing::otel::layer::FilteredLayer;
use grafbase_tracing::otel::opentelemetry_sdk::runtime::Tokio;
pub use server::GraphFetchMethod;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{reload, EnvFilter};
use crate::config::Config;

mod config;
mod error;
mod server;

const THREAD_NAME: &str = "grafbase-gateway";

/// The crate result type.
pub type Result<T> = std::result::Result<T, Error>;

/// Starts the self-hosted Grafbase gateway. If started with a schema path, will
/// not connect our API for changes in the schema and if started without, we poll
/// the schema registry every ten second for changes.
pub fn start<S>(
    listen_addr: Option<SocketAddr>,
    config_path: &Path,
    graph: GraphFetchMethod,
    reload_handle: reload::Handle<FilteredLayer<S>, S>,
) -> Result<()>
where
    S: Subscriber + for<'span> LookupSpan<'span> + Send + Sync,
{
    let config = fs::read_to_string(config_path).map_err(Error::ConfigNotFound)?;
    let config: Config = toml::from_str(&config).map_err(Error::TomlValidation)?;

    let (otel_reload_tx, otel_reload_rx) = oneshot::channel::<Handle>();

    if let Some(telemetry_config) = config.telemetry.as_ref() {
        otel_reload(reload_handle, otel_reload_rx, telemetry_config);
    }

    let runtime = runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name(THREAD_NAME)
        .build()
        .map_err(|e| Error::InternalError(e.to_string()))?;

    runtime.block_on(async {
        let _ = otel_reload_tx
            .send(Handle::current())
            .inspect_err(|e| error!("error sending otel reload signal: {e}"));

        server::serve(listen_addr, config, graph).await
    })?;

    Ok(())
}

fn otel_reload<S>(
    reload_handle: reload::Handle<FilteredLayer<S>, S>,
    reload_rx: oneshot::Receiver<Handle>,
    telemetry_config: &config::TelemetryConfig,
) where
    S: Subscriber + for<'span> LookupSpan<'span> + Send + Sync,
{
    use tracing_subscriber::filter::FilterExt;

    let otel_service_name = telemetry_config.service_name.clone();
    let tracing_config = telemetry_config.tracing.clone();

    thread::spawn(move || match reload_rx.recv() {
        Ok(rt_handle) => {
            debug!("reloading otel layer");
            // new_batched will use the tokio runtime for its internals
            rt_handle.spawn(async move {
                // unfortunately I have to set the filters here due to: https://github.com/tokio-rs/tracing/issues/1629
                let env_filter = EnvFilter::new(&tracing_config.filter);

                // create the batched layer
                let otel_layer =
                    grafbase_tracing::otel::layer::new_batched::<S, Tokio>(otel_service_name, tracing_config, Tokio)
                        .expect("should successfully build a batched otel layer for tracing");

                // replace the existing layer with the new one and update its filters
                // the explicit filters update shouldn't be required but the bug mentioned above makes it so
                reload_handle
                    .modify(|layer| {
                        *layer.inner_mut() = otel_layer;
                        // order matters, sampling goes first
                        *layer.filter_mut() = FilterExt::boxed(env_filter);
                    })
                    .expect("should successfully reload otel layer");
            });
        }
        Err(e) => {
            warn!("received an error while waiting for otel reload: {e}");
        }
    });
}
