use crate::atomics::{REGISTRY_PARSED_EPOCH_OFFSET_MILLIS, WORKER_PORT};
use crate::consts::{
    ASSET_VERSION_FILE, CONFIG_PARSER_SCRIPT, GENERATED_SCHEMAS_DIR, GIT_IGNORE_CONTENTS, GIT_IGNORE_FILE,
    MIN_NODE_VERSION, SCHEMA_PARSER_DIR, TS_NODE_SCRIPT_PATH,
};
use crate::event::{wait_for_event, wait_for_event_and_match, Event};
use crate::file_watcher::start_watcher;
use crate::types::{ServerMessage, ASSETS_GZIP};
use crate::udf_builder::install_wrangler;
use crate::{bridge, errors::ServerError};
use crate::{error_server, proxy};
use common::consts::MAX_PORT;
use common::consts::{GRAFBASE_DIRECTORY_NAME, GRAFBASE_SCHEMA_FILE_NAME, GRAFBASE_TS_CONFIG_FILE_NAME};
use common::environment::{Environment, Project, SchemaLocation};
use common::types::UdfKind;
use engine::registry::Registry;
use flate2::read::GzDecoder;
use futures_util::FutureExt;
use std::borrow::Cow;
use std::collections::HashMap;
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use std::{fs, process::Stdio};
use tokio::net::TcpListener;
use tokio::process::Command;
use tokio::sync::broadcast::{self, channel};
use version_compare::Version;
use which::which;

const EVENT_BUS_BOUND: usize = 5;

pub async fn production_start(
    listen_address: IpAddr,
    port: u16,
    tracing: bool,
    message_sender: tokio::sync::mpsc::UnboundedSender<ServerMessage>,
) -> Result<(), ServerError> {
    use bridge::BridgeState;

    create_project_dot_grafbase_directory()?;

    let environment_variables: HashMap<_, _> = crate::environment::variables().collect();
    let ParsingResponse {
        registry,
        detected_udfs,
    } = run_schema_parser(&environment_variables).await?;

    let environment = Environment::get();
    if !detected_udfs.is_empty() {
        export_embedded_files()?;
        validate_node().await?;
        install_wrangler(environment, tracing).await?;
    }

    let (bridge_app, bridge_state) =
        bridge::build_router(message_sender.clone(), Arc::clone(&registry), tracing).await?;
    bridge_state.build_all_udfs(detected_udfs).await?;

    let bridge_server =
        axum::Server::bind(&SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0)).serve(bridge_app.into_make_service());
    let bridge_port = bridge_server.local_addr().port();

    let gateway_app =
        gateway::Gateway::new(environment_variables, gateway::Bridge::new(bridge_port), registry).into_router();
    let gateway_server =
        axum::Server::bind(&SocketAddr::new(listen_address, port)).serve(gateway_app.into_make_service());

    let _ = message_sender.send(ServerMessage::Ready(port));
    tokio::select! {
        result = gateway_server => {
            result?;
        }
        result = bridge_server => {
            result?;
        }
    }
    bridge_state.close().await;
    Ok(())
}

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
pub async fn start(
    port: u16,
    search: bool,
    watch: bool,
    tracing: bool,
    message_sender: tokio::sync::mpsc::UnboundedSender<ServerMessage>,
) -> Result<(), ServerError> {
    let project = Project::get();

    create_project_dot_grafbase_directory()?;

    let (event_bus, _receiver) = channel::<Event>(EVENT_BUS_BOUND);

    if watch {
        let watch_event_bus = event_bus.clone();

        tokio::select! {
            result = start_watcher(project.grafbase_directory_path.clone(), move |path| {
                let relative_path = path.strip_prefix(&project.path).expect("must succeed by definition").to_owned();
                watch_event_bus.send(Event::Reload(relative_path)).expect("cannot fail");
            }) => { result }
            result = server_loop(port, search, message_sender, event_bus, tracing) => { result }
        }
    } else {
        server_loop(port, search, message_sender, event_bus, tracing).await
    }
}

async fn server_loop(
    port: u16,
    search: bool,
    message_sender: tokio::sync::mpsc::UnboundedSender<ServerMessage>,
    event_bus: broadcast::Sender<Event>,
    tracing: bool,
) -> Result<(), ServerError> {
    let proxy_event_bus = event_bus.clone();
    let proxy_error_event_bus = event_bus.clone();
    let listener = find_listener_for_available_port(search, port).await?;
    let proxy_port = listener.local_addr().expect("must have a local addr").port();
    let proxy_handle = tokio::spawn(async move {
        if let Err(error) = proxy::start(listener, proxy_event_bus).await {
            proxy_error_event_bus.send(Event::ProxyError).expect("must succeed");
            Err(error)
        } else {
            Ok(())
        }
    })
    .fuse();
    let mut path_changed = None;
    loop {
        let receiver = event_bus.subscribe();

        tokio::select! {
            result = spawn_servers(proxy_port, message_sender.clone(), event_bus.clone(), path_changed.as_deref(), tracing) => {
                result?;
            }
            path = wait_for_event_and_match(receiver, |event| match event {
                Event::Reload(path) => Some(path),
                Event::BridgeReady |
                Event::ProxyError => None,
            }) => {
                trace!("reload");
                let _: Result<_, _> = message_sender.send(ServerMessage::Reload(path.clone()));
                path_changed = Some(path);
            }
            () = wait_for_event(event_bus.subscribe(), |event| *event == Event::ProxyError) => { break; }
        }
    }
    proxy_handle.await?
}

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(level = "trace")]
async fn spawn_servers(
    proxy_port: u16,
    message_sender: tokio::sync::mpsc::UnboundedSender<ServerMessage>,
    event_bus: broadcast::Sender<Event>,
    path_changed: Option<&Path>,
    tracing: bool,
) -> Result<(), ServerError> {
    let bridge_event_bus = event_bus.clone();

    let receiver = event_bus.subscribe();

    let environment_variables: HashMap<_, _> = crate::environment::variables().collect();

    let worker_port = get_random_port_unchecked().await?;

    WORKER_PORT.store(worker_port, Ordering::Relaxed);

    let ParsingResponse {
        registry,
        mut detected_udfs,
    } = match run_schema_parser(&environment_variables).await {
        Ok(parsing_response) => parsing_response,
        Err(error) => {
            let _: Result<_, _> = message_sender.send(ServerMessage::CompilationError(error.to_string()));
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

    if detected_udfs.is_empty() {
        trace!("Skipping wrangler installation");
    } else {
        export_embedded_files()?;
        validate_node().await?;
        if let Err(error) = install_wrangler(environment, tracing).await {
            let _: Result<_, _> = message_sender.send(ServerMessage::CompilationError(error.to_string()));
            // TODO consider disabling colored output from wrangler
            let error = strip_ansi_escapes::strip(error.to_string().as_bytes())
                .ok()
                .and_then(|stripped| String::from_utf8(stripped).ok())
                .unwrap_or_else(|| error.to_string());
            tokio::spawn(async move { error_server::start(worker_port, error, bridge_event_bus).await }).await??;
            return Ok(());
        }
    }

    let (mut bridge_handle, bridge_port) = {
        let (listern, port) = get_listener_for_random_port().await?;
        let registry = Arc::clone(&registry);
        let message_sender = message_sender.clone();
        let handle = tokio::spawn(async move {
            bridge::start(listern, port, message_sender, bridge_event_bus, registry, tracing).await
        })
        .fuse();
        (handle, port)
    };

    let gateway = {
        let app =
            gateway::Gateway::new(environment_variables, gateway::Bridge::new(bridge_port), registry).into_router();
        // run it with hyper on localhost:3000
        let server = axum::Server::bind(&"127.0.0.1:0".parse().unwrap()).serve(app.into_make_service());
        WORKER_PORT.store(server.local_addr().port(), Ordering::Relaxed);
        server
    };

    trace!("waiting for bridge ready");
    tokio::select! {
        () = wait_for_event(receiver, |event| *event == Event::BridgeReady) => (),
        result = &mut bridge_handle => {result??; return Ok(());}
    };
    trace!("bridge ready");

    let _: Result<_, _> = message_sender.send(ServerMessage::Ready(proxy_port));

    tokio::select! {
        result = gateway => {
            result.map_err(|err| ServerError::MiniflareError(err.to_string()))?;
        },
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

        let reader = GzDecoder::new(ASSETS_GZIP);
        let mut archive = tar::Archive::new(reader);
        let full_path = &environment.user_dot_grafbase_path;
        archive
            .unpack(full_path)
            .map_err(|err| error!("unpack error: {err}"))
            .map_err(|()| ServerError::WriteFile(full_path.to_string_lossy().into_owned()))?;

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

pub struct DetectedUdf {
    pub udf_name: String,
    pub udf_kind: UdfKind,
    pub fresh: bool,
}

pub struct ParsingResponse {
    registry: Arc<Registry>,
    detected_udfs: Vec<DetectedUdf>,
}

// schema-parser is run via NodeJS due to it being built to run in a Wasm (via wasm-bindgen) environment
// and due to schema-parser not being open source
async fn run_schema_parser(environment_variables: &HashMap<String, String>) -> Result<ParsingResponse, ServerError> {
    trace!("parsing schema");
    let project = Project::get();

    let schema_path = match project.schema_path.location() {
        SchemaLocation::TsConfig(ref ts_config_path) => {
            Cow::Owned(parse_and_generate_config_from_ts(ts_config_path).await?)
        }
        SchemaLocation::Graphql(ref path) => Cow::Borrowed(path.to_str().ok_or(ServerError::ProjectPath)?),
    };
    let schema = tokio::fs::read_to_string(Path::new(schema_path.as_ref()))
        .await
        .map_err(ServerError::SchemaParserError)?;

    let crate::parser::ParserResult {
        registry,
        required_udfs,
    } = crate::parser::parse_schema(&schema, environment_variables).await?;

    let offset = REGISTRY_PARSED_EPOCH_OFFSET_MILLIS.load(Ordering::Acquire);
    let registry_mtime = SystemTime::UNIX_EPOCH.checked_add(Duration::from_millis(offset));
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

    REGISTRY_PARSED_EPOCH_OFFSET_MILLIS.store(
        u64::try_from(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        )
        .unwrap(),
        Ordering::Release,
    );

    Ok(ParsingResponse {
        registry: Arc::new(registry),
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

    validate_node().await?;
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

async fn validate_node() -> Result<(), ServerError> {
    trace!("validating Node.js version");
    trace!("minimal supported Node.js version: {}", MIN_NODE_VERSION);

    which("node").map_err(|_| ServerError::NodeInPath)?;

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

pub async fn get_listener_for_random_port() -> Result<(std::net::TcpListener, u16), ServerError> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .await
        .map_err(|_| ServerError::AvailablePort)?;
    let port = listener.local_addr().map_err(|_| ServerError::AvailablePort)?.port();
    Ok((listener.into_std().map_err(|_| ServerError::AvailablePort)?, port))
}

pub async fn get_random_port_unchecked() -> Result<u16, ServerError> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .await
        .map_err(|_| ServerError::AvailablePort)?;
    Ok(listener.local_addr().map_err(|_| ServerError::AvailablePort)?.port())
}

/// determines if a port or port range are available
pub async fn find_listener_for_available_port(
    search: bool,
    start_port: u16,
) -> Result<std::net::TcpListener, ServerError> {
    if search {
        find_listener_for_available_port_in_range(start_port..MAX_PORT).await
    } else {
        TcpListener::bind((Ipv4Addr::LOCALHOST, start_port))
            .await
            .map_err(|_| ServerError::PortInUse(start_port))?
            .into_std()
            .map_err(|_| ServerError::PortInUse(start_port))
    }
}

/// finds an available port within a range
pub async fn find_listener_for_available_port_in_range<R>(range: R) -> Result<std::net::TcpListener, ServerError>
where
    R: ExactSizeIterator<Item = u16>,
{
    for port in range {
        if let Ok(listener) = TcpListener::bind((Ipv4Addr::LOCALHOST, port)).await {
            return listener.into_std().map_err(|_| ServerError::AvailablePortMiniflare);
        }
    }
    Err(ServerError::AvailablePortMiniflare)
}
