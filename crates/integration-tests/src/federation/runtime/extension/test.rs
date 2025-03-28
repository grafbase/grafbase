use std::{collections::HashMap, ops::Range, sync::Arc};

use crate::federation::DynHookContext;
use engine::{ErrorResponse, GraphqlError};
use engine_schema::{DirectiveSite, ExtensionDirective, FieldDefinition, Subgraph, SubgraphId};
use extension_catalog::{ExtensionId, Id};
use futures::{
    StreamExt, TryFutureExt as _, TryStreamExt as _,
    stream::{BoxStream, FuturesUnordered},
};
use runtime::{
    extension::{AuthorizationDecisions, Data, QueryAuthorizationDecisions, QueryElement, Token, TokenRef},
    hooks::Anything,
};
use serde::Serialize;
use tokio::sync::Mutex;

#[derive(Default)]
pub struct TestExtensions {
    pub(super) builders: HashMap<ExtensionId, Box<dyn TestExtensionBuilder>>,
    pub(super) global_instances: Mutex<HashMap<ExtensionId, Arc<dyn TestExtension>>>,
    pub(super) subgraph_instances: Mutex<HashMap<(ExtensionId, SubgraphId), Arc<dyn TestExtension>>>,
}

impl TestExtensions {
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

pub struct TestManifest {
    pub id: Id,
    pub sdl: Option<&'static str>,
    pub r#type: extension_catalog::Type,
}

#[allow(unused_variables)] // makes it easier to copy-paste relevant functions
pub trait TestExtensionBuilder: Send + Sync + 'static {
    fn manifest(&self) -> TestManifest;
    fn build(&self, schema_directives: Vec<(&str, serde_json::Value)>) -> Arc<dyn TestExtension>;
}

#[allow(unused_variables)] // makes it easier to copy-paste relevant functions
#[async_trait::async_trait]
pub trait TestExtension: Send + Sync + 'static {
    async fn authenticate(&self, headers: &http::HeaderMap) -> Result<Token, ErrorResponse> {
        Err(GraphqlError::internal_extension_error().into())
    }

    async fn prepare_field<'ctx>(
        &self,
        directive: ExtensionDirective<'ctx>,
        field_definition: FieldDefinition<'ctx>,
        directive_arguments: serde_json::Value,
    ) -> Result<Vec<u8>, GraphqlError> {
        Ok(Vec::new())
    }

    async fn resolve_field(
        &self,
        directive: ExtensionDirective<'_>,
        field_definition: FieldDefinition<'_>,
        prepared_data: &[u8],
        subgraph_headers: http::HeaderMap,
        directive_arguments: serde_json::Value,
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

    async fn prepare_field<'ctx>(
        &'ctx self,
        directive: ExtensionDirective<'ctx>,
        field_definition: FieldDefinition<'ctx>,
        directive_arguments: impl Anything<'ctx>,
    ) -> Result<Vec<u8>, GraphqlError> {
        let instance = self
            .get_subgraph_isntance(directive.extension_id, directive.subgraph())
            .await;
        instance
            .prepare_field(
                directive,
                field_definition,
                serde_json::to_value(directive_arguments).unwrap(),
            )
            .await
    }

    fn resolve_field<'ctx, 'resp, 'f>(
        &'ctx self,
        directive: ExtensionDirective<'ctx>,
        field_definition: FieldDefinition<'ctx>,
        prepared_data: &'ctx [u8],
        subgraph_headers: http::HeaderMap,
        directive_arguments: impl Anything<'ctx>,
        inputs: impl Iterator<Item: Anything<'resp>> + Send,
    ) -> impl Future<Output = Result<Vec<Result<Data, GraphqlError>>, GraphqlError>> + Send + 'f
    where
        'ctx: 'f,
    {
        let inputs = inputs
            .into_iter()
            .map(serde_json::to_value)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let directive_arguments = serde_json::to_value(directive_arguments).unwrap();
        async move {
            let instance = self
                .get_subgraph_isntance(directive.extension_id, directive.subgraph())
                .await;
            instance
                .resolve_field(
                    directive,
                    field_definition,
                    prepared_data,
                    subgraph_headers,
                    directive_arguments,
                    inputs,
                )
                .await
        }
    }

    async fn authenticate(
        &self,
        extension_ids: &[ExtensionId],
        headers: http::HeaderMap,
    ) -> (http::HeaderMap, Result<Token, ErrorResponse>) {
        let mut futures = extension_ids
            .iter()
            .map(|id| async {
                let instance = self.get_global_instance(*id).await;
                instance.authenticate(&headers).await
            })
            .collect::<FuturesUnordered<_>>();

        let mut last_error = None;
        while let Some(result) = futures.by_ref().next().await {
            match result {
                Ok(token) => {
                    drop(futures);
                    return (headers, Ok(token));
                }
                Err(err) => {
                    last_error = Some(err);
                }
            }
        }

        drop(futures);
        (headers, Err(last_error.unwrap()))
    }

    async fn resolve_subscription<'ctx, 'f>(
        &'ctx self,
        _directive: ExtensionDirective<'ctx>,
        _field_definition: FieldDefinition<'ctx>,
        _prepared_data: &'ctx [u8],
        _subgraph_headers: http::HeaderMap,
        _directive_arguments: impl Anything<'ctx>,
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
