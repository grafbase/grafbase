mod configurations;
mod hot_reload;
mod subgraphs;

use super::errors::BackendError;
use configurations::get_and_merge_configurations;
use federated_server::{serve, GraphFetchMethod, ServerConfig, ServerRuntime};
use gateway_config::Config;
use hot_reload::hot_reload;
use std::{
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
};
use subgraphs::get_subgraph_sdls;
use tokio::sync::broadcast::{channel, Receiver, Sender};
use tokio::sync::mpsc;

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
}

impl ServerRuntime for CliRuntime {
    fn after_request(&self) {}

    fn on_ready(&self, url: String) {
        self.ready_sender.send(url).expect("must still be open");
    }
}

#[tokio::main(flavor = "multi_thread")]
pub async fn start(
    graph_ref: Option<FullGraphRef>,
    gateway_config_path: Option<PathBuf>,
    graph_overrides_path: Option<PathBuf>,
    port: Option<u16>,
) -> Result<(), BackendError> {
    // these need to live for the duration of the c,
    // leaking them prevents cloning them around
    let gateway_config_path = Box::leak(Box::new(gateway_config_path)).as_ref();
    let graph_overrides_path = Box::leak(Box::new(graph_overrides_path)).as_ref();

    let (ready_sender, mut _ready_receiver) = channel::<String>(1);

    let output_handler_ready_receiver = ready_sender.subscribe();

    tokio::task::spawn_blocking(|| {
        output_handler(output_handler_ready_receiver);
    });

    let dev_configuration = get_and_merge_configurations(gateway_config_path, graph_overrides_path).await?;

    let listen_address = SocketAddr::from((
        Ipv4Addr::LOCALHOST,
        port.or(dev_configuration
            .merged_configuration
            .network
            .listen_address
            .map(|listen_address| listen_address.port()))
            .unwrap_or(DEFAULT_PORT),
    ));

    let mut subgraphs = graphql_composition::Subgraphs::default();

    let subgraph_cache = get_subgraph_sdls(
        graph_ref.as_ref(),
        &dev_configuration,
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

    let (reload_sender, reload_receiver) = mpsc::channel::<(String, Config)>(1);

    let server_config = ServerConfig {
        listen_addr: Some(listen_address),
        config_path: None,
        config_hot_reload: false,
        config: dev_configuration.merged_configuration,
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
            graph_ref,
            subgraph_cache,
            gateway_config_path,
            graph_overrides_path,
        )
        .await
        .inspect_err(|error| {
            dbg!(error);
        })
    });

    serve(server_config, CliRuntime { ready_sender })
        .await
        .map_err(BackendError::Serve)?;

    Ok(())
}

// temporary output handler for internal testing until we move output to the CLI and use a proper terminal crate.
// none of us uses Windows, right?
fn output_handler(mut receiver: Receiver<String>) {
    // gray
    println!("\x1b[90mWarning: This command is in beta, expect missing features, bugs or breaking changes\x1b[0m\n");

    // yellow and bold
    println!("🕒 \x1b[1;33mFetching\x1b[0m your subgraphs...\n");

    let Ok(url) = receiver.blocking_recv() else {
        return;
    };

    // move the cursor up two lines and clear the line.
    // \x1b[{n}A moves the cursor up by {n} lines, \x1b[2K clears the line
    // not flushing here since we want it to update once rather than twice (once here and once for the next line if we flush)
    // this has the overall effect of replacing the "fetching" output with the "listening" output
    print!("\x1b[2A\x1b[2K");

    // green and bold, blue
    println!("📡 \x1b[1;32mListening\x1b[0m on \x1b[34m{url}\x1b[0m\n");
}
