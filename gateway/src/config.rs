use std::time::Duration;

use crate::args::Args;
use anyhow::Context;
use ascii::AsciiString;
use gateway_config::{BatchExportConfig, Config, OtlpExporterProtocol, telemetry::exporters::otlp::Headers};

const AUTHORIZATION_HEADER: &[u8] = b"authorization";
const GRAFBASE_GRAPH_REF_HEADER: &[u8] = b"grafbase-graph-ref";

pub fn load(args: &impl Args) -> anyhow::Result<Config> {
    let mut config = Config::loader()
        .load(args.config_path())
        .map_err(|err| anyhow::anyhow!(err))?
        .unwrap_or_default();

    // Merge Grafbase telemetry configuration with user configuration
    merge_grafbase_telemetry_config(&mut config, args)?;

    Ok(config)
}

fn merge_grafbase_telemetry_config(config: &mut Config, args: &impl Args) -> anyhow::Result<()> {
    // Only proceed if we have both access token and graph ref
    let token = args.grafbase_access_token()?;
    let graph_ref = args.graph_ref();

    let Some((token, graph_ref)) = token.zip(graph_ref) else {
        return Ok(());
    };

    // Get or create the grafbase telemetry config
    let config = config.telemetry.grafbase.get_or_insert_with(Default::default);

    // Set defaults only for fields that haven't been specified by the user

    // Default endpoint
    if config.endpoint.is_none() {
        config.endpoint = Some(
            ::std::env::var("GRAFBASE_OTEL_URL")
                .as_deref()
                .unwrap_or("https://otel.grafbase.com:443")
                .parse()
                .unwrap(),
        );
    }

    // Default to enabled
    if config.enabled.is_none() {
        config.enabled = Some(true);
    }

    // Default protocol
    if config.protocol.is_none() {
        config.protocol = Some(OtlpExporterProtocol::Grpc);
    }

    // Prepare the required headers
    let auth_header = (
        AsciiString::from_ascii(AUTHORIZATION_HEADER).context("Invalid auth header name")?,
        AsciiString::from_ascii(format!("Bearer {token}")).context("Invalid access token")?,
    );

    let graph_ref_header = (
        AsciiString::from_ascii(GRAFBASE_GRAPH_REF_HEADER).context("Invalid graph ref header name")?,
        AsciiString::from_ascii(graph_ref.to_string().into_bytes()).context("Invalid graph ref")?,
    );

    // Merge configuration based on protocol
    match config.protocol {
        Some(OtlpExporterProtocol::Grpc) | None => {
            let endpoint = config.endpoint.as_mut().unwrap();
            if endpoint.port().is_none() {
                endpoint.set_port(Some(4317)).unwrap();
            }
            // GRPC is the default if not specified
            let grpc = config.grpc.get_or_insert_with(Default::default);
            merge_headers(&mut grpc.headers, &auth_header, &graph_ref_header)?;
        }
        Some(OtlpExporterProtocol::Http) => {
            let endpoint = config.endpoint.as_mut().unwrap();
            if endpoint.port().is_none() {
                endpoint.set_port(Some(4318)).unwrap();
            }
            let http = config.http.get_or_insert_with(Default::default);
            merge_headers(&mut http.headers, &auth_header, &graph_ref_header)?;
        }
    }

    // Default batch export configuration
    if config.batch_export.is_none() {
        config.batch_export = Some(
            if let Some(seconds) = ::std::env::var("__GRAFBASE_OTEL_EXPORT_DELAY")
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
            {
                BatchExportConfig {
                    scheduled_delay: Duration::from_secs(seconds as u64),
                    ..Default::default()
                }
            } else {
                Default::default()
            },
        );
    }

    Ok(())
}

fn merge_headers(
    headers: &mut Headers,
    auth_header: &(AsciiString, AsciiString),
    graph_ref_header: &(AsciiString, AsciiString),
) -> anyhow::Result<()> {
    // Filter out any existing authorization or grafbase-graph-ref headers
    // and add our own to ensure they take precedence
    let mut new_headers: Vec<(AsciiString, AsciiString)> = headers
        .inner()
        .iter()
        .filter(|(k, _)| {
            let key = k.as_bytes();
            key != AUTHORIZATION_HEADER && key != GRAFBASE_GRAPH_REF_HEADER
        })
        .cloned()
        .collect();

    // Add our required headers
    new_headers.push(auth_header.clone());
    new_headers.push(graph_ref_header.clone());

    *headers = new_headers.into();

    Ok(())
}
