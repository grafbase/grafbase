use crate::api::{
    client::create_client,
    graphql::queries::subgraph_schemas_by_branch::{SubgraphSchemasByBranch, SubgraphSchemasByBranchVariables},
};

use super::errors::BackendError;
use common::{environment::PlatformData, time};
use cynic::{http::ReqwestExt, QueryBuilder};
use federated_server::{serve, GraphFetchMethod, ServerConfig};
use gateway_config::Config;
use serde_toml_merge::merge;
use std::{
    collections::HashSet,
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
};
use tokio::fs;

#[derive(Clone, Debug)]
pub struct ProjectRef {
    pub account: String,
    pub graph: String,
    pub branch: Option<String>,
}

#[tokio::main(flavor = "multi_thread")]
pub async fn start(
    graph_ref: Option<ProjectRef>,
    gateway_config: Option<PathBuf>,
    graph_overrides: Option<PathBuf>,
) -> Result<(), BackendError> {
    let default_listen_address = SocketAddr::from((Ipv4Addr::LOCALHOST, 5000));

    let dev_configuration = get_and_merge_configurations(gateway_config, graph_overrides).await?;

    get_subgraph_sdls(graph_ref, &dev_configuration).await?;

    let server_config = ServerConfig {
        listen_addr: Some(default_listen_address),
        config_path: None,
        config_hot_reload: false,
        config: dev_configuration.merged_configuration,
        fetch_method: GraphFetchMethod::FromSchema {
            federated_sdl: "".to_owned(),
        },
    };

    serve(server_config, ()).await.unwrap();

    Ok(())
}

struct DevConfiguration {
    overridden_subgraphs: HashSet<String>,
    merged_configuration: Config,
}

async fn get_and_merge_configurations(
    gateway_config_path: Option<PathBuf>,
    graph_overrides_path: Option<PathBuf>,
) -> Result<DevConfiguration, BackendError> {
    let config_value = if let Some(ref gateway_config_path) = gateway_config_path {
        let gateway_config_value = fs::read_to_string(gateway_config_path)
            .await
            .map_err(BackendError::ReadGatewayConfig)?
            .parse::<toml::Value>()
            .map_err(|_| BackendError::ParseGatewayConfig)?;

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
                .map_err(|_| BackendError::ParseGraphOverrides)?;

            let graph_overrides = graph_overrides_value
                .clone()
                .try_into()
                .map_err(|_| BackendError::ParseGraphOverrides)?;

            (Some(graph_overrides), Some(graph_overrides_value))
        } else {
            (None, None)
        };

    let merged_configuration = config_value
        .zip(overrides_value)
        .map(|(config, overrides)| merge(config, overrides))
        .transpose()
        .map_err(|_| BackendError::MergeConfigurations)?
        .map(|config| config.try_into::<Config>())
        .transpose()
        // as we have already successfully converted the graph overrides into a Config at this point
        // an error here would happen due to the gateway config having an incorrect structure
        .map_err(|_| BackendError::ParseGatewayConfig)?
        .unwrap_or_default();

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
) -> Result<(), BackendError> {
    let remote_subgraphs: Vec<String> = if let Some(graph_ref) = graph_ref {
        let platform_data = PlatformData::get();

        let client = create_client().await.map_err(BackendError::ApiError)?;

        let branch = &graph_ref.branch.unwrap_or("main".to_owned());

        // TODO: cache
        // TODO: we should not request subgraphs that are overridden
        let query = SubgraphSchemasByBranch::build(SubgraphSchemasByBranchVariables {
            account_slug: &graph_ref.account,
            name: branch.as_str(),
            graph_slug: &graph_ref.graph,
        });

        let response = time!(client
            .post(&platform_data.api_url)
            .run_graphql(query)
            .await
            .map_err(|error| BackendError::ApiError(error.into()))?);

        response
            .data
            .unwrap()
            .branch
            .unwrap()
            .subgraphs
            .into_iter()
            .filter(|subgraph| !dev_configuration.overridden_subgraphs.contains(&subgraph.name))
            .map(|subgraph| subgraph.schema)
            .collect::<Vec<_>>()
    } else {
        vec![]
    };

    for subgraph in remote_subgraphs {
        println!("{}", subgraph.len())
    }

    // let overriden_subgraphs = dev_configuration
    //     .overridden_subgraphs
    //     .iter()
    //     .map(|subgraph| subgraph.to_owned())
    //     .collect::<Vec<String>>();

    Ok(())
}
