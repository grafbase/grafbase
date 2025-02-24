mod configurations;
mod extensions;
mod hot_reload;
mod pathfinder;
mod subgraphs;

use super::errors::BackendError;
use configurations::get_and_merge_configurations;
use federated_server::{GraphFetchMethod, ServeConfig, ServerRuntime};
use hot_reload::hot_reload;
use pathfinder::{export_assets, get_pathfinder_router};
use std::{
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
};
use subgraphs::get_subgraph_sdls;
use tokio::{
    sync::{broadcast, mpsc, watch},
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

    let (ready_sender, mut _ready_receiver) = broadcast::channel::<String>(1);
    let (composition_warnings_sender, warnings_receiver) = mpsc::channel(12);

    let output_handler_ready_receiver = ready_sender.subscribe();

    let dev_configuration = get_and_merge_configurations(gateway_config_path, graph_overrides_path).await?;
    let introspection_forced = dev_configuration.introspection_forced;

    spawn_blocking(move || {
        let _ = output_handler(output_handler_ready_receiver, warnings_receiver, introspection_forced);
    });

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
            return Err(BackendError::Composition(
                diagnostics.iter_errors().collect::<Vec<_>>().join("\n"),
            ));
        }
    };

    let (sdl_sender, sdl_receiver) = mpsc::channel::<String>(2);
    let (config_sender, config_receiver) = watch::channel(dev_configuration.merged_configuration.clone());

    sdl_sender
        .send(federated_sdl)
        .await
        .expect("this really has to succeed");

    let server_config = ServeConfig {
        listen_address,
        config_path: None,
        config_hot_reload: false,
        config_receiver,
        fetch_method: GraphFetchMethod::FromSchemaReloadable { sdl_receiver },
    };

    let hot_reload_ready_receiver = ready_sender.subscribe();

    tokio::spawn(async move {
        hot_reload(
            config_sender,
            sdl_sender,
            hot_reload_ready_receiver,
            composition_warnings_sender,
            subgraph_cache,
            gateway_config_path,
            graph_overrides_path,
            dev_configuration,
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
) -> Result<(), Box<dyn std::error::Error>> {
    use crossterm::{
        QueueableCommand,
        cursor::MoveUp,
        style::Stylize,
        terminal::{Clear, ClearType},
    };
    use std::io::stdout;

    println!("{} your subgraphs...\n", "Fetching".yellow().bold());

    let url = url_receiver.blocking_recv()?;
    let url = url::Url::parse(&url)?;

    stdout().queue(MoveUp(2))?.queue(Clear(ClearType::CurrentLine))?;

    let explorer_url = format!(
        "http://{}:{}",
        url.host()
            .map(|h| h.to_string())
            .unwrap_or_else(|| "127.0.0.1".to_string()),
        url.port().unwrap()
    );

    println!("GraphQL endpoint: {}", url.to_string().bold());
    println!("Explorer:         {}\n", explorer_url.bold());

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
