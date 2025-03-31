use crate::{
    Error, SharedContext, cbor,
    extension::{InputList, api::wit},
    resources::Lease,
};

use super::{
    WasmExtensions,
    subscription::{DeduplicatedSubscription, UniqueSubscription},
};

use engine_error::{ErrorCode, GraphqlError};
use engine_schema::{ExtensionDirective, FieldDefinition};
use futures::stream::BoxStream;
use runtime::{
    extension::{Data, FieldResolverExtension},
    hooks::Anything,
};
use std::{future::Future, sync::Arc};

impl FieldResolverExtension<SharedContext> for WasmExtensions {
    async fn prepare<'ctx>(
        &'ctx self,
        _directive: ExtensionDirective<'ctx>,
        _field_definition: FieldDefinition<'ctx>,
        _directive_arguments: impl Anything<'ctx>,
    ) -> Result<Vec<u8>, GraphqlError> {
        Ok(Vec::new())
    }

    #[allow(clippy::manual_async_fn)]
    fn resolve_field<'ctx, 'resp, 'f>(
        &'ctx self,
        directive: ExtensionDirective<'ctx>,
        field_definition: FieldDefinition<'ctx>,
        _prepared_data: &'ctx [u8],
        subgraph_headers: http::HeaderMap,
        directive_arguments: impl Anything<'ctx>,
        inputs: impl Iterator<Item: Anything<'resp>> + Send,
    ) -> impl Future<Output = Result<Vec<Result<Data, GraphqlError>>, GraphqlError>> + Send + 'f
    where
        'ctx: 'f,
    {
        let inputs = InputList::from_iter(inputs);

        async move {
            let mut instance = self.get(directive.extension_id).await?;
            let subgraph = directive.subgraph();

            let directive = wit::FieldDefinitionDirective {
                name: directive.name(),
                site: wit::FieldDefinitionDirectiveSite {
                    parent_type_name: field_definition.parent_entity().name(),
                    field_name: field_definition.name(),
                },
                arguments: &cbor::to_vec(directive_arguments).unwrap(),
            };

            instance
                .resolve_field(subgraph_headers, subgraph.name(), directive, inputs)
                .await
                .map_err(|err| match err {
                    Error::Internal(err) => {
                        tracing::error!("Wasm error: {err}");
                        GraphqlError::new("Internal error", ErrorCode::ExtensionError)
                    }
                    Error::Guest(err) => err.into_graphql_error(ErrorCode::ExtensionError),
                })
        }
    }

    async fn resolve_subscription_field<'ctx, 'f>(
        &'ctx self,
        directive: ExtensionDirective<'ctx>,
        field_definition: FieldDefinition<'ctx>,
        _prepared_data: &'ctx [u8],
        subgraph_headers: http::HeaderMap,
        directive_arguments: impl Anything<'ctx>,
    ) -> Result<BoxStream<'f, Result<Arc<Data>, GraphqlError>>, GraphqlError>
    where
        'ctx: 'f,
    {
        let mut instance = self.get(directive.extension_id).await?;
        let subgraph = directive.subgraph();
        let arguments = &cbor::to_vec(directive_arguments).unwrap();

        let site = wit::FieldDefinitionDirectiveSite {
            parent_type_name: field_definition.parent_entity().name(),
            field_name: field_definition.name(),
        };

        let directive = wit::FieldDefinitionDirective {
            name: directive.name(),
            site,
            arguments,
        };

        let (headers, key) = instance
            .subscription_key(Lease::Singleton(subgraph_headers), subgraph.name(), directive.clone())
            .await
            .map_err(|err| match err {
                Error::Internal(err) => {
                    tracing::error!("Wasm error: {err}");
                    GraphqlError::new("Internal error", ErrorCode::ExtensionError)
                }
                Error::Guest(err) => err.into_graphql_error(ErrorCode::ExtensionError),
            })?;

        let headers = headers.into_inner().unwrap();

        match key {
            Some(key) => {
                let subscription = DeduplicatedSubscription {
                    extensions: self.clone(),
                    instance,
                    headers,
                    key,
                    subgraph,
                    directive,
                };

                subscription.resolve().await
            }
            None => {
                let subscription = UniqueSubscription {
                    instance,
                    headers,
                    subgraph,
                    directive,
                };

                subscription.resolve().await
            }
        }
    }
}
