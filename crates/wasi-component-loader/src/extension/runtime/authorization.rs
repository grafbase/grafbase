use crate::{
    cbor,
    extension::{AuthorizeQueryOutput, api::wit},
    resources::OwnedOrShared,
    wasmsafe,
};

use super::EngineWasmExtensions;

use engine::{EngineOperationContext, EngineRequestContext};
use engine_error::{ErrorResponse, GraphqlError};
use engine_schema::DirectiveSite;
use extension_catalog::ExtensionId;
use futures::{TryStreamExt, stream::FuturesUnordered};
use itertools::Itertools as _;
use runtime::extension::{
    Anything, AuthorizationDecisions, AuthorizationExtension, AuthorizeQuery, QueryAuthorizationDecisions,
    QueryElement, TokenRef,
};
use std::{future::Future, ops::Range, sync::Arc};

impl AuthorizationExtension<EngineRequestContext, EngineOperationContext> for EngineWasmExtensions {
    fn authorize_query<'ctx, 'fut, Extensions, Arguments>(
        &'ctx self,
        ctx: EngineRequestContext,
        headers: http::HeaderMap,
        token: TokenRef<'ctx>,
        extensions: Extensions,
        // (directive name, range within query_elements)
        directives: impl ExactSizeIterator<Item = (&'ctx str, Range<usize>)>,
        query_elements: impl ExactSizeIterator<Item = QueryElement<'ctx, Arguments>>,
    ) -> impl Future<Output = Result<AuthorizeQuery, ErrorResponse>> + Send + 'fut
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
                    subgraph_name: element.subgraph.map(|s| s.name()),
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
        for (_, directives_range, query_elements_range) in extensions.clone() {
            for (_, directive_query_elements_start, directive_query_elements_end) in
                &mut directive_names[directives_range]
            {
                *directive_query_elements_start -= query_elements_range.start as u32;
                *directive_query_elements_end -= query_elements_range.start as u32;
            }
        }

        let headers = if self.use_mutable_headers_in_authorize_query {
            OwnedOrShared::LegacySharedMut(Arc::new(tokio::sync::RwLock::new(headers)))
        } else {
            OwnedOrShared::from(Arc::new(headers))
        };

        async move {
            let ctx_ref = &ctx;
            let headers_ref = &headers;
            let directive_names = &directive_names;
            let elements = &elements;
            let (decisions, state, context, additional_headers): (Vec<_>, Vec<_>, Vec<_>, Vec<_>) = extensions
                .into_iter()
                .map(
                    move |(extension_id, directive_range, query_elements_range)| async move {
                        let mut instance = self.get(extension_id).await?;
                        wasmsafe!(
                            instance
                                .authorize_query(
                                    ctx_ref.clone(),
                                    headers_ref.clone_shared().unwrap(),
                                    token,
                                    wit::QueryElements {
                                        directive_names: &directive_names[directive_range],
                                        elements: &elements[query_elements_range.clone()],
                                    },
                                )
                                .await
                        )
                        .map(
                            |AuthorizeQueryOutput {
                                 subgraph_headers: _,
                                 additional_headers,
                                 decisions,
                                 context,
                                 state,
                             }| {
                                (
                                    QueryAuthorizationDecisions {
                                        extension_id,
                                        query_elements_range,
                                        decisions,
                                    },
                                    (extension_id, state),
                                    (extension_id, context),
                                    additional_headers,
                                )
                            },
                        )
                    },
                )
                .collect::<FuturesUnordered<_>>()
                .try_collect::<Vec<_>>()
                .await?
                .into_iter()
                .multiunzip();

            let mut headers = headers.into_inner().unwrap();
            for additional_headers in additional_headers.into_iter().flatten() {
                headers.extend(additional_headers);
            }
            Ok(AuthorizeQuery {
                headers,
                decisions,
                context,
                state,
            })
        }
    }

    fn authorize_response<'ctx, 'fut>(
        &'ctx self,
        ctx: EngineOperationContext,
        extension_id: ExtensionId,
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
            let state = ctx
                .authorization_state()
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

            wasmsafe!(
                instance
                    .authorize_response(
                        ctx.clone(),
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
            )
        }
    }
}
