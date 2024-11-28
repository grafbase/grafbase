mod configurations;
mod hot_reload;
mod pathfinder;
mod subgraphs;

use super::errors::BackendError;
use configurations::get_and_merge_configurations;
use federated_server::{serve, GraphFetchMethod, ServerConfig, ServerRouter, ServerRuntime};
use gateway_config::Config;
use hot_reload::hot_reload;
use pathfinder::{export_assets, get_pathfinder_router};
use std::{
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
    sync::Arc,
};
use subgraphs::get_subgraph_sdls;
use tokio::sync::mpsc;
use tokio::{
    sync::broadcast::{channel, Receiver, Sender},
    task::spawn_blocking,
};

#[derive(Clone, Debug)]
pub struct FullGraphRef {
    pub account: String,
    pub graph: String,
    pub branch: Option<String>,
}

const DEFAULT_PORT: u16 = 5000;

#[derive(Clone)]
struct CliRuntime {
    ready_sender: Sender<String>,
    port: u16,
    home_dir: PathBuf,
}

impl ServerRuntime for CliRuntime {
    fn after_request(&self) {}

    fn on_ready(&self, url: String) {
        self.ready_sender.send(url).expect("must still be open");
    }

    fn get_external_router<T>(&self) -> Option<ServerRouter<T>> {
        Some(get_pathfinder_router(self.port, &self.home_dir))
    }
}

#[tokio::main(flavor = "multi_thread")]
pub async fn start(
    graph_ref: Option<FullGraphRef>,
    gateway_config_path: Option<PathBuf>,
    graph_overrides_path: Option<PathBuf>,
    port: Option<u16>,
) -> Result<(), BackendError> {
    export_assets().await?;

    // these need to live for the duration of the cli run,
    // leaking them prevents cloning them around
    let gateway_config_path = Box::leak(Box::new(gateway_config_path)).as_ref();
    let graph_overrides_path = Box::leak(Box::new(graph_overrides_path)).as_ref();

    let (ready_sender, mut _ready_receiver) = channel::<String>(1);

    let output_handler_ready_receiver = ready_sender.subscribe();

    spawn_blocking(|| {
        let _ = output_handler(output_handler_ready_receiver);
    });

    let dev_configuration = get_and_merge_configurations(gateway_config_path, graph_overrides_path).await?;

    let port = port
        .or(dev_configuration
            .merged_configuration
            .network
            .listen_address
            .map(|listen_address| listen_address.port()))
        .unwrap_or(DEFAULT_PORT);

    let listen_address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));

    let mut subgraphs = graphql_composition::Subgraphs::default();

    let subgraph_cache = get_subgraph_sdls(
        graph_ref.as_ref(),
        &dev_configuration.overridden_subgraphs,
        &dev_configuration.merged_configuration,
        &mut subgraphs,
        graph_overrides_path,
    )
    .await?;

    let composition_result = graphql_composition::compose(&subgraphs);

    let federated_sdl = match composition_result.into_result() {
        Ok(result) => federated_graph::render_federated_sdl(&result).map_err(BackendError::ToFederatedSdl)?,
        Err(diagnostics) => {
            return Err(BackendError::Composition(
                diagnostics.iter_messages().collect::<Vec<_>>().join("\n"),
            ))
        }
    };

    let (reload_sender, reload_receiver) = mpsc::channel::<(String, Arc<Config>)>(1);

    let server_config = ServerConfig {
        listen_addr: Some(listen_address),
        config_path: None,
        config_hot_reload: false,
        config: dev_configuration.merged_configuration.clone(),
        fetch_method: GraphFetchMethod::FromSchema {
            federated_sdl,
            reload_signal: Some(reload_receiver),
        },
    };

    let hot_reload_ready_sender = ready_sender.subscribe();

    tokio::spawn(async move {
        hot_reload(
            reload_sender,
            hot_reload_ready_sender,
            subgraph_cache,
            gateway_config_path,
            graph_overrides_path,
            dev_configuration,
        )
        .await;
    });

    let home_dir = dirs::home_dir().ok_or(BackendError::HomeDirectory)?;

    serve(
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

fn output_handler(mut receiver: Receiver<String>) -> Result<(), Box<dyn std::error::Error>> {
    use crossterm::{
        cursor::MoveUp,
        style::Stylize,
        terminal::{Clear, ClearType},
        QueueableCommand,
    };
    use std::io::stdout;

    println!("🕒 {} your subgraphs...\n", "Fetching".yellow().bold());

    let url = receiver.blocking_recv()?;

    stdout().queue(MoveUp(2))?.queue(Clear(ClearType::CurrentLine))?;

    println!("📡 {} on {}\n", "Listening".green().bold(), url.blue().bold());

    Ok(())
}
