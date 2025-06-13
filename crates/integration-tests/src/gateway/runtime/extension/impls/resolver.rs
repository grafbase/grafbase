use std::sync::Arc;

use engine::{ErrorCode, GraphqlError};
use engine_schema::ExtensionDirective;
use futures::{FutureExt as _, stream::BoxStream};
use runtime::{
    extension::{ArgumentsId, DynField, Field, ResolverExtension, Response},
    hooks::Anything,
};

use crate::gateway::{DispatchRule, DynHookContext, ExtContext, ExtensionsDispatcher, TestExtensions};

#[allow(clippy::manual_async_fn, unused_variables)]
impl ResolverExtension<ExtContext> for ExtensionsDispatcher {
    async fn prepare<'ctx, F: Field<'ctx>>(
        &'ctx self,
        context: &ExtContext,
        directive: ExtensionDirective<'ctx>,
        directive_arguments: impl Anything<'ctx>,
        field: F,
    ) -> Result<Vec<u8>, GraphqlError> {
        match self.dispatch[&directive.extension_id] {
            DispatchRule::Wasm => {
                self.wasm
                    .prepare(&context.wasm, directive, directive_arguments, field)
                    .await
            }
            DispatchRule::Test => {
                self.test
                    .prepare(&context.test, directive, directive_arguments, field)
                    .await
            }
        }
    }

    fn resolve<'ctx, 'resp, 'f>(
        &'ctx self,
        context: &ExtContext,
        directive: ExtensionDirective<'ctx>,
        prepared_data: &'ctx [u8],
        subgraph_headers: http::HeaderMap,
        arguments: impl Iterator<Item = (ArgumentsId, impl Anything<'resp>)> + Send,
    ) -> impl Future<Output = Response> + Send + 'f
    where
        'ctx: 'f,
    {
        match self.dispatch[&directive.extension_id] {
            DispatchRule::Wasm => self
                .wasm
                .resolve(&context.wasm, directive, prepared_data, subgraph_headers, arguments)
                .boxed(),
            DispatchRule::Test => self
                .test
                .resolve(&context.test, directive, prepared_data, subgraph_headers, arguments)
                .boxed(),
        }
    }

    fn resolve_subscription<'ctx, 'resp, 'f>(
        &'ctx self,
        context: &ExtContext,
        directive: ExtensionDirective<'ctx>,
        prepared_data: &'ctx [u8],
        subgraph_headers: http::HeaderMap,
        arguments: impl Iterator<Item = (ArgumentsId, impl Anything<'resp>)> + Send,
    ) -> impl Future<Output = BoxStream<'f, Response>> + Send + 'f
    where
        'ctx: 'f,
    {
        match self.dispatch[&directive.extension_id] {
            DispatchRule::Wasm => self
                .wasm
                .resolve_subscription(&context.wasm, directive, prepared_data, subgraph_headers, arguments)
                .boxed(),
            DispatchRule::Test => self
                .test
                .resolve_subscription(&context.test, directive, prepared_data, subgraph_headers, arguments)
                .boxed(),
        }
    }
}

#[allow(clippy::manual_async_fn, unused_variables)]
impl ResolverExtension<DynHookContext> for TestExtensions {
    async fn prepare<'ctx, F: Field<'ctx>>(
        &'ctx self,
        _: &DynHookContext,
        directive: ExtensionDirective<'ctx>,
        directive_arguments: impl Anything<'ctx>,
        field: F,
    ) -> Result<Vec<u8>, GraphqlError> {
        self.state
            .lock()
            .await
            .get_resolver_ext(directive.extension_id, directive.subgraph())
            .prepare(
                directive,
                serde_json::to_value(directive_arguments).unwrap(),
                field.as_dyn(),
            )
            .await
    }

    fn resolve<'ctx, 'resp, 'f>(
        &'ctx self,
        _: &DynHookContext,
        directive: ExtensionDirective<'ctx>,
        prepared_data: &'ctx [u8],
        subgraph_headers: http::HeaderMap,
        arguments: impl Iterator<Item = (ArgumentsId, impl Anything<'resp>)> + Send,
    ) -> impl Future<Output = Response> + Send + 'f
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
                .get_resolver_ext(directive.extension_id, directive.subgraph())
                .resolve(directive, prepared_data, subgraph_headers, arguments)
                .await
        }
    }

    fn resolve_subscription<'ctx, 'resp, 'f>(
        &'ctx self,
        context: &DynHookContext,
        directive: ExtensionDirective<'ctx>,
        prepared_data: &'ctx [u8],
        subgraph_headers: http::HeaderMap,
        arguments: impl Iterator<Item = (ArgumentsId, impl Anything<'resp>)> + Send,
    ) -> impl Future<Output = BoxStream<'f, Response>> + Send + 'f
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
                .get_resolver_ext(directive.extension_id, directive.subgraph())
                .resolve_subscription(directive, prepared_data, subgraph_headers, arguments)
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
        directive: ExtensionDirective<'ctx>,
        directive_arguments: serde_json::Value,
        field: Box<dyn DynField<'ctx>>,
    ) -> Result<Vec<u8>, GraphqlError> {
        serde_json::to_vec(&directive_arguments).map_err(|e| {
            GraphqlError::new(
                format!("Failed to serialize directive arguments: {}", e),
                ErrorCode::ExtensionError,
            )
        })
    }

    async fn resolve(
        &self,
        directive: ExtensionDirective<'_>,
        prepared_data: &[u8],
        subgraph_headers: http::HeaderMap,
        arguments: Vec<(ArgumentsId, serde_json::Value)>,
    ) -> Response;

    async fn resolve_subscription<'ctx>(
        &self,
        directive: ExtensionDirective<'ctx>,
        prepared_data: &'ctx [u8],
        subgraph_headers: http::HeaderMap,
        arguments: Vec<(ArgumentsId, serde_json::Value)>,
    ) -> BoxStream<'ctx, Response> {
        unimplemented!()
    }
}
