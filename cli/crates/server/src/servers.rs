use crate::consts::{
    ASSET_VERSION_FILE, GIT_IGNORE_CONTENTS, GIT_IGNORE_FILE, MIN_NODE_VERSION, SCHEMA_PARSER_DIR, SCHEMA_PARSER_INDEX,
};
use crate::custom_resolvers::build_resolvers;
use crate::error_server;
use crate::event::{wait_for_event, wait_for_event_and_match, Event};
use crate::file_watcher::start_watcher;
use crate::types::{Assets, ServerMessage};
use crate::{bridge, errors::ServerError};
use common::consts::EPHEMERAL_PORT_RANGE;
use common::environment::Environment;
use common::types::LocalAddressType;
use common::utils::find_available_port_in_range;
use futures_util::FutureExt;

use std::env;
use std::sync::mpsc::{self, Receiver, Sender};
use std::{
    fs,
    process::Stdio,
    thread::{self, JoinHandle},
};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::runtime::Builder;
use tokio::sync::broadcast::{self, channel};
use version_compare::Version;
use which::which;

const EVENT_BUS_BOUND: usize = 5;

/// starts a development server by unpacking any files needed by the gateway worker
/// and starting the miniflare cli in `user_grafbase_path` in [`Environment`]
///
/// # Errors
///
/// returns [`ServerError::ReadVersion`] if the version file for the extracted worker files cannot be read
///
/// returns [`ServerError::CreateDir`] if the `WORKER_DIR` cannot be created
///
/// returns [`ServerError::WriteFile`] if a file cannot be written into `WORKER_DIR`
///
/// # Panics
///
/// The spawned server and miniflare thread can panic if either of the two inner spawned threads panic
#[must_use]
pub fn start(port: u16, watch: bool, tracing: bool) -> (JoinHandle<Result<(), ServerError>>, Receiver<ServerMessage>) {
    let (sender, receiver): (Sender<ServerMessage>, Receiver<ServerMessage>) = mpsc::channel();

    let environment = Environment::get();

    let handle = thread::spawn(move || {
        export_embedded_files()?;

        create_project_dot_grafbase_directory()?;

        let bridge_port = get_bridge_port(port)?;

        // manual implementation of #[tokio::main] due to a rust analyzer issue
        Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let (event_bus, _receiver) = channel::<Event>(EVENT_BUS_BOUND);

                if watch {
                    let watch_event_bus = event_bus.clone();

                    tokio::select! {
                        result = start_watcher(environment.project_grafbase_path.clone(),  move |path, event_type| {
                            let relative_path = path.strip_prefix(&environment.project_path).expect("must succeed by definition").to_owned();
                            watch_event_bus.send(Event::Reload(relative_path, event_type)).expect("cannot fail");
                        }) => { result }
                        result = server_loop(port, bridge_port, watch, sender, event_bus.clone(), tracing) => { result }
                    }
                } else {
                    Ok(spawn_servers(port, bridge_port, watch, sender, event_bus, tracing).await?)
                }
            })
    });

    (handle, receiver)
}

async fn server_loop(
    worker_port: u16,
    bridge_port: u16,
    watch: bool,
    sender: Sender<ServerMessage>,
    event_bus: broadcast::Sender<Event>,
    tracing: bool,
) -> Result<(), ServerError> {
    loop {
        let receiver = event_bus.subscribe();
        tokio::select! {
            result = spawn_servers(worker_port, bridge_port, watch, sender.clone(), event_bus.clone(), tracing) => {
                result?;
            }
            (path, file_event_type) = wait_for_event_and_match(receiver, |event| match event {
                Event::Reload(path, file_event_type) => Some((path, file_event_type)),
                Event::BridgeReady => None,
            }) => {
                trace!("reload");
                let _ = sender.send(ServerMessage::Reload(path, file_event_type));
            }
        }
    }
}

#[tracing::instrument(level = "trace")]
async fn spawn_servers(
    worker_port: u16,
    bridge_port: u16,
    watch: bool,
    sender: Sender<ServerMessage>,
    event_bus: broadcast::Sender<Event>,
    tracing: bool,
) -> Result<(), ServerError> {
    let bridge_event_bus = event_bus.clone();

    let receiver = event_bus.subscribe();

    validate_dependencies().await?;

    let environment_variables: std::collections::HashMap<_, _> = crate::environment::variables().collect();

    let resolvers = match run_schema_parser(&environment_variables).await {
        Ok(resolvers) => resolvers,
        Err(error) => {
            let _ = sender.send(ServerMessage::CompilationError(error.to_string()));
            tokio::spawn(async move { error_server::start(worker_port, error.to_string(), bridge_event_bus).await })
                .await??;
            return Ok(());
        }
    };

    let environment = Environment::get();

    let resolver_paths = match build_resolvers(&sender, environment, &environment_variables, resolvers, tracing).await {
        Ok(resolver_paths) => resolver_paths,
        Err(error) => {
            let _ = sender.send(ServerMessage::CompilationError(error.to_string()));
            tokio::spawn(async move { error_server::start(worker_port, error.to_string(), bridge_event_bus).await })
                .await??;
            return Ok(());
        }
    };

    let (bridge_sender, mut bridge_receiver) = tokio::sync::mpsc::channel(128);

    let mut bridge_handle =
        tokio::spawn(async move { bridge::start(bridge_port, worker_port, bridge_sender, bridge_event_bus).await })
            .fuse();

    let sender_cloned = sender.clone();
    tokio::spawn(async move {
        while let Some(message) = bridge_receiver.recv().await {
            sender_cloned.send(message).unwrap();
        }
    });

    trace!("waiting for bridge ready");
    tokio::select! {
        _ = wait_for_event(receiver, |event| *event == Event::BridgeReady) => (),
        result = &mut bridge_handle => {result??; return Ok(());}
    };
    trace!("bridge ready");

    let registry_path = environment
        .project_grafbase_registry_path
        .to_str()
        .ok_or(ServerError::ProjectPath)?;

    trace!("spawning miniflare for the main worker");

    let worker_port_string = worker_port.to_string();
    let bridge_port_binding_string = format!("BRIDGE_PORT={bridge_port}");
    let registry_text_blob_string = format!("REGISTRY={registry_path}");

    let mut miniflare_arguments: Vec<_> = [
        // used by miniflare when running normally as well
        "--experimental-vm-modules",
        "./node_modules/miniflare/dist/src/cli.js",
        "--modules",
        "--host",
        "127.0.0.1",
        "--port",
        &worker_port_string,
        "--no-update-check",
        "--no-cf-fetch",
        "--do-persist",
        "--wrangler-config",
        "./wrangler.toml",
        "--binding",
        &bridge_port_binding_string,
        "--text-blob",
        &registry_text_blob_string,
        "--mount",
        "stream-router=./stream-router",
    ]
    .into_iter()
    .map(std::borrow::Cow::Borrowed)
    .collect();
    miniflare_arguments.extend(resolver_paths.into_iter().flat_map(|(resolver_name, resolver_path)| {
        [
            "--mount".into(),
            format!(
                "{resolver_name}={resolver_path}",
                resolver_name = slug::slugify(resolver_name),
                resolver_path = resolver_path.display()
            )
            .into(),
        ]
    }));

    let mut miniflare = Command::new("node");
    miniflare
        // Unbounded worker limit
        .env("MINIFLARE_SUBREQUEST_LIMIT", "1000")
        .args(miniflare_arguments.iter().map(std::convert::AsRef::as_ref))
        .stdout(if tracing { Stdio::inherit() } else { Stdio::piped() })
        .stderr(if tracing { Stdio::inherit() } else { Stdio::piped() })
        .current_dir(&environment.user_dot_grafbase_path)
        .kill_on_drop(watch);
    trace!("Spawning {miniflare:?}");
    let miniflare = miniflare.spawn().map_err(ServerError::MiniflareCommandError)?;

    let _ = sender.send(ServerMessage::Ready(worker_port));

    let miniflare_output_result = miniflare.wait_with_output();

    tokio::select! {
        result = miniflare_output_result => {
            let output = result.map_err(ServerError::MiniflareCommandError)?;

            output
                .status
                .success()
                .then_some(())
                .ok_or_else(|| ServerError::MiniflareError(String::from_utf8_lossy(&output.stderr).into_owned()))?;
        }
        bridge_handle_result = bridge_handle => { bridge_handle_result??; }
    }

    Ok(())
}

fn export_embedded_files() -> Result<(), ServerError> {
    let environment = Environment::get();

    let current_version = env!("CARGO_PKG_VERSION");

    let version_path = environment.user_dot_grafbase_path.join(ASSET_VERSION_FILE);

    let export_files = if env::var("GRAFBASE_SKIP_ASSET_VERSION_CHECK").is_ok() {
        false
    } else if env::var("GRAFBASE_FORCE_EXPORT_FILES").is_ok() {
        true
    } else if version_path.exists() {
        let asset_version = fs::read_to_string(&version_path).map_err(|_| ServerError::ReadVersion)?;
        current_version != asset_version
    } else {
        true
    };

    if export_files {
        trace!("writing worker files");

        fs::create_dir_all(&environment.user_dot_grafbase_path).map_err(|_| ServerError::CreateCacheDir)?;

        let gitignore_path = &environment.user_dot_grafbase_path.join(GIT_IGNORE_FILE);

        fs::write(gitignore_path, GIT_IGNORE_CONTENTS)
            .map_err(|_| ServerError::WriteFile(gitignore_path.to_string_lossy().into_owned()))?;

        let mut write_results = Assets::iter().map(|path| {
            let file = Assets::get(path.as_ref());

            let full_path = environment.user_dot_grafbase_path.join(path.as_ref());

            let parent = full_path.parent().expect("must have a parent");
            let create_dir_result = fs::create_dir_all(parent);

            // must be Some(file) since we're iterating over existing paths
            let write_result = create_dir_result.and_then(|_| fs::write(&full_path, file.unwrap().data));

            (write_result, full_path)
        });

        if let Some((_, path)) = write_results.find(|(result, _)| result.is_err()) {
            let error_path_string = path.to_string_lossy().into_owned();
            return Err(ServerError::WriteFile(error_path_string));
        }

        if fs::write(&version_path, current_version).is_err() {
            let version_path_string = version_path.to_string_lossy().into_owned();
            return Err(ServerError::WriteFile(version_path_string));
        };
    }

    Ok(())
}

fn create_project_dot_grafbase_directory() -> Result<(), ServerError> {
    let environment = Environment::get();

    let project_dot_grafbase_path = environment.project_dot_grafbase_path.clone();

    if fs::metadata(&project_dot_grafbase_path).is_err() {
        trace!("creating .grafbase directory");
        fs::create_dir_all(&project_dot_grafbase_path).map_err(|_| ServerError::CreateCacheDir)?;
        fs::write(project_dot_grafbase_path.join(GIT_IGNORE_FILE), "*\n").map_err(|_| ServerError::CreateCacheDir)?;
    }

    Ok(())
}

#[derive(serde::Deserialize)]
struct SchemaParserResult {
    #[allow(dead_code)]
    required_resolvers: Vec<String>,
    versioned_registry: serde_json::Value,
}

// schema-parser is run via NodeJS due to it being built to run in a Wasm (via wasm-bindgen) environement
// and due to schema-parser not being open source
async fn run_schema_parser(
    environment_variables: &std::collections::HashMap<String, String>,
) -> Result<Vec<String>, ServerError> {
    trace!("parsing schema");

    let environment = Environment::get();

    let parser_path = environment
        .user_dot_grafbase_path
        .join(SCHEMA_PARSER_DIR)
        .join(SCHEMA_PARSER_INDEX);

    let parser_result_path = tokio::task::spawn_blocking(tempfile::NamedTempFile::new)
        .await?
        .map_err(ServerError::CreateTemporaryFile)?
        .into_temp_path();

    trace!(
        "using a temporary file for the parser output: {parser_result_path}",
        parser_result_path = parser_result_path.display()
    );

    let output = {
        let mut node_command = Command::new("node")
            .args([
                parser_path.to_str().ok_or(ServerError::CachePath)?,
                environment
                    .project_grafbase_schema_path
                    .to_str()
                    .ok_or(ServerError::ProjectPath)?,
                parser_result_path.to_str().expect("must be a valid path"),
            ])
            .current_dir(&environment.project_dot_grafbase_path)
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(ServerError::SchemaParserError)?;

        let node_command_stdin = node_command.stdin.as_mut().expect("stdin must be available");
        node_command_stdin
            .write_all(&serde_json::to_vec(environment_variables).expect("must serialise to JSON just fine"))
            .await
            .map_err(ServerError::SchemaParserError)?;

        node_command
            .wait_with_output()
            .await
            .map_err(ServerError::SchemaParserError)?
    };

    if !output.status.success() {
        return Err(ServerError::ParseSchema(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ));
    }

    let parser_result_string = tokio::fs::read_to_string(&parser_result_path)
        .await
        .map_err(ServerError::SchemaParserResultRead)?;
    let parser_result: SchemaParserResult =
        serde_json::from_str(&parser_result_string).map_err(ServerError::SchemaParserResultJson)?;

    tokio::fs::write(
        &environment.project_grafbase_registry_path,
        serde_json::to_string(&parser_result.versioned_registry)
            .expect("serde_json::Value serialises just fine for sure"),
    )
    .await
    .map_err(ServerError::SchemaRegistryWrite)?;

    Ok(parser_result.required_resolvers)
}

async fn get_node_version_string() -> Result<String, ServerError> {
    let output = Command::new("node")
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| ServerError::CheckNodeVersion)?
        .wait_with_output()
        .await
        .map_err(|_| ServerError::CheckNodeVersion)?;

    let node_version_string = String::from_utf8_lossy(&output.stdout).trim().to_owned();

    Ok(node_version_string)
}

async fn validate_node_version() -> Result<(), ServerError> {
    trace!("validating Node.js version");
    trace!("minimal supported Node.js version: {}", MIN_NODE_VERSION);

    let node_version_string = get_node_version_string().await?;

    trace!("installed node version: {}", node_version_string);

    let node_version = Version::from(&node_version_string).ok_or(ServerError::CheckNodeVersion)?;
    let min_version = Version::from(MIN_NODE_VERSION).expect("must be valid");

    if node_version >= min_version {
        Ok(())
    } else {
        Err(ServerError::OutdatedNode(
            node_version_string,
            MIN_NODE_VERSION.to_owned(),
        ))
    }
}

async fn validate_dependencies() -> Result<(), ServerError> {
    trace!("validating dependencies");

    which("node").map_err(|_| ServerError::NodeInPath)?;

    validate_node_version().await?;

    Ok(())
}

// the bridge runs on an available port within the ephemeral port range which is also supplied to the worker,
// making the port choice and availability transprent to the user.
// to avoid issues when starting multiple CLIs simultainiously,
// we segment the ephemeral port range into 100 segments and select a segment based on the last two digits of the process ID.
// this allows for simultainious start of up to 100 CLIs
fn get_bridge_port(http_port: u16) -> Result<u16, ServerError> {
    // must be 0-99, will fit in u16
    #[allow(clippy::cast_possible_truncation)]
    let segment = http_port % 100;
    // since the size is `max - min` in a u16 range, will fit in u16
    #[allow(clippy::cast_possible_truncation)]
    let size = EPHEMERAL_PORT_RANGE.len() as u16;
    let offset = size / 100 * segment;
    let start = EPHEMERAL_PORT_RANGE.min().expect("must exist");
    // allows us to loop back to the start of the range, giving any offset the same amount of potential ports
    let range = EPHEMERAL_PORT_RANGE.map(|port| (port + offset) % size + start);

    // TODO: loop back and limit iteration to get an even range for each
    find_available_port_in_range(range, LocalAddressType::Localhost).ok_or(ServerError::AvailablePort)
}
