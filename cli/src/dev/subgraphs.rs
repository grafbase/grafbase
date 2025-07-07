use super::{
    FullGraphRef,
    data_json::{DataJsonError, DataJsonSchemas},
    extensions::detect_extensions,
};
use crate::{
    api::{
        client::create_client,
        graphql::queries::subgraph_schemas_by_branch::{
            Subgraph, SubgraphSchemasByBranch, SubgraphSchemasByBranchVariables,
        },
    },
    common::environment::PlatformData,
    errors::BackendError,
};
use chrono::{DateTime, Utc};
use cynic::{QueryBuilder, http::ReqwestExt};
use futures::TryStreamExt as _;
use gateway_config::{Config, SubgraphConfig};
use grafbase_graphql_introspection::introspect;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::PathBuf,
    sync::Arc,
};
use tokio::{
    fs,
    sync::{Mutex, mpsc},
};

const DEFAULT_BRANCH: &str = "main";

#[derive(Clone)]
pub(crate) struct CachedIntrospectedSubgraph {
    pub(crate) introspection_url: String,
    pub(crate) introspection_headers: Vec<(String, String)>,
    pub(crate) subgraph: Arc<CachedSubgraph>,
}

pub(crate) struct CachedSubgraph {
    pub(crate) name: String,
    pub(crate) sdl: String,
    pub(crate) url: Option<String>,
    pub(crate) owners: Option<Vec<SubgraphOwner>>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct SubgraphOwner {
    pub(crate) name: String,
}

pub(crate) struct SubgraphCache {
    current_dir: PathBuf,
    /// Urls from remote subgraphs (subgraphs fetched from the API with the graph ref).
    ///
    /// subgraph name -> subgraph url
    remote_urls: HashMap<String, Option<String>>,
    /// All remote subgraphs defined for the graph with the graph ref passed to the dev command.
    remote: Box<[Arc<CachedSubgraph>]>,
    /// Local subgraphs from introspection.
    pub(super) local_from_introspection: Mutex<BTreeMap<String, CachedIntrospectedSubgraph>>,
    /// All local subgraphs defined by path in configuration.
    local_from_file: Mutex<Box<[Arc<CachedSubgraph>]>>,
    /// For the app. Regenerated on every call to `compose()`.
    data_json_schemas: Mutex<(DateTime<Utc>, super::data_json::DataJsonSchemas)>,

    /// The handler for composition warnings.
    composition_warnings_sender: mpsc::Sender<Vec<String>>,
}

impl SubgraphCache {
    /// Construct a SubgraphCache, loading all remote and local subgraphs.
    pub(crate) async fn new(
        graph_ref: Option<&FullGraphRef>,
        config: &Config,
        composition_warnings_sender: mpsc::Sender<Vec<String>>,
    ) -> Result<SubgraphCache, BackendError> {
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
                .map(|subgraph| {
                    Arc::new(CachedSubgraph {
                        name: subgraph.name,
                        sdl: subgraph.schema,
                        url: subgraph.url,
                        owners: subgraph.owners.map(|owners| {
                            owners
                                .into_iter()
                                .map(|team| SubgraphOwner { name: team.name })
                                .collect()
                        }),
                    })
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
            data_json_schemas: Mutex::new((
                Utc::now(),
                super::data_json::DataJsonSchemas {
                    api_schema: None,
                    federated_schema: None,
                    subgraphs: vec![],
                    errors: vec![],
                },
            )),
            composition_warnings_sender,
            local_from_introspection: Default::default(),
            local_from_file: Default::default(),
        };

        subgraph_cache.reload_local_subgraphs(config).await?;

        Ok(subgraph_cache)
    }

    /// Execute a closure for each cached subgraph.
    pub(super) async fn for_each_subgraph(&self, mut f: impl FnMut(&Arc<CachedSubgraph>)) {
        let local_from_introspection = self.local_from_introspection.lock().await;
        let local_from_file = self.local_from_file.lock().await;

        let mut subgraphs_with_local_override: HashSet<&str> =
            HashSet::with_capacity(local_from_file.len() + local_from_introspection.len());

        for subgraph in local_from_file.as_ref() {
            subgraphs_with_local_override.insert(subgraph.name.as_str());
            f(subgraph);
        }

        for (_, subgraph) in local_from_introspection.iter() {
            let subgraph = &subgraph.subgraph;
            subgraphs_with_local_override.insert(subgraph.name.as_str());
            f(subgraph);
        }

        for subgraph in self
            .remote
            .as_ref()
            .iter()
            .filter(|subgraph| !subgraphs_with_local_override.contains(subgraph.name.as_str()))
        {
            f(subgraph);
        }
    }

    /// Compose all cached subgraphs.
    pub(crate) async fn compose(
        &self,
        config: &Config,
    ) -> anyhow::Result<Result<String, graphql_composition::Diagnostics>> {
        let mut futs = futures::stream::FuturesOrdered::new();
        let mut all_subgraphs = Vec::with_capacity(self.remote.len());

        self.for_each_subgraph(|subgraph| {
            all_subgraphs.push(subgraph.clone());

            futs.push_back({
                let current_dir = self.current_dir.clone();
                let subgraph = subgraph.clone();
                async move {
                    let current_dir = current_dir;

                    {
                        let diagnostics = graphql_schema_validation::validate(&subgraph.sdl);

                        if diagnostics.has_errors() {
                            return Err(anyhow::anyhow!(
                                "The schema of subgraph `{}` is invalid: {}",
                                subgraph.name,
                                diagnostics
                                    .iter()
                                    .map(|diagnostic| diagnostic.to_string())
                                    .collect::<Vec<_>>()
                                    .join("\n\n")
                            ));
                        }
                    }

                    let parsed_schema = cynic_parser::parse_type_system_document(&subgraph.sdl).map_err(|err| {
                        anyhow::anyhow!("Failed to parse subgraph SDL for `{}`: {err}", subgraph.name)
                    })?;

                    let extensions = detect_extensions(Some(&current_dir), &parsed_schema).await;

                    anyhow::Result::<_>::Ok((subgraph, parsed_schema, extensions))
                }
            });
        })
        .await;

        let mut stream = futs.into_stream();
        let mut subgraphs = graphql_composition::Subgraphs::default();

        subgraphs.ingest_loaded_extensions(config.extensions.iter().map(|(extension_name, extension)| {
            if let Some(path_url) = extension.path().and_then(|path| url::Url::from_file_path(path).ok()) {
                return graphql_composition::LoadedExtension::new(path_url.to_string(), extension_name.clone());
            }

            graphql_composition::LoadedExtension::new(
                format!(
                    "https://extensions.grafbase.com/{extension_name}/{}",
                    extension.version()
                ),
                extension_name.clone(),
            )
        }));

        while let Some((subgraph, parsed_schema, extensions)) = stream.try_next().await? {
            subgraphs.ingest_loaded_extensions(
                extensions
                    .into_iter()
                    .map(|ext| graphql_composition::LoadedExtension::new(ext.url, ext.name)),
            );

            subgraphs.ingest(&parsed_schema, &subgraph.name, subgraph.url.as_deref());
        }

        let result = graphql_composition::compose(&subgraphs);

        {
            let mut warnings = result.diagnostics().iter_warnings().peekable();

            if warnings.peek().is_some() {
                self.composition_warnings_sender
                    .send(warnings.map(ToOwned::to_owned).collect())
                    .await
                    .unwrap();
            }
        }

        let result = result.into_result();

        let (schemas, result) = match result {
            Ok(graph) => {
                let federated_schema = graphql_composition::render_federated_sdl(&graph)?;
                (
                    DataJsonSchemas {
                        api_schema: Some(graphql_composition::render_api_sdl(&graph)),
                        federated_schema: Some(federated_schema.clone()),
                        subgraphs: all_subgraphs,
                        errors: vec![],
                    },
                    Ok(federated_schema),
                )
            }
            Err(diagnostics) => (
                DataJsonSchemas {
                    api_schema: None,
                    federated_schema: None,
                    subgraphs: all_subgraphs,
                    errors: diagnostics
                        .iter_warnings()
                        .map(|warning| DataJsonError {
                            message: warning.to_owned(),
                            severity: "warning",
                        })
                        .chain(diagnostics.iter_errors().map(|err| DataJsonError {
                            message: err.to_owned(),
                            severity: "error",
                        }))
                        .collect(),
                },
                Err(diagnostics),
            ),
        };

        *self.data_json_schemas.lock().await = (Utc::now(), schemas);

        Ok(result)
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
                    local_from_file.push(Arc::new(cached_subgraph));
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

    pub(crate) async fn with_data_json_schemas<O>(&self, f: impl FnOnce(DateTime<Utc>, &DataJsonSchemas) -> O) -> O {
        let data_json_schemas = self.data_json_schemas.lock().await;
        let (updated_at, schemas) = &*data_json_schemas;
        f(*updated_at, schemas)
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
            owners: None,
        }))
    } else if let Some(introspection_url) = subgraph.introspection_url.as_ref().or(parsed_url.as_ref()) {
        let headers: Vec<(&String, &String)> = subgraph
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
                owners: None,
            }),
        }))
    } else {
        Err(BackendError::NoDefinedRouteToSubgraphSdl(name.to_owned()))
    }
}

async fn fetch_remote_subgraphs(graph_ref: &FullGraphRef) -> Result<Vec<Subgraph>, BackendError> {
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
