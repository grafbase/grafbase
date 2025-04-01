use std::sync::Arc;

use engine::GraphqlError;
use engine_schema::{FieldDefinition, Subgraph};
use extension_catalog::ExtensionId;
use futures::FutureExt as _;
use runtime::{
    extension::{ArgumentsId, Data, DynSelectionSet, SelectionSet, SelectionSetResolverExtension},
    hooks::Anything,
};

use crate::federation::{DispatchRule, DynHookContext, ExtContext, ExtensionsDispatcher, TestExtensions};

#[allow(clippy::manual_async_fn, unused_variables)]
impl SelectionSetResolverExtension<ExtContext> for ExtensionsDispatcher {
    async fn prepare<'ctx>(
        &'ctx self,
        extension_id: ExtensionId,
        subgraph: Subgraph<'ctx>,
        field_definition: FieldDefinition<'ctx>,
        selection_set: impl SelectionSet<'ctx>,
    ) -> Result<Vec<u8>, GraphqlError> {
        match self.dispatch[&extension_id] {
            DispatchRule::Wasm => {
                self.wasm
                    .prepare(extension_id, subgraph, field_definition, selection_set)
                    .await
            }
            DispatchRule::Test => {
                self.test
                    .prepare(extension_id, subgraph, field_definition, selection_set)
                    .await
            }
        }
    }

    fn resolve_query_or_mutation_field<'ctx, 'resp, 'f>(
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
                .resolve_query_or_mutation_field(extension_id, subgraph, prepared_data, subgraph_headers, arguments)
                .boxed(),
            DispatchRule::Test => self
                .test
                .resolve_query_or_mutation_field(extension_id, subgraph, prepared_data, subgraph_headers, arguments)
                .boxed(),
        }
    }
}

#[allow(clippy::manual_async_fn, unused_variables)]
impl SelectionSetResolverExtension<DynHookContext> for TestExtensions {
    async fn prepare<'ctx>(
        &'ctx self,
        extension_id: ExtensionId,
        subgraph: Subgraph<'ctx>,
        field_definition: FieldDefinition<'ctx>,
        selection_set: impl SelectionSet<'ctx>,
    ) -> Result<Vec<u8>, GraphqlError> {
        self.state
            .lock()
            .await
            .get_selection_set_resolver_ext(extension_id, subgraph)
            .prepare(extension_id, subgraph, field_definition, selection_set.as_dyn())
            .await
    }

    fn resolve_query_or_mutation_field<'ctx, 'resp, 'f>(
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
                .resolve_field(extension_id, subgraph, prepared_data, subgraph_headers, arguments)
                .await
        }
    }
}

pub trait SelectionSetResolverTestExtensionBuilder: Send + Sync + 'static {
    fn build(&self, schema_directives: Vec<(&str, serde_json::Value)>) -> Arc<dyn SelectionSetResolverTestExtension>;
}

impl<F: Fn() -> Arc<dyn SelectionSetResolverTestExtension> + Send + Sync + 'static>
    SelectionSetResolverTestExtensionBuilder for F
{
    fn build(&self, _schema_directives: Vec<(&str, serde_json::Value)>) -> Arc<dyn SelectionSetResolverTestExtension> {
        self()
    }
}

#[allow(unused_variables)] // makes it easier to copy-paste relevant functions
#[async_trait::async_trait]
pub trait SelectionSetResolverTestExtension: Send + Sync + 'static {
    async fn prepare<'ctx>(
        &self,
        extension_id: ExtensionId,
        subgraph: Subgraph<'ctx>,
        field_definition: FieldDefinition<'ctx>,
        selection_set: Box<dyn DynSelectionSet<'ctx>>,
    ) -> Result<Vec<u8>, GraphqlError> {
        Ok(Vec::new())
    }

    async fn resolve_field(
        &self,
        extension_id: ExtensionId,
        subgraph: Subgraph<'_>,
        prepared_data: &[u8],
        subgraph_headers: http::HeaderMap,
        arguments: Vec<(ArgumentsId, serde_json::Value)>,
    ) -> Result<Data, GraphqlError>;
}
