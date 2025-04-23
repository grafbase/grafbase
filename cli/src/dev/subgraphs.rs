use super::FullGraphRef;
use crate::common::environment::PlatformData;
use crate::dev::detect_extensions;
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
use serde_dynamic_string::DynamicString;
use std::collections::HashSet;
use std::path::PathBuf;
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};
use tokio::{fs, sync::Mutex};

const DEFAULT_BRANCH: &str = "main";

#[derive(Clone)]
pub(crate) struct CachedIntrospectedSubgraph {
    pub(crate) introspection_url: String,
    pub(crate) introspection_headers: Vec<(String, DynamicString<String>)>,
    pub(crate) subgraph: Arc<CachedSubgraph>,
}

#[derive(Default)]
pub(crate) struct CachedSubgraph {
    pub(crate) name: String,
    pub(crate) sdl: String,
    pub(crate) url: Option<String>,
}

pub(crate) struct SubgraphCache {
    current_dir: PathBuf,
    /// Urls from remote subgraphs (subgraphs fetched from the API with the graph ref).
    ///
    /// subgraph name -> subgraph url
    remote_urls: HashMap<String, Option<String>>,
    /// All remote subgraphs defined for the graph with the graph ref passed to the dev command.
    remote: Box<[CachedSubgraph]>,
    /// Local subgraphs from introspection.
    pub(super) local_from_introspection: Mutex<BTreeMap<String, CachedIntrospectedSubgraph>>,
    /// All local subgraphs defined by path in configuration.
    local_from_file: Mutex<Box<[CachedSubgraph]>>,
}

impl SubgraphCache {
    /// Construct a SubgraphCache, loading all remote and local subgraphs.
    pub(crate) async fn new(graph_ref: Option<&FullGraphRef>, config: &Config) -> Result<SubgraphCache, BackendError> {
        // subgraph name -> subgraph url
        let mut remote_urls: HashMap<String, Option<String>> = HashMap::new();

        let current_dir = std::env::current_dir()
            .map_err(|error| BackendError::Error(format!("Failed to get current directory: {error}")))?;

        let remote = if let Some(graph_ref) = graph_ref {
            let all_remote_subgraphs = fetch_remote_subgraphs(graph_ref).await?;

            let remote_subgraphs = all_remote_subgraphs.iter().filter(|subgraph| {
                !config
                    .subgraphs
                    .get(&subgraph.name)
                    .map(|cfg| cfg.has_schema_override())
                    .unwrap_or_default()
            });

            for subgraph in remote_subgraphs {
                remote_urls.insert(subgraph.name.clone(), subgraph.url.clone());
            }

            let all_remote_subgraphs = all_remote_subgraphs
                .into_iter()
                .map(|subgraph| CachedSubgraph {
                    name: subgraph.name,
                    sdl: subgraph.schema,
                    url: subgraph.url,
                })
                .collect::<Vec<_>>();

            all_remote_subgraphs.into_boxed_slice()
        } else {
            Default::default()
        };

        let subgraph_cache = SubgraphCache {
            current_dir,
            remote_urls,
            remote,
            local_from_introspection: Default::default(),
            local_from_file: Default::default(),
        };

        subgraph_cache.reload_local_subgraphs(config).await?;

        Ok(subgraph_cache)
    }

    /// Compose all cached subgraphs.
    pub(crate) async fn compose(&self) -> Result<graphql_composition::CompositionResult, BackendError> {
        let local_from_introspection = self.local_from_introspection.lock().await;
        let local_from_file = self.local_from_file.lock().await;

        let mut subgraphs_with_local_override: HashSet<&str> =
            HashSet::with_capacity(local_from_file.len() + local_from_introspection.len());
        let mut subgraphs = graphql_composition::Subgraphs::default();

        for subgraph in local_from_file.as_ref() {
            subgraphs_with_local_override.insert(subgraph.name.as_str());
            self.ingest_cached_subgraph(subgraph, &mut subgraphs).await?;
        }

        for (_, subgraph) in local_from_introspection.iter() {
            let subgraph = &subgraph.subgraph;
            subgraphs_with_local_override.insert(subgraph.name.as_str());
            self.ingest_cached_subgraph(subgraph, &mut subgraphs).await?;
        }

        for subgraph in self
            .remote
            .as_ref()
            .iter()
            .filter(|subgraph| !subgraphs_with_local_override.contains(subgraph.name.as_str()))
        {
            self.ingest_cached_subgraph(subgraph, &mut subgraphs).await?;
        }

        Ok(graphql_composition::compose(&subgraphs))
    }

    /// Helper for [SubgraphCache::compose()].
    async fn ingest_cached_subgraph(
        &self,
        cached_subgraph: &CachedSubgraph,
        subgraphs: &mut graphql_composition::Subgraphs,
    ) -> Result<(), BackendError> {
        let parsed_schema =
            cynic_parser::parse_type_system_document(&cached_subgraph.sdl).map_err(BackendError::ParseSubgraphSdl)?;

        let extensions = detect_extensions(Some(&self.current_dir), &parsed_schema).await;

        subgraphs.ingest_loaded_extensions(
            extensions
                .into_iter()
                .map(|ext| graphql_composition::LoadedExtension::new(ext.url, ext.name)),
        );

        subgraphs.ingest(&parsed_schema, &cached_subgraph.name, cached_subgraph.url.as_deref());

        Ok(())
    }

    /// Reload local subgraphs after a configuration or schema change.
    pub(super) async fn reload_local_subgraphs(&self, config: &Config) -> Result<(), BackendError> {
        let futures = config
            .subgraphs
            .iter()
            .filter(|(_, subgraph)| subgraph.has_schema_override())
            .map(|(name, subgraph)| handle_overridden_subgraph(&self.remote_urls, name, subgraph));

        let results = futures::future::try_join_all(futures).await?;

        let mut local_from_file = Vec::new();
        let mut local_from_introspection = BTreeMap::new();

        for overridden_subgraph in results {
            match overridden_subgraph {
                OverriddenSubgraph::FromFile(cached_subgraph) => {
                    local_from_file.push(cached_subgraph);
                }
                OverriddenSubgraph::FromIntrospection(cached_introspected_subgraph) => {
                    local_from_introspection.insert(
                        cached_introspected_subgraph.subgraph.name.clone(),
                        cached_introspected_subgraph,
                    );
                }
            }
        }

        *self.local_from_introspection.lock().await = local_from_introspection;
        *self.local_from_file.lock().await = local_from_file.into_boxed_slice();

        Ok(())
    }
}

enum OverriddenSubgraph {
    FromFile(CachedSubgraph),
    FromIntrospection(CachedIntrospectedSubgraph),
}

async fn handle_overridden_subgraph(
    remote_urls: &HashMap<String, Option<String>>,
    name: &str,
    subgraph: &SubgraphConfig,
) -> Result<OverriddenSubgraph, BackendError> {
    let url = subgraph
        .url
        .as_ref()
        .map(|url| url.as_str())
        .or_else(|| remote_urls.get(name).and_then(|url| url.as_deref()))
        .or(subgraph.introspection_url.as_ref().map(|url| url.as_str()))
        .map(String::from);

    let parsed_url = url.as_ref().and_then(|url| reqwest::Url::parse(url).ok());

    if let Some(ref schema_path) = subgraph.schema_path {
        let sdl = fs::read_to_string(schema_path)
            .await
            .map_err(|error| BackendError::ReadSdlFromFile(schema_path.clone(), error))?;

        Ok(OverriddenSubgraph::FromFile(CachedSubgraph {
            name: name.to_owned(),
            sdl,
            url,
        }))
    } else if let Some(introspection_url) = subgraph.introspection_url.as_ref().or(parsed_url.as_ref()) {
        let headers: Vec<(&String, &DynamicString<String>)> = subgraph
            .introspection_headers
            .as_ref()
            .map(|intropection_headers| intropection_headers.iter().collect())
            .unwrap_or_default();

        let sdl = introspect(introspection_url.as_str(), &headers)
            .await
            .map_err(|_| BackendError::IntrospectSubgraph(introspection_url.to_string()))?;

        Ok(OverriddenSubgraph::FromIntrospection(CachedIntrospectedSubgraph {
            introspection_url: introspection_url.to_string(),
            introspection_headers: headers
                .iter()
                .map(|(key, value)| ((*key).clone(), (*value).clone()))
                .collect(),
            subgraph: Arc::new(CachedSubgraph {
                name: name.to_owned(),
                sdl,
                url: url.clone(),
            }),
        }))
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
