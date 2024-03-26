use std::{fs, net::SocketAddr, path::PathBuf, sync::OnceLock};

use anyhow::anyhow;
use ascii::AsciiString;
use clap::{ArgGroup, Parser};
use federated_server::{Config, GraphFetchMethod};
use graph_ref::GraphRef;
use licensing::{ES256PublicKey, JWTClaims, License};

/// the tracing filter to be used when tracing is on
const TRACE_LOG_FILTER: &str = "tower_http=debug,federated_dev=trace,engine_v2=debug,federated_server=trace";
/// the tracing filter to be used when tracing is off
const DEFAULT_LOG_FILTER: &str = "info";

#[allow(clippy::panic)]
fn public_key() -> &'static ES256PublicKey {
    static PUBLIC_KEY: OnceLock<ES256PublicKey> = OnceLock::new();

    PUBLIC_KEY.get_or_init(|| match std::option_env!("GRAFBASE_LICENSE_PUBLIC_KEY") {
        Some(pem) => ES256PublicKey::from_pem(pem).expect("GRAFBASE_LICENSE_PUBLIC_KEY must be in PEM format"),
        None => {
            tracing::warn!("using the test key for license validation, not meant for production use");

            let pem = "-----BEGIN PUBLIC KEY-----\nMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE7cZKRjxtSY4EpAakYVVLoP2YaPkK\nAElkyqt+CsXDR2WF6Xu0XUA8qkfH6h19OZ2NOVcFyvmAVL1+OYx8vQWXMQ==\n-----END PUBLIC KEY-----";

            ES256PublicKey::from_pem(pem).expect("GRAFBASE_LICENSE_PUBLIC_KEY must be in PEM format")
        }
    })
}

#[derive(Debug, Parser)]
#[clap(
    group(
        ArgGroup::new("hybrid-or-airgapped")
            .required(true)
            .args(["graph_ref", "schema"])
    ),
    group(
        ArgGroup::new("graph-ref-with-access-token")
            .args(["graph_ref"])
            .requires("grafbase_access_token")
    )
)]
#[command(name = "The Grafbase Gateway", version)]
/// The Grafbase Gateway
pub struct Args {
    /// IP address on which the server will listen for incomming connections. Defaults to 127.0.0.1:5000.
    #[arg(short, long)]
    pub listen_address: Option<SocketAddr>,
    #[arg(short, long, help = GraphRef::ARG_DESCRIPTION, env = "GRAFBASE_GRAPH_REF")]
    pub graph_ref: Option<GraphRef>,
    /// An access token to the Grafbase API. The scope must allow operations on the given account,
    /// and graph defined in the graph-ref argument.
    #[arg(env = "GRAFBASE_ACCESS_TOKEN")]
    pub grafbase_access_token: Option<AsciiString>,
    /// Path to the TOML configuration file
    #[arg(long, short, env = "GRAFBASE_CONFIG_PATH")]
    config: PathBuf,
    /// Path to graph SDL. If provided, the graph will be static and no connection is made
    /// to the Grafbase API. A license must be present if defined.
    #[arg(long, short, env = "GRAFBASE_SCHEMA_PATH")]
    pub schema: Option<PathBuf>,
    /// Path to a Grafbase license file. Must be provided if defining a schema path.
    #[arg(long, env = "GRAFBASE_LICENSE_PATH")]
    pub license: Option<PathBuf>,
    /// Set the tracing level
    #[arg(short, long, default_value_t = 0)]
    pub trace: u16,
}

impl Args {
    /// The method of fetching a graph
    pub fn fetch_method(&self, license: Option<JWTClaims<License>>) -> anyhow::Result<GraphFetchMethod> {
        match (self.graph_ref.as_ref(), self.schema.as_ref()) {
            (None, Some(path)) => {
                let federated_graph = fs::read_to_string(path).map_err(|e| anyhow!("error loading schema:\n{e}"))?;

                Ok(GraphFetchMethod::FromLocal {
                    federated_schema: federated_graph,
                    license,
                })
            }
            #[cfg(not(feature = "lambda"))]
            (Some(graph_ref), None) => Ok(GraphFetchMethod::FromApi {
                access_token: self
                    .grafbase_access_token
                    .clone()
                    .expect("present due to the arg group"),
                graph_name: graph_ref.graph().to_string(),
                branch: graph_ref.branch().map(ToString::to_string),
            }),
            #[cfg(feature = "lambda")]
            (Some(_), None) => {
                let error = anyhow!("Hybrid mode is not available for lambda deployments, please provide the full GraphQL schema as a file.");

                Err(error)
            }
            _ => unreachable!(),
        }
    }

    /// Defines the log level for associated crates
    pub fn log_filter(&self) -> &str {
        if self.trace >= 1 {
            TRACE_LOG_FILTER
        } else {
            DEFAULT_LOG_FILTER
        }
    }

    /// The validated client license document.
    pub fn license(&self) -> anyhow::Result<Option<JWTClaims<License>>> {
        let Some(path) = self.license.as_ref() else {
            return Ok(None);
        };

        let token = fs::read_to_string(path).map_err(|e| anyhow!("error loading license:\n{e}"))?;
        let claims = License::verify(&token, public_key())?;

        Ok(Some(claims))
    }

    /// Load and validate the gateway configuration
    pub fn config(&self, license: &Option<JWTClaims<License>>) -> anyhow::Result<Config> {
        let enterprise_features = license.is_some() || self.grafbase_access_token.is_some();

        let config = fs::read_to_string(&self.config)?;
        let config: Config = toml::from_str(&config)?;

        let mut disallowed_features = Vec::new();

        if !enterprise_features {
            if config.operation_limits.is_some() {
                disallowed_features.push("operation limits");
            }

            if config.trusted_documents.is_some() {
                disallowed_features.push("trusted documents")
            }

            if config.authentication.is_some() {
                disallowed_features.push("authentication");
            }

            if config.subscriptions.is_some() {
                disallowed_features.push("subscriptions");
            }
        }

        if disallowed_features.is_empty() {
            return Ok(config);
        }

        let features = disallowed_features.join(", ");

        Err(anyhow!(
            "the following features are only available with a valid license: {features}"
        ))
    }
}
