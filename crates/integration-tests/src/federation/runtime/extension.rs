use std::{collections::HashMap, future::Future, sync::Arc};

use engine::{ErrorResponse, GraphqlError};
use engine_schema::{DirectiveSite, Subgraph, SubgraphId};
use extension_catalog::{Extension, ExtensionCatalog, ExtensionId, Id, Manifest};
use futures::stream::BoxStream;
use runtime::{
    extension::{AuthorizationDecisions, Data, ExtensionFieldDirective, Lease, QueryElement, Token, TokenRef},
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
        headers: &mut http::HeaderMap,
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
        headers: Lease<http::HeaderMap>,
    ) -> Result<(Lease<http::HeaderMap>, Token), ErrorResponse> {
        let instance = self.get_global_instance(extension_id).await;
        let token = headers
            .with_ref(async move |headers| instance.authenticate(headers).await)
            .await?;
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
    fn authorize_query<'ctx, 'fut, Groups, QueryElements, Arguments>(
        &'ctx self,
        extension_id: ExtensionId,
        wasm_context: &'ctx Self::SharedContext,
        mut headers: Lease<http::HeaderMap>,
        token: TokenRef<'ctx>,
        elements_grouped_by_directive_name: Groups,
    ) -> impl Future<Output = Result<(Lease<http::HeaderMap>, AuthorizationDecisions), ErrorResponse>> + Send + 'fut
    where
        'ctx: 'fut,
        Groups: ExactSizeIterator<Item = (&'ctx str, QueryElements)>,
        QueryElements: ExactSizeIterator<Item = QueryElement<'ctx, Arguments>>,
        Arguments: Anything<'ctx>,
    {
        let elements_grouped_by_directive_name = elements_grouped_by_directive_name
            .into_iter()
            .map(|(name, elements)| {
                (
                    name,
                    elements
                        .into_iter()
                        .map(|element| QueryElement {
                            site: element.site,
                            arguments: serde_json::to_value(element.arguments).unwrap(),
                        })
                        .collect(),
                )
            })
            .collect();
        async move {
            let instance = self.get_global_instance(extension_id).await;
            headers
                .with_ref_mut(async |headers| {
                    let headers = headers.unwrap();
                    instance
                        .authorize_query(wasm_context, headers, token, elements_grouped_by_directive_name)
                        .await
                })
                .await
                .map(|decisions| (headers, decisions))
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
