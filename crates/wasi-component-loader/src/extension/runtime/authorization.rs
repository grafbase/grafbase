use crate::{Error, SharedContext, cbor, extension::api::wit, resources::Lease};

use super::WasmExtensions;

use engine_error::{ErrorCode, ErrorResponse, GraphqlError};
use engine_schema::DirectiveSite;
use extension_catalog::ExtensionId;
use futures::{TryStreamExt, stream::FuturesUnordered};
use runtime::{
    extension::{AuthorizationDecisions, AuthorizationExtension, QueryAuthorizationDecisions, QueryElement, TokenRef},
    hooks::Anything,
};
use std::{future::Future, ops::Range, sync::Arc};

impl AuthorizationExtension<SharedContext> for WasmExtensions {
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
                    id: element.site.id().as_guid(),
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
                        let mut instance = self.get(extension_id).await?;
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
                                    err.into_graphql_error_response(ErrorCode::Unauthorized)
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
            let mut instance = self.get(extension_id).await?;
            instance
                .authorize_response(
                    state,
                    wit::ResponseElements {
                        directive_names: vec![(directive_name, 0, 1)],
                        elements: vec![wit::ResponseElement {
                            query_element_id: directive_site.id().as_guid(),
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
