mod subscription;

use engine_error::{ErrorCode, GraphqlError};
use engine_schema::ExtensionDirective;
use futures::{StreamExt as _, stream::BoxStream};
use runtime::{
    extension::{ArgumentsId, Field as _, ResolverExtension, Response, SelectionSet as _},
    hooks::Anything,
};

use crate::{
    Error, cbor,
    extension::{
        WasmExtensions,
        api::wit::{self, Field, SelectionSet},
    },
};

#[allow(clippy::manual_async_fn)]
impl ResolverExtension for WasmExtensions {
    async fn prepare<'ctx, F: runtime::extension::Field<'ctx>>(
        &'ctx self,
        directive: ExtensionDirective<'ctx>,
        directive_arguments: impl Anything<'ctx>,
        field: F,
    ) -> Result<Vec<u8>, GraphqlError> {
        let mut instance = self.get(directive.extension_id).await?;
        let mut fields = Vec::new();

        fields.push(Field {
            alias: field.alias(),
            definition_id: field.definition().id.as_guid(),
            arguments: field.arguments().map(Into::into),
            selection_set: None,
        });

        if let Some(selection_set) = field.selection_set() {
            let mut stack: Vec<(usize, F::SelectionSet)> = vec![(0, selection_set)];

            while let Some((field_id, selection_set)) = stack.pop() {
                let start = fields.len();
                for field in selection_set.fields_ordered_by_parent_entity() {
                    fields.push(Field {
                        alias: field.alias(),
                        definition_id: field.definition().id.as_guid(),
                        arguments: field.arguments().map(Into::into),
                        selection_set: None,
                    });
                    if let Some(selection_set) = field.selection_set() {
                        stack.push((fields.len() - 1, selection_set));
                    }
                }
                fields[field_id].selection_set = Some(SelectionSet {
                    requires_typename: selection_set.requires_typename(),
                    fields_ordered_by_parent_entity: (start as u16, fields.len() as u16),
                });
            }
        }

        let dir = wit::Directive {
            name: directive.name(),
            arguments: cbor::to_vec(directive_arguments).unwrap(),
        };
        instance
            .prepare(directive.subgraph().name(), dir, 0, &fields)
            .await
            .map_err(|err| match err {
                Error::Internal(err) => {
                    tracing::error!("Wasm error: {err}");
                    GraphqlError::new("Internal error", ErrorCode::ExtensionError)
                }
                Error::Guest(err) => err.into_graphql_error(ErrorCode::ExtensionError),
            })?
    }

    fn resolve<'ctx, 'resp, 'f>(
        &'ctx self,
        directive: ExtensionDirective<'ctx>,
        prepared_data: &'ctx [u8],
        subgraph_headers: http::HeaderMap,
        arguments: impl Iterator<Item = (ArgumentsId, impl Anything<'resp>)> + Send,
    ) -> impl Future<Output = Response> + Send + 'f
    where
        'ctx: 'f,
    {
        let arguments = arguments
            .map(|(id, value)| (id.into(), cbor::to_vec(&value).unwrap()))
            .collect::<Vec<(wit::ArgumentsId, Vec<u8>)>>();
        async move {
            let mut instance = match self.get(directive.extension_id).await {
                Ok(instance) => instance,
                Err(err) => {
                    tracing::error!("Error getting extension instance: {err}");
                    return Response {
                        data: None,
                        errors: vec![GraphqlError::internal_extension_error()],
                    };
                }
            };
            let arguments_refs = arguments
                .iter()
                .map(|(id, value)| (*id, value.as_slice()))
                .collect::<Vec<_>>();
            let result = instance
                .resolve(subgraph_headers, prepared_data, &arguments_refs)
                .await
                .map_err(|err| match err {
                    Error::Internal(err) => {
                        tracing::error!("Wasm error: {err}");
                        GraphqlError::new("Internal error", ErrorCode::ExtensionError)
                    }
                    Error::Guest(err) => err.into_graphql_error(ErrorCode::ExtensionError),
                });
            match result {
                Ok(response) => response.into(),
                Err(err) => Response {
                    data: None,
                    errors: vec![err],
                },
            }
        }
    }

    fn resolve_subscription<'ctx, 'resp, 'f>(
        &'ctx self,
        directive: ExtensionDirective<'ctx>,
        prepared_data: &'ctx [u8],
        subgraph_headers: http::HeaderMap,
        arguments: impl Iterator<Item = (ArgumentsId, impl Anything<'resp>)> + Send,
    ) -> impl Future<Output = BoxStream<'f, Response>> + Send + 'f
    where
        'ctx: 'f,
    {
        let arguments = arguments
            .map(|(id, value)| (id.into(), cbor::to_vec(&value).unwrap()))
            .collect::<Vec<(wit::ArgumentsId, Vec<u8>)>>();
        async move {
            let mut instance = match self.get(directive.extension_id).await {
                Ok(instance) => instance,
                Err(err) => {
                    tracing::error!("Error getting extension instance: {err}");
                    let response = Response {
                        data: None,
                        errors: vec![GraphqlError::internal_extension_error()],
                    };
                    return futures::stream::once(std::future::ready(response)).boxed();
                }
            };
            let arguments_refs = arguments
                .iter()
                .map(|(id, value)| (*id, value.as_slice()))
                .collect::<Vec<_>>();
            let result = instance
                .create_subscription(subgraph_headers, prepared_data, &arguments_refs)
                .await
                .map_err(|err| match err {
                    Error::Internal(err) => {
                        tracing::error!("Wasm error: {err}");
                        GraphqlError::new("Internal error", ErrorCode::ExtensionError)
                    }
                    Error::Guest(err) => err.into_graphql_error(ErrorCode::ExtensionError),
                });
            match result {
                Ok(Ok(key)) => match key {
                    Some(key) => {
                        subscription::DeduplicatedSubscription {
                            extensions: self.clone(),
                            key,
                            instance,
                        }
                        .resolve()
                        .await
                    }
                    None => subscription::UniqueSubscription { instance }.resolve().await,
                },
                Ok(Err(err)) => {
                    let response = Response {
                        data: None,
                        errors: vec![err],
                    };
                    futures::stream::once(std::future::ready(response)).boxed()
                }
                Err(err) => {
                    tracing::error!("Error creating subscription: {err}");
                    let response = Response {
                        data: None,
                        errors: vec![GraphqlError::internal_extension_error()],
                    };
                    futures::stream::once(std::future::ready(response)).boxed()
                }
            }
        }
    }
}
