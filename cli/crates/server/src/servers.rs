use crate::consts::{
    ASSET_VERSION_FILE, CONFIG_PARSER_SCRIPT, GENERATED_SCHEMAS_DIR, GIT_IGNORE_CONTENTS, GIT_IGNORE_FILE,
    MIN_NODE_VERSION, SCHEMA_PARSER_DIR, SCHEMA_PARSER_INDEX, TS_NODE_SCRIPT_PATH,
};
use crate::event::{wait_for_event, wait_for_event_and_match, Event};
use crate::file_watcher::start_watcher;
use crate::types::{ServerMessage, ASSETS_GZIP};
use crate::udf_builder::install_wrangler;
use crate::{bridge, errors::ServerError};
use crate::{error_server, playground};
use common::consts::{GRAFBASE_DIRECTORY_NAME, GRAFBASE_SCHEMA_FILE_NAME, GRAFBASE_TS_CONFIG_FILE_NAME};
use common::environment::{Environment, Project, SchemaLocation};
use common::types::UdfKind;
use futures_util::FutureExt;

use flate2::read::GzDecoder;
use std::borrow::Cow;
use std::env;
use std::net::TcpListener;
use std::path::Path;
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

    let project = Project::get();

    let handle = thread::spawn(move || {
        export_embedded_files()?;

        create_project_dot_grafbase_directory()?;

        // TODO change these to occupy a port and then return it with an event
        let bridge_port = find_available_port_for_internal_use()?;
        let playground_port = find_available_port_for_internal_use()?;

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
                        result = start_watcher(project.grafbase_directory_path.clone(), move |path| {
                            let relative_path = path.strip_prefix(&project.path).expect("must succeed by definition").to_owned();
                            watch_event_bus.send(Event::Reload(relative_path)).expect("cannot fail");
                        }) => { result }
                        result = server_loop(port, bridge_port, playground_port, watch, sender, event_bus, tracing) => { result }
                    }
                } else {
                    server_loop(port, bridge_port, playground_port, watch, sender, event_bus, tracing).await
                }
            })
    });

    (handle, receiver)
}

async fn server_loop(
    worker_port: u16,
    bridge_port: u16,
    playground_port: u16,
    watch: bool,
    sender: Sender<ServerMessage>,
    event_bus: broadcast::Sender<Event>,
    tracing: bool,
) -> Result<(), ServerError> {
    let mut path_changed = None;
    loop {
        let receiver = event_bus.subscribe();
        tokio::select! {
            result = spawn_servers(worker_port, bridge_port, playground_port, watch, sender.clone(), event_bus.clone(), path_changed.as_deref(), tracing) => {
                result?;
            }
            path = wait_for_event_and_match(receiver, |event| match event {
                Event::Reload(path) => Some(path),
                Event::BridgeReady => None
            }) => {
                trace!("reload");
                let _: Result<_, _> = sender.send(ServerMessage::Reload(path.clone()));
                path_changed = Some(path);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(level = "trace")]
async fn spawn_servers(
    worker_port: u16,
    bridge_port: u16,
    playground_port: u16,
    watch: bool,
    sender: Sender<ServerMessage>,
    event_bus: broadcast::Sender<Event>,
    path_changed: Option<&Path>,
    tracing: bool,
) -> Result<(), ServerError> {
    let bridge_event_bus = event_bus.clone();
    let playground_event_bus = event_bus.clone();

    let receiver = event_bus.subscribe();

    validate_dependencies().await?;

    let environment_variables: std::collections::HashMap<_, _> = crate::environment::variables().collect();

    let ParsingResponse { mut detected_udfs } = match run_schema_parser(&environment_variables).await {
        Ok(parsing_response) => parsing_response,
        Err(error) => {
            let _: Result<_, _> = sender.send(ServerMessage::CompilationError(error.to_string()));
            tokio::spawn(async move { error_server::start(worker_port, error.to_string(), bridge_event_bus).await })
                .await??;
            return Ok(());
        }
    };

    // If the rebuild has been triggered by a change in the schema file, we can honour the freshness of resolvers
    // determined by inspecting the modified time of final artifacts of detected resolvers compared to the modified time
    // of the generated schema registry file.
    // Otherwise, we trigger a rebuild all resolvers. That, individually, will still more often than not be very quick
    // because the build naturally reuses the intermediate artifacts from node_modules from previous builds.
    // For this logic to become more fine-grained we would need to have an understanding of the module dependency graph
    // in resolvers, and that's a non-trivial problem.
    if path_changed
        .map(|path| (Path::new(GRAFBASE_DIRECTORY_NAME), path))
        .filter(|(dir, path)| path != &dir.join(GRAFBASE_SCHEMA_FILE_NAME))
        .filter(|(dir, path)| path != &dir.join(GRAFBASE_TS_CONFIG_FILE_NAME))
        .is_some()
    {
        for udf in &mut detected_udfs {
            udf.fresh = false;
        }
    }

    let environment = Environment::get();
    let project = Project::get();

    if detected_udfs.is_empty() {
        trace!("Skipping wrangler installation");
    } else {
        if let Err(error) = install_wrangler(environment, tracing).await {
            let _: Result<_, _> = sender.send(ServerMessage::CompilationError(error.to_string()));
            // TODO consider disabling colored output from wrangler
            let error = strip_ansi_escapes::strip(error.to_string().as_bytes())
                .ok()
                .and_then(|stripped| String::from_utf8(stripped).ok())
                .unwrap_or_else(|| error.to_string());
            tokio::spawn(async move { error_server::start(worker_port, error, bridge_event_bus).await }).await??;
            return Ok(());
        }

        crate::udf_builder::install_dependencies(project, &sender, tracing).await?;
    }

    let playground_handle =
        tokio::spawn(async move { playground::start(playground_port, worker_port, playground_event_bus).await }).fuse();

    let (bridge_sender, mut bridge_receiver) = tokio::sync::mpsc::channel(128);

    let mut bridge_handle =
        tokio::spawn(
            async move { bridge::start(bridge_port, worker_port, bridge_sender, bridge_event_bus, tracing).await },
        )
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

    let registry_path = project.registry_path.to_str().ok_or(ServerError::ProjectPath)?;

    trace!("spawning miniflare for the main worker");

    let worker_port_string = worker_port.to_string();
    let bridge_port_binding_string = format!("BRIDGE_PORT={bridge_port}");
    let playground_port_binding_string = format!("PLAYGROUND_PORT={playground_port}");
    let registry_text_blob_string = format!("REGISTRY={registry_path}");

    #[allow(unused_mut)]
    let mut miniflare_arguments = Vec::from(
        [
            // used by miniflare when running normally as well
            "--experimental-vm-modules",
            crate::consts::MINIFLARE_CLI_JS_PATH,
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
            "--binding",
            &playground_port_binding_string,
            "--text-blob",
            &registry_text_blob_string,
            "--mount",
            "stream-router=./stream-router",
        ]
        .map(Cow::Borrowed),
    );

    #[cfg(feature = "dynamodb")]
    {
        #[allow(clippy::panic)]
        fn get_env(key: &str) -> String {
            let val = std::env::var(key).unwrap_or_else(|_| panic!("Environment variable not found:{key}"));
            format!("{key}={val}")
        }

        miniflare_arguments.extend(
            vec![
                "AWS_ACCESS_KEY_ID",
                "AWS_SECRET_ACCESS_KEY",
                "DYNAMODB_REGION",
                "DYNAMODB_TABLE_NAME",
            ]
            .iter()
            .map(|key| get_env(key))
            .flat_map(|env| ["--binding".into(), env.into()]),
        );
    }

    let mut miniflare = Command::new("node");
    miniflare
        // Unbounded worker limit
        .env("MINIFLARE_SUBREQUEST_LIMIT", "1000")
        .args(miniflare_arguments.iter().map(std::convert::AsRef::as_ref))
        .stdout(if tracing { Stdio::inherit() } else { Stdio::piped() })
        .stderr(if tracing { Stdio::inherit() } else { Stdio::piped() })
        .current_dir(&environment.user_dot_grafbase_path)
        .kill_on_drop(true);
    trace!("Spawning {miniflare:?}");
    let miniflare = miniflare.spawn().map_err(ServerError::MiniflareCommandError)?;

    let _: Result<_, _> = sender.send(ServerMessage::Ready(worker_port));

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
        playground_handle_result = playground_handle => { playground_handle_result??; }
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

        let reader = GzDecoder::new(ASSETS_GZIP);
        let mut archive = tar::Archive::new(reader);
        let full_path = &environment.user_dot_grafbase_path;
        archive
            .unpack(full_path)
            .map_err(|err| error!("unpack error: {err}"))
            .map_err(|_| ServerError::WriteFile(full_path.to_string_lossy().into_owned()))?;

        if fs::write(&version_path, current_version).is_err() {
            let version_path_string = version_path.to_string_lossy().into_owned();
            return Err(ServerError::WriteFile(version_path_string));
        };
    }

    Ok(())
}

fn create_project_dot_grafbase_directory() -> Result<(), ServerError> {
    let project = Project::get();

    let project_dot_grafbase_path = project.dot_grafbase_directory_path.clone();

    if fs::metadata(&project_dot_grafbase_path).is_err() {
        trace!("creating .grafbase directory");
        fs::create_dir_all(&project_dot_grafbase_path).map_err(|_| ServerError::CreateCacheDir)?;
        fs::write(project_dot_grafbase_path.join(GIT_IGNORE_FILE), "*\n").map_err(|_| ServerError::CreateCacheDir)?;
    }

    Ok(())
}

#[derive(serde::Deserialize)]
struct SchemaParserResult {
    versioned_registry: serde_json::Value,
    required_udfs: Vec<(UdfKind, String)>,
}

pub struct DetectedUdf {
    pub udf_name: String,
    pub udf_kind: UdfKind,
    pub fresh: bool,
}

pub struct ParsingResponse {
    detected_udfs: Vec<DetectedUdf>,
}

// schema-parser is run via NodeJS due to it being built to run in a Wasm (via wasm-bindgen) environment
// and due to schema-parser not being open source
async fn run_schema_parser(
    environment_variables: &std::collections::HashMap<String, String>,
) -> Result<ParsingResponse, ServerError> {
    trace!("parsing schema");
    let environment = Environment::get();
    let project = Project::get();

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
        let schema_path = match project.schema_path.location() {
            SchemaLocation::TsConfig(ref ts_config_path) => {
                Cow::Owned(parse_and_generate_config_from_ts(ts_config_path).await?)
            }
            SchemaLocation::Graphql(ref path) => Cow::Borrowed(path.to_str().ok_or(ServerError::ProjectPath)?),
        };

        let mut node_command = Command::new("node")
            .args([
                parser_path.to_str().ok_or(ServerError::CachePath)?,
                &schema_path,
                parser_result_path.to_str().expect("must be a valid path"),
            ])
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

    let SchemaParserResult {
        versioned_registry,
        required_udfs,
    } = serde_json::from_str(&parser_result_string).map_err(ServerError::SchemaParserResultJson)?;

    let registry_mtime = tokio::fs::metadata(&project.registry_path)
        .await
        .ok()
        .map(|metadata| metadata.modified().expect("must be supported"));

    let detected_resolvers = futures_util::future::join_all(required_udfs.into_iter().map(|(udf_kind, udf_name)| {
        // Last file to be written to in the build process.
        let wrangler_toml_path = project
            .udfs_build_artifact_path(udf_kind)
            .join(&udf_name)
            .join("wrangler.toml");
        async move {
            let wrangler_toml_mtime = tokio::fs::metadata(&wrangler_toml_path)
                .await
                .ok()
                .map(|metadata| metadata.modified().expect("must be supported"));
            let fresh = registry_mtime
                .zip(wrangler_toml_mtime)
                .map(|(registry_mtime, wrangler_toml_mtime)| wrangler_toml_mtime > registry_mtime)
                .unwrap_or_default();
            DetectedUdf {
                udf_name,
                udf_kind,
                fresh,
            }
        }
    }))
    .await;

    tokio::fs::write(
        &project.registry_path,
        serde_json::to_string(&versioned_registry).expect("serde_json::Value serialises just fine for sure"),
    )
    .await
    .map_err(ServerError::SchemaRegistryWrite)?;
    Ok(ParsingResponse {
        detected_udfs: detected_resolvers,
    })
}

/// Parses a TypeScript Grafbase configuration and generates a GraphQL schema
/// file to the filesystem, returning a path to the generated file.
async fn parse_and_generate_config_from_ts(ts_config_path: &Path) -> Result<String, ServerError> {
    let environment = Environment::get();
    let project = Project::get();

    let generated_schemas_dir = project.dot_grafbase_directory_path.join(GENERATED_SCHEMAS_DIR);
    let generated_config_path = generated_schemas_dir.join(GRAFBASE_SCHEMA_FILE_NAME);

    if !generated_schemas_dir.exists() {
        std::fs::create_dir_all(generated_schemas_dir).map_err(ServerError::SchemaParserError)?;
    }

    let config_parser_path = environment
        .user_dot_grafbase_path
        .join(SCHEMA_PARSER_DIR)
        .join(CONFIG_PARSER_SCRIPT);

    let ts_node_path = environment.user_dot_grafbase_path.join(TS_NODE_SCRIPT_PATH);

    let args = [
        ts_node_path.as_path(),
        config_parser_path.as_path(),
        ts_config_path,
        &generated_config_path,
    ];

    let node_command = Command::new("node")
        .args(args)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(ServerError::SchemaParserError)?;

    let output = node_command
        .wait_with_output()
        .await
        .map_err(ServerError::SchemaParserError)?;

    if !output.status.success() {
        let msg = String::from_utf8_lossy(&output.stderr);
        return Err(ServerError::LoadTsConfig(msg.into_owned()));
    }

    let generated_config_path = generated_config_path.to_str().ok_or(ServerError::ProjectPath)?;

    trace!("Generated configuration in {}.", generated_config_path);

    Ok(generated_config_path.to_string())
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

pub fn find_available_port_for_internal_use() -> Result<u16, ServerError> {
    let tcp_listener = TcpListener::bind("127.0.0.1:0").map_err(|_| ServerError::AvailablePort)?;
    Ok(tcp_listener
        .local_addr()
        .map_err(|_| ServerError::AvailablePort)?
        .port())
}
