mod extensions;
mod hot_reload;
mod pathfinder;
mod subgraphs;

pub(crate) use self::subgraphs::SubgraphCache;

use super::errors::BackendError;
use crate::{
    cli_input::{DevCommand, FullGraphRef},
    errors::CliError,
};
use federated_server::{GraphFetchMethod, ServeConfig, ServerRuntime};
use hot_reload::hot_reload;
use pathfinder::{export_assets, get_pathfinder_router};
use std::{
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
    sync::Arc,
};
use tokio::{
    sync::{broadcast, mpsc, watch},
    task::spawn_blocking,
};

pub const DEFAULT_PORT: u16 = 5000;

#[derive(Clone)]
struct CliRuntime {
    ready_sender: broadcast::Sender<String>,
    port: u16,
    home_dir: PathBuf,
}

impl ServerRuntime for CliRuntime {
    fn on_ready(&self, url: String) {
        self.ready_sender.send(url).expect("must still be open");
    }

    fn base_router<S>(&self) -> Option<axum::Router<S>> {
        Some(get_pathfinder_router(self.port, &self.home_dir))
    }
}

pub(crate) fn dev(cmd: DevCommand) -> Result<(), CliError> {
    start(cmd).map_err(CliError::GenericError)
}

#[tokio::main(flavor = "multi_thread")]
async fn start(args: DevCommand) -> anyhow::Result<()> {
    export_assets().await?;

    let mut config = args.config()?;
    if !config.extensions.is_empty() {
        crate::extension::install::execute(&config)
            .await
            .map_err(|err| BackendError::Error(err.to_string()))?;
    }

    let (ready_sender, mut _ready_receiver) = broadcast::channel::<String>(1);
    let (composition_warnings_sender, warnings_receiver) = mpsc::channel(12);

    let output_handler_ready_receiver = ready_sender.subscribe();

    let introspection_forced = config.graph.introspection == Some(false);
    config.graph.introspection = Some(true);

    let port = args
        .port
        .or(config
            .network
            .listen_address
            .map(|listen_address| listen_address.port()))
        .unwrap_or(DEFAULT_PORT);

    let listen_address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));

    let mcp_url = config
        .mcp
        .as_ref()
        .filter(|m| m.enabled)
        .map(|m| format!("http://{listen_address}{}", m.path));

    spawn_blocking(move || {
        let _ = output_handler(
            output_handler_ready_receiver,
            warnings_receiver,
            introspection_forced,
            mcp_url,
        );
    });

    let subgraph_cache = Arc::new(SubgraphCache::new(args.graph_ref.as_ref(), &config).await?);

    let composition_result = subgraph_cache.compose().await?;

    {
        let mut warnings = composition_result.diagnostics().iter_warnings().peekable();

        if warnings.peek().is_some() {
            composition_warnings_sender
                .send(warnings.map(ToOwned::to_owned).collect())
                .await
                .unwrap();
        }
    }

    let federated_sdl = match composition_result.into_result() {
        Ok(result) => federated_graph::render_federated_sdl(&result).map_err(BackendError::ToFederatedSdl)?,
        Err(diagnostics) => {
            return Err(BackendError::Composition(diagnostics.iter_errors().collect::<Vec<_>>().join("\n")).into());
        }
    };

    let (sdl_sender, sdl_receiver) = mpsc::channel::<String>(2);
    let (config_sender, config_receiver) = watch::channel(config.clone());

    sdl_sender
        .send(federated_sdl)
        .await
        .expect("this really has to succeed");

    let current_dir = std::env::current_dir()
        .map_err(|error| BackendError::Error(format!("Failed to get current directory: {error}")))?;
    let server_config = ServeConfig {
        listen_address,
        config_path: None,
        config_hot_reload: false,
        config_receiver,
        fetch_method: GraphFetchMethod::FromSchemaReloadable {
            current_dir,
            sdl_receiver,
        },
    };

    let hot_reload_ready_receiver = ready_sender.subscribe();

    tokio::spawn(async move {
        hot_reload(
            config_sender,
            sdl_sender,
            hot_reload_ready_receiver,
            composition_warnings_sender,
            subgraph_cache,
            config,
        )
        .await;
    });

    let home_dir = dirs::home_dir().ok_or(BackendError::HomeDirectory)?;

    federated_server::serve(
        server_config,
        CliRuntime {
            ready_sender,
            port,
            home_dir,
        },
    )
    .await
    .map_err(BackendError::Serve)?;

    Ok(())
}

fn output_handler(
    mut url_receiver: broadcast::Receiver<String>,
    mut warnings_receiver: mpsc::Receiver<Vec<String>>,
    introspection_forced: bool,
    mcp_url: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    use crossterm::{
        QueueableCommand,
        cursor::MoveUp,
        style::Stylize,
        terminal::{Clear, ClearType},
    };
    use std::io::stdout;

    println!("Composing graph...\n");

    let url = url_receiver.blocking_recv()?;
    let graphql_url = url::Url::parse(&url)?;

    stdout().queue(MoveUp(2))?.queue(Clear(ClearType::CurrentLine))?;

    let explorer_url = format!(
        "http://{}:{}",
        graphql_url
            .host()
            .map(|h| h.to_string())
            .unwrap_or_else(|| "127.0.0.1".to_string()),
        graphql_url.port().unwrap()
    );

    println!("\n{} {}   {}", "➜".green(), "Local:".bold(), explorer_url);
    println!("{} {}: {}", "➜".green(), "GraphQL".bold(), graphql_url);

    if let Some(mcp_url) = mcp_url {
        println!("{} {}:     {}", "➜".green(), "MCP".bold(), mcp_url);
    }

    println!("\n");

    if introspection_forced {
        tracing::info!("introspection is always enabled in dev mode, config overridden");
    }

    while let Some(warnings) = warnings_receiver.blocking_recv() {
        println!(
            "⚠️ {}\n{}",
            "Composition warnings:".yellow().bold(),
            warnings
                .into_iter()
                .map(|warning| format!("- {warning}\n"))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    Ok(())
}
