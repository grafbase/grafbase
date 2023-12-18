use crate::atomics::WORKER_PORT;
use crate::config::{build_config, Config, ConfigActor};
use crate::consts::{ASSET_VERSION_FILE, GIT_IGNORE_CONTENTS, GIT_IGNORE_FILE};
use crate::file_watcher::Watcher;
use crate::node::validate_node;
use crate::proxy::ProxyHandle;
use crate::types::{MessageSender, ServerMessage, ASSETS_GZIP};
use crate::udf_builder::install_wrangler;
use crate::{bridge, errors::ServerError};
use crate::{error_server, proxy};
use bridge::BridgeState;
use common::channels::constant_watch_receiver;
use common::consts::MAX_PORT;
use common::consts::{GRAFBASE_SCHEMA_FILE_NAME, GRAFBASE_TS_CONFIG_FILE_NAME};
use common::environment::{Environment, Project};
use engine::registry::Registry;
use flate2::read::GzDecoder;
use futures_util::StreamExt;
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
use tokio::task::JoinSet;

pub struct ProductionServer {
    registry: Arc<Registry>,
    bridge_app: axum::Router,
    environment_variables: HashMap<String, String>,
    message_sender: MessageSender,
    federated_graph_config: Option<parser_sdl::federation::FederatedGraphConfig>,
}

impl ProductionServer {
    pub async fn build(
        message_sender: MessageSender,
        parallelism: NonZeroUsize,
        tracing: bool,
    ) -> Result<Self, ServerError> {
        create_project_dot_grafbase_directory()?;

        let environment_variables: HashMap<_, _> = crate::environment::variables().collect();
        let Config {
            registry,
            detected_udfs,
            federated_graph_config,
            ..
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
            environment_variables,
            message_sender,
            federated_graph_config,
        })
    }

    pub async fn serve(self, listen_address: IpAddr, port: u16) -> Result<(), ServerError> {
        let is_federated = self.federated_graph_config.is_some();

        if let Some(config) = self.federated_graph_config {
            let _ = self.message_sender.send(ServerMessage::Ready {
                listen_address,
                port,
                is_federated,
            });
            return federated_dev::run(port, true, constant_watch_receiver(config))
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
    port: PortSelection,
    watch: bool,
    tracing: bool,
    message_sender: MessageSender,
) -> Result<(), ServerError> {
    let project = Project::get();

    // Exporting Pathfinder, TS parser & miniflare for resolvers.
    export_embedded_files()?;
    create_project_dot_grafbase_directory()?;

    let proxy = proxy::start(port).await?;

    let watcher = if watch {
        Some(Watcher::start(project.path.clone()).await?)
    } else {
        None
    };

    let file_changes = watcher.as_ref().map(Watcher::file_changes);
    let config = ConfigActor::new(file_changes.clone(), message_sender.clone()).await;
    let is_federated = is_config_federated(&config, message_sender.clone()).await?;

    if is_federated {
        federated_dev(proxy, message_sender, config).await?;
    } else {
        if let Some(file_changes) = file_changes {
            crate::codegen_server::start_codegen_worker(file_changes, config.config_stream(), message_sender.clone())
                .expect("Invariant violation: codegen worker started twice.");
        }

        standalone_dev(proxy, message_sender, config, tracing).await?;
    }

    if let Some(watcher) = watcher {
        // Shutdown the watcher - any errors that occurred in the watcher should end up raised here
        watcher.shutdown().await?;
    }

    Ok(())
}

async fn federated_dev(
    mut proxy: ProxyHandle,
    message_sender: MessageSender,
    config: ConfigActor,
) -> Result<(), ServerError> {
    let worker_port = get_random_port_unchecked().await?;
    WORKER_PORT.store(worker_port, Ordering::Relaxed);
    message_sender
        .send(ServerMessage::Ready {
            listen_address: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: proxy.port,
            is_federated: true,
        })
        .ok();

    let server = federated_dev::run(worker_port, false, config.into_federated_config_receiver());

    tokio::select! {
        result = proxy.join() => {
            result.unwrap()??;
        },
        result = server => {
            result.map_err(|error| ServerError::GatewayError(error.to_string()))?;
        }
    }

    Ok(())
}

async fn is_config_federated(config: &ConfigActor, message_sender: MessageSender) -> Result<bool, ServerError> {
    let mut config_stream = config.result_stream();

    let mut join_set = JoinSet::new();
    while let Some(config) = config_stream.next().await {
        join_set.shutdown().await;
        match config {
            Ok(config) => {
                return Ok(config.federated_graph_config.is_some());
            }
            Err(error) => {
                join_set.spawn(handle_config_error(error.to_string(), message_sender.clone()));
            }
        }
    }

    // We should only get here if the watcher had a problem.
    // Just return false and let the rest of the mechanisms deal with it
    Ok(false)
}

async fn handle_config_error(
    error: String,
    message_sender: tokio::sync::mpsc::UnboundedSender<ServerMessage>,
) -> Result<(), ServerError> {
    message_sender.send(ServerMessage::CompilationError(error.clone())).ok();
    let worker_port = get_random_port_unchecked().await?;
    WORKER_PORT.store(worker_port, Ordering::Relaxed);
    error_server::start(worker_port, error).await
}

async fn standalone_dev(
    mut proxy: ProxyHandle,
    message_sender: MessageSender,
    mut config: ConfigActor,
    tracing: bool,
) -> Result<(), ServerError> {
    loop {
        let mut join_set = match config.current_result() {
            Ok(config) => spawn_servers(proxy.port, message_sender.clone(), config, tracing).await?,
            Err(error) => {
                let mut set = JoinSet::new();
                set.spawn(handle_config_error(error.to_string(), message_sender.clone()));
                set
            }
        };

        tokio::select! {
            result = config.changed() => {
                if result.is_err() {
                    // Watcher died - return and let the parent deal with it
                    return Ok(());
                }
            }
            result = join_set.join_next() => {
                result.expect("this set should not be empty")??;
            }
            result = proxy.join() => {
                result.unwrap()??;
                // if the proxy is dead we should exit.
                return Ok(())
            }
        }

        join_set.shutdown().await;
    }
}

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(level = "trace", skip(config))]
async fn spawn_servers(
    proxy_port: u16,
    message_sender: MessageSender,
    config: Config,
    tracing: bool,
) -> Result<JoinSet<Result<(), ServerError>>, ServerError> {
    let mut join_set = JoinSet::new();

    let environment_variables: HashMap<_, _> = crate::environment::variables().collect();

    let Config {
        registry,
        mut detected_udfs,
        triggering_file: path_changed,
        federated_graph_config: _,
    } = config;

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
            message_sender
                .send(ServerMessage::CompilationError(error.to_string()))
                .ok();

            // TODO consider disabling colored output from wrangler
            let error = strip_ansi_escapes::strip(error.to_string().as_bytes())
                .ok()
                .and_then(|stripped| String::from_utf8(stripped).ok())
                .unwrap_or_else(|| error.to_string());

            join_set.spawn(handle_config_error(error, message_sender.clone()));
            return Ok(join_set);
        }
    }

    let bridge_port = {
        let (listen, port) = get_listener_for_random_port().await?;
        let registry = Arc::clone(&registry);
        let message_sender = message_sender.clone();
        let (start_sender, started) = tokio::sync::oneshot::channel();

        trace!("starting bridge at port {port}");
        join_set.spawn(bridge::start(listen, message_sender, registry, start_sender, tracing));

        if started.await.is_err() {
            // The error is in the join_set which the layer above should listen for.
            return Ok(join_set);
        }

        trace!("bridge ready");

        port
    };

    let gateway_server = axum::Server::bind(&"127.0.0.1:0".parse().unwrap()).serve(
        gateway::Gateway::new(environment_variables, gateway::Bridge::new(bridge_port), registry)
            .await
            .map_err(|error| ServerError::GatewayError(error.to_string()))?
            .into_router()
            .into_make_service(),
    );

    WORKER_PORT.store(gateway_server.local_addr().port(), Ordering::Relaxed);
    join_set.spawn(async move { Ok(gateway_server.await?) });

    message_sender
        .send(ServerMessage::Ready {
            listen_address: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: proxy_port,
            is_federated: false,
        })
        .ok();

    Ok(join_set)
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

#[derive(Debug, Clone, Copy)]
pub enum PortSelection {
    Automatic { starting_at: u16 },
    Specific(u16),
}

impl PortSelection {
    pub async fn into_listener(self) -> Result<std::net::TcpListener, ServerError> {
        match self {
            PortSelection::Automatic { starting_at } => {
                find_listener_for_available_port_in_range(starting_at..MAX_PORT).await
            }
            PortSelection::Specific(port) => TcpListener::bind((Ipv4Addr::LOCALHOST, port))
                .await
                .map_err(|_| ServerError::PortInUse(port))?
                .into_std()
                .map_err(|_| ServerError::PortInUse(port)),
        }
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
