use super::{configurations::DevConfiguration, FullGraphRef};
use crate::api::{
    client::create_client,
    graphql::queries::subgraph_schemas_by_branch::{
        Subgraph, SubgraphSchemasByBranch, SubgraphSchemasByBranchVariables,
    },
};
use crate::errors::BackendError;
use common::environment::PlatformData;
use cynic::{http::ReqwestExt, QueryBuilder};
use grafbase_graphql_introspection::introspect;
use graphql_composition::Subgraphs;
use serde_dynamic_string::DynamicString;
use std::{
    collections::{BTreeMap, HashMap},
    env::set_current_dir,
    path::PathBuf,
};
use tokio::fs;

const DEFAULT_BRANCH: &str = "main";

pub struct SubgraphCache {
    pub remote: BTreeMap<&'static String, &'static Subgraph>,
}

pub async fn get_subgraph_sdls(
    graph_ref: Option<&FullGraphRef>,
    dev_configuration: &DevConfiguration,
    subgraphs: &mut Subgraphs,
    graph_overrides_path: Option<&PathBuf>,
) -> Result<SubgraphCache, BackendError> {
    let mut remote_urls: HashMap<&String, &String> = HashMap::new();
    let remote_subgraphs: Vec<Subgraph>;
    let mut subgraph_cache = SubgraphCache {
        remote: BTreeMap::new(),
    };

    if let Some(graph_ref) = graph_ref {
        remote_subgraphs = fetch_remote_subgraphs(graph_ref).await?;

        // these will live forever in the cache so no need to clone them
        // reloads do not supply a graph ref so this will only happen once
        let remote_subgraphs = Box::leak(Box::new(remote_subgraphs));

        for subgraph in remote_subgraphs.iter() {
            subgraph_cache.remote.insert(&subgraph.name, subgraph);
        }

        let remote_subgraphs = remote_subgraphs
            .iter()
            .filter(|subgraph| !dev_configuration.overridden_subgraphs.contains(&subgraph.name))
            .collect::<Vec<_>>();

        for subgraph in remote_subgraphs {
            remote_urls.insert(&subgraph.name, &subgraph.url);
            let url = if let Some(url) = dev_configuration
                .merged_configuration
                .subgraphs
                .get(&subgraph.name)
                .and_then(|subgraph| subgraph.url.as_ref())
            {
                url.as_str()
            } else {
                subgraph.url.as_str()
            };

            subgraphs
                .ingest_str(&subgraph.schema, &subgraph.name, url)
                .map_err(BackendError::IngestSubgraph)?;
        }
    }

    let remote_urls = &remote_urls;

    if let Some(graph_overrides_path) = graph_overrides_path {
        // switching the current directory to where the overrides config is located
        // as we want relative paths in `schema_path` to work correctly
        set_current_dir(
            fs::canonicalize(graph_overrides_path)
                .await
                .expect("must work")
                .parent()
                .expect("must exist"),
        )
        .map_err(BackendError::SetCurrentDirectory)?;
    }

    let futures = dev_configuration
        .overridden_subgraphs
        .iter()
        .map(|subgraph| (subgraph, &dev_configuration.merged_configuration.subgraphs[subgraph]))
        .map(|(name, subgraph)| async move {
            let Some(url) = subgraph
                .url
                .as_ref()
                .map(|url| url.as_str())
                .or_else(|| remote_urls.get(&name).map(|url| url.as_str()))
                .or(subgraph.introspection_url.as_ref().map(|url| url.as_str()))
            else {
                return Err(BackendError::NoDefinedRouteToSubgraphSdl(name.clone()));
            };

            if let Some(ref schema_path) = subgraph.schema_path {
                let sdl = fs::read_to_string(schema_path)
                    .await
                    .map_err(|error| BackendError::ReadSdlFromFile(schema_path.clone(), error))?;

                Ok((sdl, name, url))
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
            .ingest_str(&sdl, name, url)
            .map_err(BackendError::IngestSubgraph)?;
    }

    Ok(subgraph_cache)
}

async fn fetch_remote_subgraphs(graph_ref: &FullGraphRef) -> Result<Vec<Subgraph>, BackendError> {
    let platform_data = PlatformData::get();

    let client = create_client().await.map_err(BackendError::ApiError)?;

    let branch = graph_ref.branch.as_deref().unwrap_or(DEFAULT_BRANCH);

    let query = SubgraphSchemasByBranch::build(SubgraphSchemasByBranchVariables {
        account_slug: &graph_ref.account,
        name: branch,
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

    Ok(branch.subgraphs)
}
