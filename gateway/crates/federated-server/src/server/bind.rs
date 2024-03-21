use std::net::SocketAddr;

use axum::Router;

use crate::config::{TelemetryConfig, TlsConfig};

use super::{gateway::GatewayWatcher, state::ServerState};

pub(super) struct BindConfig<'a> {
    #[allow(dead_code)] // for non-lambda
    pub(super) addr: SocketAddr,
    pub(super) path: &'a str,
    pub(super) router: Router<ServerState>,
    pub(super) gateway: GatewayWatcher,
    #[allow(dead_code)] // for non-lambda
    pub(super) tls: Option<TlsConfig>,
    #[allow(dead_code)] // for lambda
    pub(super) telemetry: Option<TelemetryConfig>,
    pub(super) csrf: bool,
}

#[cfg(not(feature = "lambda"))]
pub(super) async fn bind(
    BindConfig {
        addr,
        path,
        router,
        gateway,
        tls,
        telemetry: _,
        csrf,
    }: BindConfig<'_>,
) -> Result<(), crate::Error> {
    let state = ServerState::new(gateway, None);
    let mut router = router.with_state(state);

    if csrf {
        router = super::csrf::inject_layer(router);
    }

    let app = router.into_make_service();

    match tls {
        Some(ref tls) => {
            tracing::info!("starting the Grafbase gateway at https://{addr}{path}");

            let rustls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(&tls.certificate, &tls.key)
                .await
                .map_err(crate::Error::CertificateError)?;

            axum_server::bind_rustls(addr, rustls_config)
                .serve(app)
                .await
                .map_err(crate::Error::Server)?
        }
        None => {
            tracing::info!("starting the Grafbase gateway in http://{addr}{path}");
            axum_server::bind(addr).serve(app).await.map_err(crate::Error::Server)?
        }
    }

    Ok(())
}

#[cfg(feature = "lambda")]
pub(super) async fn bind(
    BindConfig {
        addr: _,
        path,
        router,
        gateway,
        tls: _,
        telemetry,
        csrf,
    }: BindConfig<'_>,
) -> Result<(), crate::Error> {
    use grafbase_tracing::otel::opentelemetry::trace::TracerProvider;
    use grafbase_tracing::otel::tracing_subscriber::layer::SubscriberExt;
    use grafbase_tracing::otel::tracing_subscriber::EnvFilter;
    use grafbase_tracing::otel::{self, opentelemetry_sdk::runtime::Tokio};
    use grafbase_tracing::otel::{tracing_opentelemetry, tracing_subscriber};
    use tracing_futures::WithSubscriber;

    let (provider, subscriber) = match telemetry {
        Some(config) => {
            let provider = otel::layer::new_provider(&config.service_name, &config.tracing, Tokio).unwrap();

            let tracer = provider.tracer("lambda-otel");

            let subscriber = tracing_subscriber::registry()
                .with(tracing_opentelemetry::layer().with_tracer(tracer))
                .with(tracing_subscriber::fmt::layer().with_ansi(false))
                .with(EnvFilter::new(&config.tracing.filter));

            (Some(provider), Some(subscriber))
        }
        None => (None, None),
    };

    let state = ServerState::new(gateway, provider);
    let mut router = router.with_state(state);

    if csrf {
        router = super::csrf::inject_layer(router);
    }

    let app = tower::ServiceBuilder::new()
        .layer(axum_aws_lambda::LambdaLayer::default())
        .service(router);

    tracing::info!("starting the Grafbase Lambda gateway in {path}");

    match subscriber {
        Some(subscriber) => lambda_http::run(app)
            .with_subscriber(subscriber)
            .await
            .expect("unable to start lambda http server"),
        None => lambda_http::run(app).await.expect("unable to start lambda http server"),
    }

    Ok(())
}
