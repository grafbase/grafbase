use crate::consts::{
    ASSET_VERSION_FILE, EPHEMERAL_PORT_RANGE, GIT_IGNORE_CONTENTS, GIT_IGNORE_FILE, MIN_NODE_VERSION,
    SCHEMA_PARSER_DIR, SCHEMA_PARSER_INDEX,
};
use crate::event::{wait_for_event, Event};
use crate::file_watcher::start_watcher;
use crate::types::{Assets, ServerMessage};
use crate::{bridge, errors::ServerError};
use common::environment::Environment;
use common::types::LocalAddressType;
use common::utils::find_available_port_in_range;
use std::sync::mpsc::{self, Receiver, Sender};
use std::{
    fs,
    process::Stdio,
    thread::{self, JoinHandle},
};
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
pub fn start(port: u16, watch: bool) -> (JoinHandle<Result<(), ServerError>>, Receiver<ServerMessage>) {
    let (sender, receiver): (Sender<ServerMessage>, Receiver<ServerMessage>) = mpsc::channel();

    let environemnt = Environment::get();

    let handle = thread::spawn(move || {
        export_embedded_files()?;

        create_project_dot_grafbase_folder()?;

        // the bridge runs on an available port within the ephemeral port range which is also supplied to the worker,
        // making the port choice and availability transprent to the user
        let bridge_port = find_available_port_in_range(EPHEMERAL_PORT_RANGE, LocalAddressType::Localhost)
            .ok_or(ServerError::AvailablePort)?;

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
                        result = start_watcher(environemnt.project_grafbase_schema_path.clone(),  move || { watch_event_bus.send(Event::Reload).expect("cannot fail"); }) => { result }
                        result = server_loop(port, bridge_port, watch, sender, event_bus.clone()) => { result }
                    }
                } else {
                    Ok(spawn_servers(port, bridge_port, watch, sender, event_bus).await?)
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
) -> Result<(), ServerError> {
    loop {
        let receiver = event_bus.subscribe();
        tokio::select! {
            result = spawn_servers(worker_port, bridge_port, watch, sender.clone(), event_bus.clone()) => {
                result?;
            }
            _ = wait_for_event(receiver, Event::Reload) => {
                trace!("reload");
                let _ = sender.send(ServerMessage::Reload);
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
) -> Result<(), ServerError> {
    let bridge_sender = event_bus.clone();

    let receiver = event_bus.subscribe();

    validate_dependencies().await?;

    run_schema_parser().await?;

    let environment = Environment::get();

    let bridge_handle = tokio::spawn(async move { bridge::start(bridge_port, bridge_sender).await });

    trace!("waiting for bridge ready");

    wait_for_event(receiver, Event::BridgeReady).await;

    trace!("bridge ready");

    let registry_path = environment
        .project_grafbase_registry_path
        .to_str()
        .ok_or(ServerError::ProjectPath)?;

    trace!("spawining miniflare");

    let miniflare = Command::new("node")
        .args([
            // used by miniflare when running normally as well
            "--experimental-vm-modules",
            "./node_modules/miniflare/dist/src/cli.js",
            "--host",
            "127.0.0.1",
            "--port",
            &worker_port.to_string(),
            "--no-update-check",
            "--no-cf-fetch",
            "--wrangler-config",
            "wrangler.toml",
            "--binding",
            &format!("BRIDGE_PORT={bridge_port}"),
            "--text-blob",
            &format!("REGISTRY={registry_path}"),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(&environment.user_dot_grafbase_path)
        .kill_on_drop(watch)
        .spawn()
        .map_err(ServerError::MiniflareCommandError)?;

    let _ = sender.send(ServerMessage::Ready(worker_port));

    let miniflare_output_result = miniflare.wait_with_output();

    tokio::select! {
        _ = bridge_handle => {}
        result = miniflare_output_result => {
            let output = result.map_err(ServerError::MiniflareCommandError)?;

            output
                .status
                .success()
                .then_some(())
                .ok_or_else(|| ServerError::MiniflareError(String::from_utf8_lossy(&output.stderr).into_owned()))?;
        }
    }

    Ok(())
}

fn export_embedded_files() -> Result<(), ServerError> {
    let environment = Environment::get();

    let current_version = env!("CARGO_PKG_VERSION");

    let version_path = environment.user_dot_grafbase_path.join(ASSET_VERSION_FILE);

    let export_files = if environment.user_dot_grafbase_path.is_dir() {
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

            let parent_exists = parent.metadata().is_ok();

            let create_dir_result = if parent_exists {
                Ok(())
            } else {
                fs::create_dir_all(&parent)
            };

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

fn create_project_dot_grafbase_folder() -> Result<(), ServerError> {
    let environment = Environment::get();

    let project_dot_grafbase_path = environment.project_dot_grafbase_path.clone();

    if fs::metadata(&project_dot_grafbase_path).is_err() {
        trace!("creating .grafbase directory");
        fs::create_dir_all(&project_dot_grafbase_path).map_err(|_| ServerError::CreateCacheDir)?;
        fs::write(&project_dot_grafbase_path.join(GIT_IGNORE_FILE), "*\n").map_err(|_| ServerError::CreateCacheDir)?;
    }

    Ok(())
}

// schema-parser is run via NodeJS due to it being built to run in a Wasm (via wasm-bindgen) environement
// and due to schema-parser not being open source
async fn run_schema_parser() -> Result<(), ServerError> {
    trace!("parsing schema");

    let environment = Environment::get();

    let parser_path = environment
        .user_dot_grafbase_path
        .join(SCHEMA_PARSER_DIR)
        .join(SCHEMA_PARSER_INDEX);

    let output = Command::new("node")
        .args(&[
            &parser_path.to_str().ok_or(ServerError::CachePath)?,
            &environment
                .project_grafbase_schema_path
                .to_str()
                .ok_or(ServerError::ProjectPath)?,
        ])
        .current_dir(&environment.project_dot_grafbase_path)
        .stderr(Stdio::piped())
        .spawn()
        .map_err(ServerError::SchemaParserError)?
        .wait_with_output()
        .await
        .map_err(ServerError::SchemaParserError)?;

    output
        .status
        .success()
        .then_some(())
        .ok_or_else(|| ServerError::ParseSchema(String::from_utf8_lossy(&output.stderr).into_owned()))?;

    Ok(())
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
