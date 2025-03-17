use std::{collections::HashMap, future::Future, ops::Range, sync::Arc};

use engine::{ErrorResponse, GraphqlError};
use engine_schema::{DirectiveSite, Subgraph, SubgraphId};
use extension_catalog::{Extension, ExtensionCatalog, ExtensionId, Id, Manifest};
use futures::{
    TryFutureExt as _, TryStreamExt as _,
    stream::{BoxStream, FuturesUnordered},
};
use runtime::{
    extension::{
        AuthorizationDecisions, Data, ExtensionFieldDirective, QueryAuthorizationDecisions, QueryElement, Token,
        TokenRef,
    },
    hooks::{Anything, DynHookContext},
};
use serde::Serialize;
use tokio::sync::Mutex;
use url::Url;

pub struct TestExtensions {
    pub tmpdir: tempfile::TempDir,
    catalog: ExtensionCatalog,
    builders: HashMap<ExtensionId, Box<dyn TestExtensionBuilder>>,
    global_instances: Mutex<HashMap<ExtensionId, Arc<dyn TestExtension>>>,
    subgraph_instances: Mutex<HashMap<(ExtensionId, SubgraphId), Arc<dyn TestExtension>>>,
}

impl Default for TestExtensions {
    fn default() -> Self {
        Self {
            tmpdir: tempfile::tempdir().unwrap(),
            catalog: Default::default(),
            builders: Default::default(),
            global_instances: Default::default(),
            subgraph_instances: Default::default(),
        }
    }
}

impl TestExtensions {
    #[track_caller]
    pub fn push_extension<Builder: TestExtensionBuilder + Sized>(&mut self, builder: Builder) {
        let config = builder.config();

        let manifest = extension_catalog::Manifest {
            id: builder.id(),
            kind: config.kind,
            sdk_version: "0.0.0".parse().unwrap(),
            minimum_gateway_version: "0.0.0".parse().unwrap(),
            sdl: config.sdl.map(str::to_string),
            description: "Test extension".to_owned(),
            homepage_url: None,
            license: None,
            readme: None,
            repository_url: None,
            permissions: Default::default(),
        };

        let dir = self.tmpdir.path().join(manifest.id.to_string());
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(
            dir.join("manifest.json"),
            serde_json::to_vec(&manifest.clone().into_versioned()).unwrap(),
        )
        .unwrap();
        let id = self.catalog.push(Extension {
            manifest,
            wasm_path: Default::default(),
        });
        self.builders.insert(id, Box::new(builder));
    }

    pub fn catalog(&self) -> &ExtensionCatalog {
        &self.catalog
    }

    pub fn iter(&self) -> impl Iterator<Item = (Url, &Manifest)> {
        self.catalog.iter().map(move |ext| {
            (
                Url::from_file_path(self.tmpdir.path().join(ext.manifest.id.to_string())).unwrap(),
                &ext.manifest,
            )
        })
    }

    async fn get_subgraph_isntance(&self, extension_id: ExtensionId, subgraph: Subgraph<'_>) -> Arc<dyn TestExtension> {
        self.subgraph_instances
            .lock()
            .await
            .entry((extension_id, subgraph.id()))
            .or_insert_with(|| {
                self.builders.get(&extension_id).unwrap().build(
                    subgraph
                        .extension_schema_directives()
                        .filter(|dir| dir.extension_id == extension_id)
                        .map(|dir| (dir.name(), serde_json::to_value(dir.static_arguments()).unwrap()))
                        .collect(),
                )
            })
            .clone()
    }

    async fn get_global_instance(&self, extension_id: ExtensionId) -> Arc<dyn TestExtension> {
        self.global_instances
            .lock()
            .await
            .entry(extension_id)
            .or_insert_with(|| self.builders.get(&extension_id).unwrap().build(Vec::new()))
            .clone()
    }
}

pub struct TestExtensionConfig {
    pub sdl: Option<&'static str>,
    pub kind: extension_catalog::Kind,
}

#[allow(unused_variables)] // makes it easier to copy-paste relevant functions
pub trait TestExtensionBuilder: Send + Sync + 'static {
    fn id(&self) -> Id
    where
        Self: Sized;

    fn config(&self) -> TestExtensionConfig
    where
        Self: Sized;

    fn build(&self, schema_directives: Vec<(&str, serde_json::Value)>) -> Arc<dyn TestExtension>;
}

#[allow(unused_variables)] // makes it easier to copy-paste relevant functions
#[async_trait::async_trait]
pub trait TestExtension: Send + Sync + 'static {
    async fn authenticate(&self, headers: &http::HeaderMap) -> Result<Token, ErrorResponse> {
        Err(GraphqlError::internal_extension_error().into())
    }

    async fn resolve_field(
        &self,
        headers: http::HeaderMap,
        directive: ExtensionFieldDirective<'_, serde_json::Value>,
        inputs: Vec<serde_json::Value>,
    ) -> Result<Vec<Result<Data, GraphqlError>>, GraphqlError> {
        Err(GraphqlError::internal_extension_error())
    }

    #[allow(clippy::manual_async_fn)]
    async fn authorize_query(
        &self,
        wasm_context: &DynHookContext,
        headers: &tokio::sync::RwLock<http::HeaderMap>,
        token: TokenRef<'_>,
        elements_grouped_by_directive_name: Vec<(&str, Vec<QueryElement<'_, serde_json::Value>>)>,
    ) -> Result<AuthorizationDecisions, ErrorResponse> {
        Err(GraphqlError::internal_extension_error().into())
    }

    #[allow(clippy::manual_async_fn)]
    async fn authorize_response(
        &self,
        wasm_context: &DynHookContext,
        directive_name: &str,
        directive_site: DirectiveSite<'_>,
        items: Vec<serde_json::Value>,
    ) -> Result<AuthorizationDecisions, GraphqlError> {
        Err(GraphqlError::internal_extension_error())
    }
}

impl runtime::extension::ExtensionRuntime for TestExtensions {
    type SharedContext = DynHookContext;

    fn resolve_field<'ctx, 'resp, 'f>(
        &'ctx self,
        headers: http::HeaderMap,
        ExtensionFieldDirective {
            extension_id,
            subgraph,
            field,
            name,
            arguments,
        }: ExtensionFieldDirective<'ctx, impl Anything<'ctx>>,
        inputs: impl IntoIterator<Item: Anything<'resp>> + Send,
    ) -> impl Future<Output = Result<Vec<Result<Data, GraphqlError>>, GraphqlError>> + Send + 'f
    where
        'ctx: 'f,
    {
        let inputs = inputs
            .into_iter()
            .map(serde_json::to_value)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        async move {
            let instance = self.get_subgraph_isntance(extension_id, subgraph).await;
            instance
                .resolve_field(
                    headers,
                    ExtensionFieldDirective {
                        extension_id,
                        subgraph,
                        field,
                        name,
                        arguments: serde_json::to_value(arguments).unwrap(),
                    },
                    inputs,
                )
                .await
        }
    }

    async fn authenticate(
        &self,
        extension_id: ExtensionId,
        _authorizer_id: runtime::extension::AuthorizerId,
        headers: http::HeaderMap,
    ) -> Result<(http::HeaderMap, Token), ErrorResponse> {
        let instance = self.get_global_instance(extension_id).await;
        let token = instance.authenticate(&headers).await?;
        Ok((headers, token))
    }

    async fn resolve_subscription<'ctx, 'f>(
        &'ctx self,
        _: http::HeaderMap,
        _: ExtensionFieldDirective<'ctx, impl Anything<'ctx>>,
    ) -> Result<BoxStream<'f, Result<Arc<Data>, GraphqlError>>, GraphqlError>
    where
        'ctx: 'f,
    {
        unimplemented!()
    }

    #[allow(clippy::manual_async_fn)]
    fn authorize_query<'ctx, 'fut, Extensions, Arguments>(
        &'ctx self,
        wasm_context: &'ctx Self::SharedContext,
        headers: http::HeaderMap,
        token: TokenRef<'ctx>,
        extensions: Extensions,
        // (directive name, range within query_elements)
        directives: impl ExactSizeIterator<Item = (&'ctx str, Range<usize>)>,
        query_elements: impl ExactSizeIterator<Item = QueryElement<'ctx, Arguments>>,
    ) -> impl Future<Output = Result<(http::HeaderMap, Vec<QueryAuthorizationDecisions>), ErrorResponse>> + Send + 'fut
    where
        'ctx: 'fut,
        // (extension id, range within directives, range within query_elements)
        Extensions: IntoIterator<
                Item = (ExtensionId, Range<usize>, Range<usize>),
                IntoIter: ExactSizeIterator<Item = (ExtensionId, Range<usize>, Range<usize>)>,
            > + Send
            + Clone
            + 'ctx,
        Arguments: Anything<'ctx>,
    {
        let directives = directives.collect::<Vec<_>>();
        let query_elements = query_elements
            .map(|element| QueryElement {
                site: element.site,
                arguments: serde_json::to_value(element.arguments).unwrap(),
            })
            .collect::<Vec<_>>();
        async move {
            let headers = tokio::sync::RwLock::new(headers);
            let headers_ref = &headers;
            let directives = &directives;
            let query_elements = &query_elements;
            let decisions = extensions
                .into_iter()
                .map(
                    move |(extension_id, directive_range, query_elements_range)| async move {
                        let instance = self.get_global_instance(extension_id).await;

                        instance
                            .authorize_query(
                                wasm_context,
                                headers_ref,
                                token,
                                directives[directive_range]
                                    .iter()
                                    .map(|(name, range)| (*name, query_elements[range.clone()].to_vec()))
                                    .collect(),
                            )
                            .and_then(|decisions| async {
                                Ok(QueryAuthorizationDecisions {
                                    extension_id,
                                    query_elements_range,
                                    decisions,
                                })
                            })
                            .await
                    },
                )
                .collect::<FuturesUnordered<_>>()
                .try_collect()
                .await?;
            let headers = headers.into_inner();
            Ok((headers, decisions))
        }
    }

    fn authorize_response<'ctx, 'fut>(
        &'ctx self,
        extension_id: ExtensionId,
        wasm_context: &'ctx Self::SharedContext,
        directive_name: &'ctx str,
        directive_site: DirectiveSite<'ctx>,
        items: impl IntoIterator<Item: Anything<'ctx>>,
    ) -> impl Future<Output = Result<AuthorizationDecisions, GraphqlError>> + Send + 'fut
    where
        'ctx: 'fut,
    {
        let items = items
            .into_iter()
            .map(serde_json::to_value)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        async move {
            let instance = self.get_global_instance(extension_id).await;
            instance
                .authorize_response(wasm_context, directive_name, directive_site, items)
                .await
        }
    }
}

pub fn json_data(value: impl Serialize) -> Data {
    Data::JsonBytes(serde_json::to_vec(&value).unwrap())
}
