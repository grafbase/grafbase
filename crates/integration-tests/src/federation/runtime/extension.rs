use std::{collections::HashMap, future::Future, sync::Arc};

use engine_schema::{Subgraph, SubgraphId};
use extension_catalog::{Extension, ExtensionCatalog, ExtensionId, Id, Manifest};
use futures::stream::BoxStream;
use runtime::{
    error::{ErrorResponse, PartialGraphqlError},
    extension::{AuthorizationDecisions, Data, ExtensionFieldDirective, QueryElement},
    hooks::{Anything, DynHookContext},
};
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
    pub fn push_extension<Builder: TestExtensionBuilder + Sized + Default>(&mut self, builder: Builder) {
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
        self.builders.insert(id, Box::new(Builder::default()));
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
    async fn resolve<'a>(
        &self,
        headers: http::HeaderMap,
        directive: ExtensionFieldDirective<'a, serde_json::Value>,
        inputs: Vec<serde_json::Value>,
    ) -> Result<Vec<Result<serde_json::Value, PartialGraphqlError>>, PartialGraphqlError> {
        Err(PartialGraphqlError::internal_extension_error())
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
    ) -> impl Future<Output = Result<Vec<Result<Data, PartialGraphqlError>>, PartialGraphqlError>> + Send + 'f
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
                .resolve(
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
                .map(|items| {
                    items
                        .into_iter()
                        .map(|res| res.map(|value| Data::JsonBytes(serde_json::to_vec(&value).unwrap())))
                        .collect()
                })
        }
    }

    async fn authenticate(
        &self,
        extension_id: ExtensionId,
        _authorizer_id: runtime::extension::AuthorizerId,
        _headers: http::HeaderMap,
    ) -> Result<(http::HeaderMap, Vec<u8>), ErrorResponse> {
        let _instance = self.get_global_instance(extension_id).await;
        unimplemented!()
    }

    async fn resolve_subscription<'ctx, 'f>(
        &'ctx self,
        _: http::HeaderMap,
        _: ExtensionFieldDirective<'ctx, impl Anything<'ctx>>,
    ) -> Result<BoxStream<'f, Result<Arc<Data>, PartialGraphqlError>>, PartialGraphqlError>
    where
        'ctx: 'f,
    {
        unimplemented!()
    }

    #[allow(clippy::manual_async_fn)]
    fn authorize_query<'ctx, 'fut, Groups, QueryElements, Arguments>(
        &'ctx self,
        _extension_id: ExtensionId,
        _elements_grouped_by_directive_name: Groups,
    ) -> impl Future<Output = Result<AuthorizationDecisions, ErrorResponse>> + Send + 'fut
    where
        'ctx: 'fut,
        Groups: ExactSizeIterator<Item = (&'ctx str, QueryElements)>,
        QueryElements: ExactSizeIterator<Item = QueryElement<'ctx, Arguments>>,
        Arguments: Anything<'ctx>,
    {
        async { unimplemented!() }
    }
}
