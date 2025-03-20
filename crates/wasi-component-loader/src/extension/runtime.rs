mod subscription;

use crate::{Error, SharedContext, cbor, resources::Lease};

use super::{
    ExtensionPoolId, InputList, WasmExtensions,
    api::wit::{self, ResponseElement, ResponseElements},
};

use engine::{ErrorCode, ErrorResponse, GraphqlError};
use engine_schema::DirectiveSite;
use extension_catalog::ExtensionId;
use futures::{
    TryStreamExt,
    stream::{BoxStream, FuturesUnordered},
};
use runtime::{
    extension::{
        AuthorizationDecisions, AuthorizerId, Data, ExtensionFieldDirective, ExtensionRuntime,
        QueryAuthorizationDecisions, QueryElement, Token, TokenRef,
    },
    hooks::Anything,
};
use std::{future::Future, ops::Range, sync::Arc};
use subscription::{DeduplicatedSubscription, UniqueSubscription};

impl ExtensionRuntime for WasmExtensions {
    type SharedContext = crate::resources::SharedContext;

    #[allow(clippy::manual_async_fn)]
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
        let inputs = InputList::from_iter(inputs);

        async move {
            let mut instance = self.get(ExtensionPoolId::Resolver(extension_id)).await?;

            let directive = wit::FieldDefinitionDirective {
                name,
                site: wit::FieldDefinitionDirectiveSite {
                    parent_type_name: field.parent_entity().name(),
                    field_name: field.name(),
                },
                arguments: &cbor::to_vec(arguments).unwrap(),
            };

            instance
                .resolve_field(headers, subgraph.name(), directive, inputs)
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

    async fn authenticate(
        &self,
        extension_id: ExtensionId,
        authorizer_id: AuthorizerId,
        headers: http::HeaderMap,
    ) -> Result<(http::HeaderMap, Token), ErrorResponse> {
        let mut instance = self
            .get(ExtensionPoolId::Authorizer(extension_id, authorizer_id))
            .await?;

        let headers = Lease::Singleton(headers);
        instance
            .authenticate(headers)
            .await
            .map(|(headers, token)| (headers.into_inner().unwrap(), token))
            .map_err(|err| match err {
                crate::ErrorResponse::Internal(err) => {
                    tracing::error!("Wasm error: {err}");
                    ErrorResponse::from(GraphqlError::new("Internal error", ErrorCode::ExtensionError))
                }
                crate::ErrorResponse::Guest(err) => err.into_graphql_error_response(ErrorCode::Unauthenticated),
            })
    }

    async fn resolve_subscription<'ctx, 'f>(
        &'ctx self,
        headers: http::HeaderMap,
        directive: ExtensionFieldDirective<'ctx, impl Anything<'ctx>>,
    ) -> Result<BoxStream<'f, Result<Arc<Data>, GraphqlError>>, GraphqlError>
    where
        'ctx: 'f,
    {
        let ExtensionFieldDirective {
            extension_id,
            subgraph,
            field,
            name,
            arguments,
        } = directive;

        let mut instance = self.get(ExtensionPoolId::Resolver(extension_id)).await?;
        let arguments = &cbor::to_vec(arguments).unwrap();

        let site = wit::FieldDefinitionDirectiveSite {
            parent_type_name: field.parent_entity().name(),
            field_name: field.name(),
        };

        let directive = wit::FieldDefinitionDirective { name, site, arguments };

        let (headers, key) = instance
            .subscription_key(Lease::Singleton(headers), subgraph.name(), directive.clone())
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

    fn authorize_query<'ctx, 'fut, Extensions, Arguments>(
        &'ctx self,
        wasm_context: &'ctx SharedContext,
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
        let elements = {
            let mut out = Vec::new();
            out.reserve_exact(query_elements.len());
            for element in query_elements {
                // Some help for rust-analyzer who struggles for some reason.
                let element: QueryElement<'_, _> = element;
                let arguments = cbor::to_vec(element.arguments).unwrap();

                out.push(wit::QueryElement {
                    id: element.site.id().into(),
                    site: element.site.into(),
                    arguments,
                });
            }
            out
        };

        let mut directive_names = {
            let mut out = Vec::<(&'ctx str, u32, u32)>::new();
            out.reserve_exact(directives.len());

            for (directive_name, query_elements_range) in directives {
                out.push((
                    directive_name,
                    query_elements_range.start as u32,
                    query_elements_range.end as u32,
                ));
            }

            out
        };

        // The range we have in the current directive_names are relative to the whole elements
        // array. But we won't send the whole elements array to each extension. We'll only send the
        // relevant part. So we must adjust the range to take this in account.
        for (_, _, query_elements_range) in extensions.clone() {
            for (_, directive_query_elements_start, directive_query_elements_end) in &mut directive_names {
                *directive_query_elements_start -= query_elements_range.start as u32;
                *directive_query_elements_end -= query_elements_range.start as u32;
            }
        }

        let headers = Arc::new(tokio::sync::RwLock::new(headers));

        async move {
            let headers_ref = &headers;
            let directive_names = &directive_names;
            let elements = &elements;
            let decisions = extensions
                .into_iter()
                .map(
                    move |(extension_id, directive_range, query_elements_range)| async move {
                        let mut instance = self.get(ExtensionPoolId::Authorization(extension_id)).await?;
                        match instance
                            .authorize_query(
                                Lease::SharedMut(headers_ref.clone()),
                                token,
                                wit::QueryElements {
                                    directive_names: &directive_names[directive_range],
                                    elements: &elements[query_elements_range.clone()],
                                },
                            )
                            .await
                        {
                            Ok((_, decisions, state)) => {
                                if !state.is_empty() {
                                    wasm_context
                                        .authorization_state
                                        .write()
                                        .await
                                        .push((extension_id, state));
                                }
                                Ok(QueryAuthorizationDecisions {
                                    extension_id,
                                    query_elements_range,
                                    decisions,
                                })
                            }
                            Err(err) => Err(match err {
                                crate::ErrorResponse::Internal(err) => {
                                    tracing::error!("Wasm error: {err}");
                                    ErrorResponse::from(GraphqlError::new("Internal error", ErrorCode::ExtensionError))
                                }
                                crate::ErrorResponse::Guest(err) => {
                                    err.into_graphql_error_response(ErrorCode::Unauthenticated)
                                }
                            }),
                        }
                    },
                )
                .collect::<FuturesUnordered<_>>()
                .try_collect::<Vec<_>>()
                .await?;

            let headers = Arc::into_inner(headers).unwrap().into_inner();
            Ok((headers, decisions))
        }
    }

    fn authorize_response<'ctx, 'fut>(
        &'ctx self,
        extension_id: ExtensionId,
        wasm_context: &'ctx SharedContext,
        directive_name: &'ctx str,
        directive_site: DirectiveSite<'ctx>,
        items: impl IntoIterator<Item: Anything<'ctx>>,
    ) -> impl Future<Output = Result<AuthorizationDecisions, GraphqlError>> + Send + 'fut
    where
        'ctx: 'fut,
    {
        let items = items
            .into_iter()
            .map(|item| cbor::to_vec(item).unwrap())
            .collect::<Vec<_>>();
        async move {
            let guard = wasm_context.authorization_state.read().await;
            let state = guard
                .iter()
                .find_map(|(id, state)| {
                    if *id == extension_id {
                        Some(state.as_slice())
                    } else {
                        None
                    }
                })
                .unwrap_or(&[]);
            let mut instance = self.get(ExtensionPoolId::Authorization(extension_id)).await?;
            instance
                .authorize_response(
                    state,
                    ResponseElements {
                        directive_names: vec![(directive_name, 0, 1)],
                        elements: vec![ResponseElement {
                            query_element_id: directive_site.id().into(),
                            items_range: (0, items.len() as u32),
                        }],
                        items,
                    },
                )
                .await
                .map_err(|err| match err {
                    Error::Internal(err) => {
                        tracing::error!("Wasm error: {err}");
                        GraphqlError::new("Internal error", ErrorCode::ExtensionError)
                    }
                    Error::Guest(err) => err.into_graphql_error(ErrorCode::Unauthorized),
                })
        }
    }
}
