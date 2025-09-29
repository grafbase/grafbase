use std::sync::Arc;

use engine::{ErrorCode, GraphqlError};
use engine_schema::ExtensionDirective;
use event_queue::EventQueue;
use futures::stream::BoxStream;
use runtime::{
    extension::Anything,
    extension::{ArgumentsId, DynField, Field, ResolverExtension, Response},
};

use crate::gateway::{DispatchRule, EngineTestExtensions, TestExtensions};

#[allow(clippy::manual_async_fn, unused_variables)]
impl ResolverExtension<engine::EngineOperationContext> for EngineTestExtensions {
    async fn prepare<'ctx, F: Field<'ctx>>(
        &'ctx self,
        event_queue: Arc<EventQueue>,
        directive: ExtensionDirective<'ctx>,
        directive_arguments: impl Anything<'ctx>,
        field: F,
    ) -> Result<Vec<u8>, GraphqlError> {
        match self.dispatch[&directive.extension_id] {
            DispatchRule::Wasm => {
                self.wasm
                    .prepare(event_queue, directive, directive_arguments, field)
                    .await
            }
            DispatchRule::Test => self.test.prepare(directive, directive_arguments, field).await,
        }
    }

    type Arguments = Vec<(ArgumentsId, serde_json::Value)>;
    fn prepare_arguments<'resp>(
        &self,
        arguments: impl IntoIterator<Item = (ArgumentsId, impl Anything<'resp>)> + Send,
    ) -> Self::Arguments {
        arguments
            .into_iter()
            .map(|(id, value)| (id, serde_json::to_value(&value).unwrap()))
            .collect()
    }

    async fn resolve<'ctx>(
        &'ctx self,
        ctx: engine::EngineOperationContext,
        directive: ExtensionDirective<'ctx>,
        prepared_data: &'ctx [u8],
        subgraph_headers: http::HeaderMap,
        arguments: Self::Arguments,
    ) -> Response {
        match self.dispatch[&directive.extension_id] {
            DispatchRule::Wasm => {
                let arguments = self.wasm.prepare_arguments(arguments);
                self.wasm
                    .resolve(ctx, directive, prepared_data, subgraph_headers, arguments)
                    .await
            }
            DispatchRule::Test => {
                self.test
                    .resolve(ctx, directive, prepared_data, subgraph_headers, arguments)
                    .await
            }
        }
    }

    async fn resolve_subscription<'ctx, 's>(
        &'ctx self,
        ctx: engine::EngineOperationContext,
        directive: ExtensionDirective<'ctx>,
        prepared_data: &'ctx [u8],
        subgraph_headers: http::HeaderMap,
        arguments: Self::Arguments,
    ) -> BoxStream<'s, Response>
    where
        'ctx: 's,
    {
        match self.dispatch[&directive.extension_id] {
            DispatchRule::Wasm => {
                let arguments = self.wasm.prepare_arguments(arguments);
                self.wasm
                    .resolve_subscription(ctx, directive, prepared_data, subgraph_headers, arguments)
                    .await
            }
            DispatchRule::Test => {
                self.test
                    .resolve_subscription(ctx, directive, prepared_data, subgraph_headers, arguments)
                    .await
            }
        }
    }
}

#[allow(clippy::manual_async_fn, unused_variables)]
impl TestExtensions {
    async fn prepare<'ctx, F: Field<'ctx>>(
        &'ctx self,
        directive: ExtensionDirective<'ctx>,
        directive_arguments: impl Anything<'ctx>,
        field: F,
    ) -> Result<Vec<u8>, GraphqlError> {
        self.state
            .lock()
            .await
            .get_resolver_ext(
                directive.extension_id,
                directive.subgraph().expect("Must be present for resolvers"),
            )
            .prepare(
                directive,
                serde_json::to_value(directive_arguments).unwrap(),
                field.as_dyn(),
            )
            .await
    }

    async fn resolve<'ctx>(
        &'ctx self,
        ctx: engine::EngineOperationContext,
        directive: ExtensionDirective<'ctx>,
        prepared_data: &'ctx [u8],
        subgraph_headers: http::HeaderMap,
        arguments: Vec<(ArgumentsId, serde_json::Value)>,
    ) -> Response {
        self.state
            .lock()
            .await
            .get_resolver_ext(
                directive.extension_id,
                directive.subgraph().expect("Must be present for resolvers"),
            )
            .resolve(directive, prepared_data, subgraph_headers, arguments)
            .await
    }

    async fn resolve_subscription<'ctx, 's>(
        &'ctx self,
        ctx: engine::EngineOperationContext,
        directive: ExtensionDirective<'ctx>,
        prepared_data: &'ctx [u8],
        subgraph_headers: http::HeaderMap,
        arguments: Vec<(ArgumentsId, serde_json::Value)>,
    ) -> BoxStream<'s, Response>
    where
        'ctx: 's,
    {
        self.state
            .lock()
            .await
            .get_resolver_ext(
                directive.extension_id,
                directive.subgraph().expect("Must be present for resolvers"),
            )
            .resolve_subscription(directive, prepared_data, subgraph_headers, arguments)
            .await
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
                format!("Failed to serialize directive arguments: {e}"),
                ErrorCode::ExtensionError,
            )
        })
    }

    async fn resolve(
        &self,
        directive: ExtensionDirective<'_>,
        prepared_data: &[u8],
        headers: http::HeaderMap,
        arguments: Vec<(ArgumentsId, serde_json::Value)>,
    ) -> Response;

    async fn resolve_subscription<'ctx>(
        &self,
        directive: ExtensionDirective<'ctx>,
        prepared_data: &'ctx [u8],
        headers: http::HeaderMap,
        arguments: Vec<(ArgumentsId, serde_json::Value)>,
    ) -> BoxStream<'ctx, Response> {
        unimplemented!()
    }
}
