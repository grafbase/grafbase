use std::sync::Arc;

use engine::GraphqlError;
use engine_schema::Subgraph;
use extension_catalog::ExtensionId;
use futures::FutureExt as _;
use runtime::{
    extension::{ArgumentsId, Data, DynField, Field, ResolverExtension},
    hooks::Anything,
};

use crate::gateway::{DispatchRule, DynHookContext, ExtContext, ExtensionsDispatcher, TestExtensions};

#[allow(clippy::manual_async_fn, unused_variables)]
impl ResolverExtension<ExtContext> for ExtensionsDispatcher {
    async fn prepare<'ctx, F: Field<'ctx>>(
        &'ctx self,
        extension_id: ExtensionId,
        subgraph: Subgraph<'ctx>,
        field: F,
    ) -> Result<Vec<u8>, GraphqlError> {
        match self.dispatch[&extension_id] {
            DispatchRule::Wasm => self.wasm.prepare(extension_id, subgraph, field).await,
            DispatchRule::Test => self.test.prepare(extension_id, subgraph, field).await,
        }
    }

    fn resolve<'ctx, 'resp, 'f>(
        &'ctx self,
        extension_id: ExtensionId,
        subgraph: Subgraph<'ctx>,
        prepared_data: &'ctx [u8],
        subgraph_headers: http::HeaderMap,
        arguments: impl Iterator<Item = (ArgumentsId, impl Anything<'resp>)> + Send,
    ) -> impl Future<Output = Result<Data, GraphqlError>> + Send + 'f
    where
        'ctx: 'f,
    {
        match self.dispatch[&extension_id] {
            DispatchRule::Wasm => self
                .wasm
                .resolve(extension_id, subgraph, prepared_data, subgraph_headers, arguments)
                .boxed(),
            DispatchRule::Test => self
                .test
                .resolve(extension_id, subgraph, prepared_data, subgraph_headers, arguments)
                .boxed(),
        }
    }
}

#[allow(clippy::manual_async_fn, unused_variables)]
impl ResolverExtension<DynHookContext> for TestExtensions {
    async fn prepare<'ctx, F: Field<'ctx>>(
        &'ctx self,
        extension_id: ExtensionId,
        subgraph: Subgraph<'ctx>,
        field: F,
    ) -> Result<Vec<u8>, GraphqlError> {
        self.state
            .lock()
            .await
            .get_selection_set_resolver_ext(extension_id, subgraph)
            .prepare(extension_id, subgraph, field.as_dyn())
            .await
    }

    fn resolve<'ctx, 'resp, 'f>(
        &'ctx self,
        extension_id: ExtensionId,
        subgraph: Subgraph<'ctx>,
        prepared_data: &'ctx [u8],
        subgraph_headers: http::HeaderMap,
        arguments: impl Iterator<Item = (ArgumentsId, impl Anything<'resp>)> + Send,
    ) -> impl Future<Output = Result<Data, GraphqlError>> + Send + 'f
    where
        'ctx: 'f,
    {
        let arguments = arguments
            .into_iter()
            .map(|(id, args)| (id, serde_json::to_value(args).unwrap()))
            .collect::<Vec<_>>();
        async move {
            self.state
                .lock()
                .await
                .get_selection_set_resolver_ext(extension_id, subgraph)
                .resolve(extension_id, subgraph, prepared_data, subgraph_headers, arguments)
                .await
        }
    }
}

pub trait ResolverTestExtensionBuilder: Send + Sync + 'static {
    fn build(&self, schema_directives: Vec<(&str, serde_json::Value)>) -> Arc<dyn ResolverTestExtension>;
}

impl<F: Fn() -> Arc<dyn ResolverTestExtension> + Send + Sync + 'static> ResolverTestExtensionBuilder for F {
    fn build(&self, _schema_directives: Vec<(&str, serde_json::Value)>) -> Arc<dyn ResolverTestExtension> {
        self()
    }
}

#[allow(unused_variables)] // makes it easier to copy-paste relevant functions
#[async_trait::async_trait]
pub trait ResolverTestExtension: Send + Sync + 'static {
    async fn prepare<'ctx>(
        &self,
        extension_id: ExtensionId,
        subgraph: Subgraph<'ctx>,
        field: Box<dyn DynField<'ctx>>,
    ) -> Result<Vec<u8>, GraphqlError> {
        Ok(Vec::new())
    }

    async fn resolve(
        &self,
        extension_id: ExtensionId,
        subgraph: Subgraph<'_>,
        prepared_data: &[u8],
        subgraph_headers: http::HeaderMap,
        arguments: Vec<(ArgumentsId, serde_json::Value)>,
    ) -> Result<Data, GraphqlError>;
}
