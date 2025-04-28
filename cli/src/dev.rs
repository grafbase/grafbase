mod assets;
mod data_json;
mod extensions;
mod hot_reload;
mod subgraphs;

pub(crate) use self::subgraphs::SubgraphCache;

use super::errors::BackendError;
use crate::{
    cli_input::{DevCommand, FullGraphRef},
    errors::CliError,
};
use assets::{export_assets, get_base_router};
use federated_server::{GraphFetchMethod, ServeConfig, ServerRuntime};
use hot_reload::hot_reload;
use std::{
    net::{Ipv4Addr, SocketAddr},
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
    graphql_url: Arc<tokio::sync::OnceCell<String>>,
    mcp_url: Option<String>,
    subgraph_cache: Arc<SubgraphCache>,
}

impl ServerRuntime for CliRuntime {
    fn on_ready(&self, url: String) {
        self.graphql_url
            .set(url.clone())
            .expect("set error for url in on_ready()");

        self.ready_sender.send(url).expect("must still be open");
    }

    fn base_router<S>(&self) -> Option<axum::Router<S>> {
        Some(get_base_router(
            self.graphql_url.clone(),
            self.mcp_url.clone(),
            self.subgraph_cache.clone(),
        ))
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

    let (ready_sender, ready_receiver) = broadcast::channel::<String>(1);
    let (composition_warnings_sender, warnings_receiver) = mpsc::channel(12);
    let (sdl_sender, sdl_receiver) = mpsc::channel::<String>(2);

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

    spawn_blocking({
        let mcp_url = mcp_url.clone();
        move || {
            let _ = output_handler(
                output_handler_ready_receiver,
                warnings_receiver,
                introspection_forced,
                mcp_url,
            );
        }
    });

    let subgraph_cache =
        Arc::new(SubgraphCache::new(args.graph_ref.as_ref(), &config, composition_warnings_sender).await?);

    let composition_result = subgraph_cache.compose().await?;

    let federated_sdl = match composition_result {
        Ok(federated_schema) => federated_schema,
        Err(diagnostics) => {
            return Err(BackendError::Composition(diagnostics.iter_errors().collect::<Vec<_>>().join("\n")).into());
        }
    };

    sdl_sender
        .send(federated_sdl)
        .await
        .expect("this really has to succeed");

    let (config_sender, config_receiver) = watch::channel(config.clone());

    tokio::spawn(hot_reload(
        config_sender,
        sdl_sender,
        ready_receiver,
        subgraph_cache.clone(),
        config,
    ));

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

    federated_server::serve(
        server_config,
        CliRuntime {
            ready_sender,
            graphql_url: Default::default(),
            mcp_url,
            subgraph_cache,
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
