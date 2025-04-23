use super::{FullGraphRef, extensions::*};
use crate::common::environment::PlatformData;
use crate::{
    api::{
        client::create_client,
        graphql::queries::subgraph_schemas_by_branch::{
            Subgraph, SubgraphSchemasByBranch, SubgraphSchemasByBranchVariables,
        },
    },
    errors::BackendError,
};
use cynic::{QueryBuilder, http::ReqwestExt};
use gateway_config::{Config, SubgraphConfig};
use grafbase_graphql_introspection::introspect;
use graphql_composition as composition;
use serde_dynamic_string::DynamicString;
use std::{
    collections::{BTreeMap, HashMap},
    path::Path,
    sync::Arc,
};
use tokio::{fs, sync::Mutex};

const DEFAULT_BRANCH: &str = "main";

#[derive(Clone)]
pub struct CachedIntrospectedSubgraph {
    pub introspection_url: String,
    pub introspection_headers: Vec<(String, DynamicString<String>)>,
    pub sdl: String,
}

pub struct SubgraphCache {
    pub remote: BTreeMap<&'static String, &'static Subgraph>,
    pub local: Mutex<BTreeMap<String, CachedIntrospectedSubgraph>>,
}

pub async fn get_subgraph_sdls(
    graph_ref: Option<&FullGraphRef>,
    config: &Config,
    subgraphs: &mut composition::Subgraphs,
) -> Result<Arc<SubgraphCache>, BackendError> {
    let mut remote_urls: HashMap<&str, Option<&str>> = HashMap::new();
    let remote_subgraphs: Vec<Subgraph>;
    let mut subgraph_cache = SubgraphCache {
        remote: BTreeMap::new(),
        local: Mutex::new(BTreeMap::new()),
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
            .filter(|subgraph| {
                !config
                    .subgraphs
                    .get(&subgraph.name)
                    .map(|cfg| cfg.has_schema_override())
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>();

        for subgraph in remote_subgraphs {
            remote_urls.insert(&subgraph.name, subgraph.url.as_deref());
            let url = if let Some(url) = config
                .subgraphs
                .get(&subgraph.name)
                .and_then(|subgraph| subgraph.url.as_ref())
            {
                Some(url.as_str())
            } else {
                subgraph.url.as_deref()
            };

            let parsed_sdl = cynic_parser::parse_type_system_document(&subgraph.schema)?;
            subgraphs.ingest(&parsed_sdl, &subgraph.name, url);
        }
    }

    let remote_urls = &remote_urls;

    let current_dir = std::env::current_dir()
        .map_err(|error| BackendError::Error(format!("Failed to get current directory: {error}")))?;
    let subgraph_cache = Arc::new(subgraph_cache);
    let futures = config
        .subgraphs
        .iter()
        .filter(|(_, subgraph)| subgraph.has_schema_override())
        .map(|(name, subgraph)| {
            let subgraph_cache = subgraph_cache.clone();
            handle_overridden_subgraph(&current_dir, subgraph_cache, remote_urls, name, subgraph)
        });

    let results = futures::future::try_join_all(futures).await?;

    for OverriddenSubgraph {
        parsed_schema,
        url,
        name,
        extensions,
    } in results
    {
        subgraphs.ingest(&parsed_schema, &name, url.as_deref());

        let extensions = extensions
            .into_iter()
            .map(|extension| composition::LoadedExtension::new(extension.url, extension.name));
        subgraphs.ingest_loaded_extensions(extensions);
    }

    Ok(subgraph_cache)
}

struct OverriddenSubgraph {
    parsed_schema: cynic_parser::TypeSystemDocument,
    url: Option<String>,
    name: String,
    extensions: Vec<DetectedExtension>,
}

async fn handle_overridden_subgraph(
    current_dir: &Path,
    subgraph_cache: Arc<SubgraphCache>,
    remote_urls: &HashMap<&str, Option<&str>>,
    name: &str,
    subgraph: &SubgraphConfig,
) -> Result<OverriddenSubgraph, BackendError> {
    let url = subgraph
        .url
        .as_ref()
        .map(|url| url.as_str())
        .or_else(|| remote_urls.get(name).copied().flatten())
        .or(subgraph.introspection_url.as_ref().map(|url| url.as_str()))
        .map(String::from);

    let parsed_url = url.as_ref().and_then(|url| reqwest::Url::parse(url).ok());

    if let Some(ref schema_path) = subgraph.schema_path {
        let sdl = fs::read_to_string(schema_path)
            .await
            .map_err(|error| BackendError::ReadSdlFromFile(schema_path.clone(), error))?;

        let parsed_schema = cynic_parser::parse_type_system_document(&sdl).map_err(BackendError::ParseSubgraphSdl)?;

        let extensions = detect_extensions(Some(current_dir), &parsed_schema).await;

        Ok(OverriddenSubgraph {
            parsed_schema,
            url,
            name: name.to_owned(),
            extensions,
        })
    } else if let Some(introspection_url) = subgraph.introspection_url.as_ref().or(parsed_url.as_ref()) {
        let headers: Vec<(&String, &DynamicString<String>)> = subgraph
            .introspection_headers
            .as_ref()
            .map(|intropection_headers| intropection_headers.iter().collect())
            .unwrap_or_default();

        // TODO: this also parses and prettifies, expose internal functionality
        let sdl = introspect(introspection_url.as_str(), &headers)
            .await
            .map_err(|_| BackendError::IntrospectSubgraph(introspection_url.to_string()))?;

        subgraph_cache.local.lock().await.insert(
            name.to_owned(),
            CachedIntrospectedSubgraph {
                introspection_url: introspection_url.to_string(),
                introspection_headers: headers
                    .iter()
                    .map(|(key, value)| ((*key).clone(), (*value).clone()))
                    .collect(),
                sdl: sdl.clone(),
            },
        );

        let parsed_schema = cynic_parser::parse_type_system_document(&sdl)?;
        let extensions = detect_extensions(None, &parsed_schema).await;

        Ok(OverriddenSubgraph {
            parsed_schema,
            url,
            name: name.to_owned(),
            extensions,
        })
    } else {
        Err(BackendError::NoDefinedRouteToSubgraphSdl(name.to_owned()))
    }
}

pub(crate) async fn fetch_remote_subgraphs(graph_ref: &FullGraphRef) -> Result<Vec<Subgraph>, BackendError> {
    let platform_data = PlatformData::get();

    let client = create_client().map_err(BackendError::ApiError)?;

    let branch = graph_ref.branch().unwrap_or(DEFAULT_BRANCH);

    let query = SubgraphSchemasByBranch::build(SubgraphSchemasByBranchVariables {
        account_slug: graph_ref.account(),
        name: branch,
        graph_slug: graph_ref.graph(),
    });

    let response = client
        .post(platform_data.api_url())
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
