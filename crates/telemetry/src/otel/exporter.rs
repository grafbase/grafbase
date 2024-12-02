use crate::error::TracingError;
use gateway_config::{Headers, OtlpExporterTlsConfig};
use std::str::FromStr;
use tonic::{
    metadata::{MetadataKey, MetadataMap},
    transport::ClientTlsConfig,
};

pub(super) fn build_metadata(headers: Headers) -> MetadataMap {
    let metadata = tonic::metadata::MetadataMap::with_capacity(headers.len());

    headers
        .into_iter()
        .fold(metadata, |mut acc, (header_name, header_value)| {
            let key = MetadataKey::from_str(header_name.as_str()).unwrap();
            acc.insert(key, header_value.as_str().parse().unwrap());
            acc
        })
}

pub(super) fn build_tls_config(tls: Option<OtlpExporterTlsConfig>) -> Result<ClientTlsConfig, TracingError> {
    if let Some(tls_config) = tls {
        ClientTlsConfig::try_from(tls_config).map_err(TracingError::FileReadError)
    } else {
        Ok(ClientTlsConfig::default().with_native_roots())
    }
}
