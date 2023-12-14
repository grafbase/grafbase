use crate::atomics::WORKER_PORT;
use crate::config::{build_config, ParsingResponse};
use crate::consts::{ASSET_VERSION_FILE, GIT_IGNORE_CONTENTS, GIT_IGNORE_FILE};
use crate::event::{wait_for_event, wait_for_event_and_match, Event};
use crate::file_watcher::start_watcher;
use crate::node::validate_node;
use crate::types::{ServerMessage, ASSETS_GZIP};
use crate::udf_builder::install_wrangler;
use crate::{bridge, errors::ServerError};
use crate::{error_server, proxy};
use bridge::BridgeState;
use common::consts::MAX_PORT;
use common::consts::{GRAFBASE_SCHEMA_FILE_NAME, GRAFBASE_TS_CONFIG_FILE_NAME};
use common::environment::{Environment, Project};
use engine::registry::Registry;
use flate2::read::GzDecoder;
use futures_util::FutureExt;
use sha2::Digest;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::broadcast::{self, channel};
use tokio::sync::mpsc::UnboundedSender;

const EVENT_BUS_BOUND: usize = 5;

pub struct ProductionServer {
    registry: Arc<Registry>,
    bridge_app: axum::Router,
    bridge_state: Box<dyn BridgeState>,
    environment_variables: HashMap<String, String>,
    message_sender: UnboundedSender<ServerMessage>,
    federated_graph_config: Option<parser_sdl::federation::FederatedGraphConfig>,
}

impl ProductionServer {
    pub async fn build(
        message_sender: UnboundedSender<ServerMessage>,
        parallelism: NonZeroUsize,
        tracing: bool,
    ) -> Result<Self, ServerError> {
        create_project_dot_grafbase_directory()?;

        let environment_variables: HashMap<_, _> = crate::environment::variables().collect();
        let ParsingResponse {
            registry,
            detected_udfs,
            federated_graph_config,
        } = build_config(&environment_variables, None).await?;
        let registry = Arc::new(registry);

        let (bridge_app, bridge_state) =
            bridge::build_router(message_sender.clone(), Arc::clone(&registry), tracing).await?;
        if !detected_udfs.is_empty() {
            validate_node().await?;
            let project = Project::get();

            let mut hasher = sha2::Sha256::new();

            for entry in walkdir::WalkDir::new(&project.path)
                .sort_by_file_name()
                .follow_links(true)
                .into_iter()
                .filter_map(Result::ok)
                // Only path we can somewhat safely ignore is the schema one
                .filter(|entry| !entry.file_type().is_dir() && entry.path() != project.schema_path.path())
            {
                let content =
                    std::fs::read(entry.path()).map_err(|err| ServerError::ReadFile(entry.path().into(), err))?;
                hasher.update(entry.path().to_string_lossy().as_bytes());
                hasher.update(content);
            }
            let hash = hasher.finalize().to_vec();
            let hash_path = project.dot_grafbase_directory_path.join("grafbase_hash");
            if hash != std::fs::read(&hash_path).unwrap_or_default() {
                export_embedded_files()?;
                install_wrangler(Environment::get(), tracing).await?;
                bridge_state.build_all_udfs(detected_udfs, parallelism).await?;
            }
            // If we fail to write the hash, we're just going to recompile the UDFs.
            let _ = std::fs::write(hash_path, hash);
        }
        Ok(Self {
            registry,
            bridge_app,
            bridge_state: Box::new(bridge_state),
            environment_variables,
            message_sender,
            federated_graph_config,
        })
    }

    pub async fn serve(self, listen_address: IpAddr, port: u16) -> Result<(), ServerError> {
        let is_federated = self.federated_graph_config.is_some();

        if let Some(config) = &self.federated_graph_config {
            let _ = self.message_sender.send(ServerMessage::Ready {
                listen_address,
                port,
                is_federated,
            });
            return federated_dev::run(port, true, config.clone())
                .await
                .map_err(|error| ServerError::GatewayError(error.to_string()));
        }
        let bridge_server = axum::Server::bind(&SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0))
            .serve(self.bridge_app.into_make_service());
        let bridge_port = bridge_server.local_addr().port();

        let gateway_app = gateway::Gateway::new(
            self.environment_variables,
            gateway::Bridge::new(bridge_port),
            self.registry,
        )
        .await
        .map_err(|error| ServerError::GatewayError(error.to_string()))?
        .into_router();

        let gateway_server =
            axum::Server::bind(&SocketAddr::new(listen_address, port)).serve(gateway_app.into_make_service());

        let _ = self.message_sender.send(ServerMessage::Ready {
            listen_address,
            port,
            is_federated,
        });
        tokio::select! {
            result = gateway_server => {
                result?;
            }
            result = bridge_server => {
                result?;
            }
        }
        self.bridge_state.close().await;
        Ok(())
    }
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
    message_sender: UnboundedSender<ServerMessage>,
) -> Result<(), ServerError> {
    let project = Project::get();

    // Exporting Pathfinder, TS parser & miniflare for resolvers.
    export_embedded_files()?;
    create_project_dot_grafbase_directory()?;

    let (event_bus, receiver) = channel::<Event>(EVENT_BUS_BOUND);

    if watch {
        let watch_event_bus = event_bus.clone();
        crate::codegen_server::start_codegen_worker(receiver, message_sender.clone())
            .expect("Invariant violation: codegen worker started twice.");

        tokio::select! {
            result = start_watcher(project.path.clone(), move |path| {
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
    message_sender: UnboundedSender<ServerMessage>,
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
                Event::NewSdlFromTsConfig(_) |
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
    message_sender: UnboundedSender<ServerMessage>,
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
        federated_graph_config,
    } = match build_config(&environment_variables, Some(event_bus)).await {
        Ok(parsing_response) => parsing_response,
        Err(error) => {
            let _: Result<_, _> = message_sender.send(ServerMessage::CompilationError(error.to_string()));
            tokio::spawn(async move { error_server::start(worker_port, error.to_string(), bridge_event_bus).await })
                .await??;
            return Ok(());
        }
    };

    let is_federated = federated_graph_config.is_some();

    if let Some(config) = federated_graph_config {
        let _: Result<_, _> = message_sender.send(ServerMessage::Ready {
            listen_address: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: proxy_port,
            is_federated,
        });

        return federated_dev::run(worker_port, false, config)
            .await
            .map_err(|error| ServerError::GatewayError(error.to_string()));
    }

    let registry = Arc::new(registry);
    // If the rebuild has been triggered by a change in the schema file, we can honour the freshness of resolvers
    // determined by inspecting the modified time of final artifacts of detected resolvers compared to the modified time
    // of the generated schema registry file.
    // Otherwise, we trigger a rebuild all resolvers. That, individually, will still more often than not be very quick
    // because the build naturally reuses the intermediate artifacts from node_modules from previous builds.
    // For this logic to become more fine-grained we would need to have an understanding of the module dependency graph
    // in resolvers, and that's a non-trivial problem.
    if path_changed
        .map(|path| (Path::new("./"), path))
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
        let app = gateway::Gateway::new(environment_variables, gateway::Bridge::new(bridge_port), registry)
            .await
            .map_err(|error| ServerError::GatewayError(error.to_string()))?
            .into_router();

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

    let _: Result<_, _> = message_sender.send(ServerMessage::Ready {
        listen_address: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port: proxy_port,
        is_federated,
    });

    tokio::select! {
        result = gateway => {
            result.map_err(|err| ServerError::MiniflareError(err.to_string()))?;
        },
        bridge_handle_result = bridge_handle => { bridge_handle_result??; }
    }

    Ok(())
}

pub fn export_embedded_files() -> Result<(), ServerError> {
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
