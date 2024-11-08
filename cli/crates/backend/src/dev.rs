use super::errors::BackendError;
use crate::api::{
    client::create_client,
    graphql::queries::subgraph_schemas_by_branch::{SubgraphSchemasByBranch, SubgraphSchemasByBranchVariables},
};
use common::environment::PlatformData;
use cynic::{http::ReqwestExt, QueryBuilder};
use federated_server::{serve, GraphFetchMethod, ServerConfig, ServerRuntime};
use gateway_config::Config;
use grafbase_graphql_introspection::introspect;
use graphql_composition::Subgraphs;
use serde_dynamic_string::DynamicString;
use serde_toml_merge::merge;
use std::{
    collections::{HashMap, HashSet},
    env::set_current_dir,
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
};
use tokio::fs;
use tokio::sync::mpsc::{channel, Receiver, Sender};

#[derive(Clone, Debug)]
pub struct ProjectRef {
    pub account: String,
    pub graph: String,
    pub branch: Option<String>,
}

const DEFAULT_BRANCH: &str = "main";
const DEFAULT_PORT: u16 = 5000;

#[derive(Clone)]
struct CliRuntime {
    ready_sender: Sender<String>,
}

impl ServerRuntime for CliRuntime {
    fn after_request(&self) {}

    fn on_ready(&self, url: String) {
        let sender = self.ready_sender.clone();
        tokio::spawn(async move { sender.send(url).await.expect("must still be open") });
    }
}

#[tokio::main(flavor = "multi_thread")]
pub async fn start(
    graph_ref: Option<ProjectRef>,
    gateway_config_path: Option<PathBuf>,
    graph_overrides_path: Option<PathBuf>,
    port: Option<u16>,
) -> Result<(), BackendError> {
    let (ready_sender, mut ready_receiver) = channel::<String>(1);

    tokio::spawn(async move { output_handler(&mut ready_receiver).await });

    let dev_configuration = get_and_merge_configurations(&gateway_config_path, &graph_overrides_path).await?;

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

    get_subgraph_sdls(graph_ref, &dev_configuration, &mut subgraphs, graph_overrides_path).await?;

    let composition_result = graphql_composition::compose(&subgraphs);

    let federated_sdl = match composition_result.into_result() {
        Ok(result) => result.into_federated_sdl().map_err(BackendError::ToFederatedSdl)?,
        Err(diagnostics) => {
            return Err(BackendError::Composition(
                diagnostics.iter_messages().collect::<Vec<_>>().join("\n"),
            ))
        }
    };

    let server_config = ServerConfig {
        listen_addr: Some(listen_address),
        config_path: None,
        config_hot_reload: false,
        config: dev_configuration.merged_configuration,
        fetch_method: GraphFetchMethod::FromSchema { federated_sdl },
    };

    serve(server_config, CliRuntime { ready_sender })
        .await
        .map_err(BackendError::Serve)?;

    Ok(())
}

struct DevConfiguration {
    overridden_subgraphs: HashSet<String>,
    merged_configuration: Config,
}

async fn get_and_merge_configurations(
    gateway_config_path: &Option<PathBuf>,
    graph_overrides_path: &Option<PathBuf>,
) -> Result<DevConfiguration, BackendError> {
    // TODO: hard error if `file` or introspection fields used in normal config

    let config_value = if let Some(ref gateway_config_path) = gateway_config_path {
        let gateway_config_value = fs::read_to_string(gateway_config_path)
            .await
            .map_err(BackendError::ReadGatewayConfig)?
            .parse::<toml::Value>()
            .map_err(|error| BackendError::ParseGatewayConfig(error.to_string()))?;

        Some(gateway_config_value)
    } else {
        None
    };

    let (graph_overrides, overrides_value): (Option<Config>, Option<toml::Value>) =
        if let Some(ref graph_overrides_path) = graph_overrides_path {
            let graph_overrides_value = fs::read_to_string(graph_overrides_path)
                .await
                .map_err(BackendError::ReadGraphOverrides)?
                .parse::<toml::Value>()
                .map_err(BackendError::ParseGraphOverrides)?;

            let graph_overrides = graph_overrides_value
                .clone()
                .try_into()
                .map_err(BackendError::ParseGraphOverrides)?;

            (Some(graph_overrides), Some(graph_overrides_value))
        } else {
            (None, None)
        };

    let merged_configuration = if overrides_value.is_none() {
        if let Some(value) = config_value {
            value
                .try_into()
                .map_err(|error| BackendError::ParseGatewayConfig(error.to_string()))?
        } else {
            Config::default()
        }
    } else {
        config_value
            .zip(overrides_value)
            .map(|(config, overrides)| merge(config, overrides))
            .transpose()
            .map_err(|_| BackendError::MergeConfigurations)?
            .map(|config| config.try_into::<Config>())
            .transpose()
            // as we have already successfully converted the graph overrides into a Config at this point
            // an error here would happen due to the gateway config having an incorrect structure
            .map_err(|error| BackendError::ParseGatewayConfig(error.to_string()))?
            .unwrap_or_default()
    };

    let overridden_subgraphs = graph_overrides
        .map(|config| config.subgraphs.into_keys().collect::<HashSet<_>>())
        .unwrap_or_default();

    Ok(DevConfiguration {
        overridden_subgraphs,
        merged_configuration,
    })
}

async fn get_subgraph_sdls(
    graph_ref: Option<ProjectRef>,
    dev_configuration: &DevConfiguration,
    subgraphs: &mut Subgraphs,
    graph_overrides_path: Option<PathBuf>,
) -> Result<(), BackendError> {
    let mut remote_urls: HashMap<String, String> = HashMap::new();

    if let Some(graph_ref) = graph_ref {
        let platform_data = PlatformData::get();

        let client = create_client().await.map_err(BackendError::ApiError)?;

        let branch = &graph_ref.branch.unwrap_or(DEFAULT_BRANCH.to_owned());

        // TODO: cache when we have hot reloading
        // TODO: we should not request subgraphs that are overridden
        let query = SubgraphSchemasByBranch::build(SubgraphSchemasByBranchVariables {
            account_slug: &graph_ref.account,
            name: branch.as_str(),
            graph_slug: &graph_ref.graph,
        });

        let response = client
            .post(&platform_data.api_url)
            .run_graphql(query)
            .await
            .map_err(|error| BackendError::ApiError(error.into()))?;

        let branch = response
            .data
            .ok_or(BackendError::FetchBranch)?
            .branch
            .ok_or(BackendError::BranchDoesntExist)?;

        let remote_subgraphs = branch
            .subgraphs
            .into_iter()
            .filter(|subgraph| !dev_configuration.overridden_subgraphs.contains(&subgraph.name))
            .collect::<Vec<_>>();

        for subgraph in remote_subgraphs {
            remote_urls.insert(subgraph.name.clone(), subgraph.url.clone());
            let url = if let Some(url) = dev_configuration
                .merged_configuration
                .subgraphs
                .get(&subgraph.name)
                .and_then(|subgraph| subgraph.url.as_ref())
            {
                url.to_string()
            } else {
                subgraph.url
            };

            subgraphs
                .ingest_str(&subgraph.schema, &subgraph.name, &url)
                .map_err(BackendError::IngestSubgraph)?;
        }
    }

    let remote_urls = &remote_urls;
    let graph_overrides_path = &graph_overrides_path;

    let futures = dev_configuration
        .overridden_subgraphs
        .iter()
        .map(|subgraph| (subgraph, &dev_configuration.merged_configuration.subgraphs[subgraph]))
        .map(|(name, subgraph)| async move {
            let Some(url) = subgraph
                .url
                .as_ref()
                .map(|url| url.to_string())
                .or_else(|| remote_urls.get(name).cloned())
                .or(subgraph.introspection_url.as_ref().map(|url| url.to_string()))
            else {
                return Err(BackendError::NoDefinedRouteToSubgraphSdl(name.clone()));
            };

            if let Some(ref schema_file_path) = subgraph.file {
                // switching the current directory to where the overrides config is located
                // as we want relative paths in `file` to work correctly
                set_current_dir(
                    fs::canonicalize(graph_overrides_path.clone().expect("must exist if `file` is in use"))
                        .await
                        .expect("must work")
                        .parent()
                        .expect("must exist"),
                )
                .map_err(|error| BackendError::ReadSdlFromFile(schema_file_path.clone(), error))?;

                let sdl = fs::read_to_string(schema_file_path)
                    .await
                    .map_err(|error| BackendError::ReadSdlFromFile(schema_file_path.clone(), error))?;

                Ok((sdl, name, url.to_string()))
            } else if let Some(ref introspection_url) = subgraph.introspection_url {
                let headers: Vec<(&String, &DynamicString<String>)> = subgraph
                    .introspection_headers
                    .as_ref()
                    .map(|intropection_headers| intropection_headers.iter().collect())
                    .unwrap_or_default();
                // TODO: this also parses and prettifies, expose internal functionality
                let sdl = introspect(introspection_url.as_str(), &headers)
                    .await
                    .map_err(|_| BackendError::IntrospectSubgraph(introspection_url.to_string()))?;

                Ok((sdl, name, url))
            } else {
                Err(BackendError::NoDefinedRouteToSubgraphSdl(name.clone()))
            }
        });

    let results = futures::future::try_join_all(futures).await?;

    for (sdl, name, url) in results {
        subgraphs
            .ingest_str(&sdl, name, &url)
            .map_err(BackendError::IngestSubgraph)?;
    }

    Ok(())
}

// temporary output handler for internal testing until we move output to the CLI and use a proper terminal crate.
// none of us uses Windows, right?
async fn output_handler(receiver: &mut Receiver<String>) {
    // gray
    println!("\x1b[90mWarning: This command is in beta, expect missing features, bugs or breaking changes\x1b[0m\n");

    // yellow and bold
    println!("🕒 \x1b[1;33mFetching\x1b[0m your subgraphs...\n");

    let Some(url) = receiver.recv().await else {
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
