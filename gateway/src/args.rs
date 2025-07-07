mod lambda;
mod log;
mod std;

use ::std::{net::SocketAddr, path::Path, time::Duration};

use anyhow::Context;
use ascii::AsciiString;
use clap::Parser;
use federated_server::{AccessToken, GraphFetchMethod};
use gateway_config::{BatchExportConfig, Config, OtlpExporterConfig, OtlpExporterGrpcConfig, OtlpExporterProtocol};
use graph_ref::GraphRef;
pub(crate) use log::*;

pub(crate) trait Args {
    fn listen_address(&self) -> Option<SocketAddr>;

    fn log_level(&self) -> LogLevel<'_>;

    fn fetch_method(&self) -> anyhow::Result<GraphFetchMethod>;

    fn config(&self) -> anyhow::Result<Config>;

    fn config_path(&self) -> Option<&Path>;

    fn hot_reload(&self) -> bool;

    fn log_style(&self) -> LogStyle;

    fn graph_ref(&self) -> Option<&GraphRef>;

    fn can_export_telemetry_to_platform(&self) -> bool {
        AccessToken::is_defined_in_env() && self.graph_ref().is_some()
    }

    fn grafbase_access_token(&self) -> anyhow::Result<Option<AccessToken>> {
        AccessToken::from_env().map_err(anyhow::Error::msg)
    }

    fn grafbase_otel_config(&self) -> anyhow::Result<Option<OtlpExporterConfig>> {
        let token = self.grafbase_access_token()?;
        let graph_ref = self.graph_ref();

        let Some((token, graph_ref)) = token.zip(graph_ref) else {
            return Ok(None);
        };

        let endpoint = ::std::env::var("GRAFBASE_OTEL_URL")
            .unwrap_or("https://otel.grafbase.com".to_string())
            .parse()
            .unwrap();

        let grpc = OtlpExporterGrpcConfig {
            tls: None,
            headers: vec![
                (
                    AsciiString::from_ascii(b"authorization").context("Invalid auth header name")?,
                    AsciiString::from_ascii(format!("Bearer {token}")).context("Invalid access token")?,
                ),
                (
                    AsciiString::from_ascii(b"grafbase-graph-ref").context("Invalid graph ref header name")?,
                    AsciiString::from_ascii(graph_ref.to_string().into_bytes()).context("Invalid graph ref")?,
                ),
            ]
            .into(),
        };

        let batch_export = if let Some(seconds) = ::std::env::var("__GRAFBASE_OTEL_EXPORT_DELAY")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
        {
            BatchExportConfig {
                scheduled_delay: Duration::from_secs(seconds as u64),
                ..Default::default()
            }
        } else {
            Default::default()
        };

        let config = OtlpExporterConfig {
            endpoint: Some(endpoint),
            enabled: Some(true),
            protocol: Some(OtlpExporterProtocol::Grpc),
            grpc: Some(grpc),
            batch_export: Some(batch_export),
            ..Default::default()
        };

        Ok(Some(config))
    }
}

pub(crate) fn parse() -> impl Args {
    cfg_if::cfg_if! {
        if #[cfg(feature = "lambda")] {
            lambda::Args::parse()
        } else {
            std::Args::parse()
        }
    }
}
