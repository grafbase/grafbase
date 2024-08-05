use crate::{
    config::{OtlpExporterConfig, OtlpExporterProtocol},
    error::TracingError,
};
use either::Either;
use opentelemetry_otlp::{HttpExporterBuilder, TonicExporterBuilder, WithExportConfig};
use std::{str::FromStr, time::Duration};
use tonic::{metadata::MetadataKey, transport::ClientTlsConfig};

pub(super) fn build_otlp_exporter(
    config: &OtlpExporterConfig,
) -> Result<Either<TonicExporterBuilder, HttpExporterBuilder>, TracingError> {
    let exporter_timeout = Duration::from_secs(config.timeout.num_seconds() as u64);

    match config.protocol {
        OtlpExporterProtocol::Grpc => {
            let grpc_config = config.grpc.clone().unwrap_or_default();

            let metadata = {
                let metadata = tonic::metadata::MetadataMap::with_capacity(grpc_config.headers.len());

                grpc_config
                    .headers
                    .into_iter()
                    .fold(metadata, |mut acc, (header_name, header_value)| {
                        let key = MetadataKey::from_str(header_name.as_str()).unwrap();
                        acc.insert(key, header_value.as_str().parse().unwrap());
                        acc
                    })
            };

            let mut grpc_exporter = opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(config.endpoint.to_string())
                .with_timeout(exporter_timeout)
                .with_metadata(metadata);

            grpc_exporter = if let Some(tls_config) = grpc_config.tls {
                grpc_exporter
                    .with_tls_config(ClientTlsConfig::try_from(tls_config).map_err(TracingError::FileReadError)?)
            } else {
                grpc_exporter.with_tls_config(ClientTlsConfig::default().with_native_roots())
            };

            Ok(Either::Left(grpc_exporter))
        }
        OtlpExporterProtocol::Http => {
            let http_config = config.http.clone().unwrap_or_default();

            let http_exporter = opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint(config.endpoint.to_string())
                .with_timeout(exporter_timeout)
                .with_headers(http_config.headers.into_map());

            Ok(Either::Right(http_exporter))
        }
    }
}
