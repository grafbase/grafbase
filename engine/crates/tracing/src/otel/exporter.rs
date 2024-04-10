#[cfg(feature = "otlp")]
use crate::{
    config::{TracingOtlpExporterConfig, TracingOtlpExporterProtocol},
    error::TracingError,
};

#[cfg(feature = "otlp")]
pub(super) fn build_otlp_exporter<Exporter>(config: &TracingOtlpExporterConfig) -> Result<Exporter, TracingError>
    where
        Exporter: From<opentelemetry_otlp::TonicExporterBuilder>,
        Exporter: From<opentelemetry_otlp::HttpExporterBuilder>,
{
    use std::{str::FromStr, time::Duration};
    use opentelemetry_otlp::WithExportConfig;
    use tonic::{metadata::MetadataKey, transport::ClientTlsConfig};

    let exporter_timeout = Duration::from_secs(config.timeout.num_seconds() as u64);

    match config.protocol {
        TracingOtlpExporterProtocol::Grpc => {
            let grpc_config = config.grpc.clone().unwrap_or_default();

            let metadata = {
                // note: I'm not using MetadataMap::from_headers due to `http` crate version issues.
                // we're using 1 but otel currently pins tonic to an older version that requires 0.2.
                // once versions get aligned we can replace the following
                let headers = grpc_config.headers.try_into_map()?;

                let metadata = tonic::metadata::MetadataMap::with_capacity(headers.len());

                headers
                    .into_iter()
                    .fold(metadata, |mut acc, (header_name, header_value)| {
                        let key = MetadataKey::from_str(&header_name).unwrap();
                        acc.insert(key, header_value.parse().unwrap());
                        acc
                    })
            };

            let mut grpc_exporter = opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(config.endpoint.to_string())
                .with_timeout(exporter_timeout)
                .with_metadata(metadata);

            if let Some(tls_config) = grpc_config.tls {
                grpc_exporter = grpc_exporter.with_tls_config(ClientTlsConfig::try_from(tls_config)?);
            }

            Ok(grpc_exporter.into())
        }
        TracingOtlpExporterProtocol::Http => {
            let http_config = config.http.clone().unwrap_or_default();

            let http_exporter = opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint(config.endpoint.to_string())
                .with_timeout(exporter_timeout)
                .with_headers(http_config.headers.try_into_map()?);

            Ok(http_exporter.into())
        }
    }
}
