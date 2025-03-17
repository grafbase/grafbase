mod subscription;

use crate::{
    SharedContext, cbor,
    extension::ExtensionLoader,
    resources::{Lease, SharedResources},
};

use super::{
    ExtensionGuestConfig, InputList,
    api::wit::{self, ResponseElement, ResponseElements},
    pool::{ExtensionGuard, Pool},
};

use dashmap::DashMap;
use engine::{ErrorCode, ErrorResponse, GraphqlError};
use engine_schema::DirectiveSite;
use extension_catalog::ExtensionId;
use futures::{
    TryStreamExt,
    stream::{BoxStream, FuturesUnordered},
};
use futures_util::{StreamExt, stream};
use gateway_config::WasiExtensionsConfig;
use runtime::{
    extension::{
        AuthorizationDecisions, AuthorizerId, Data, ExtensionFieldDirective, ExtensionRuntime,
        QueryAuthorizationDecisions, QueryElement, Token, TokenRef,
    },
    hooks::Anything,
};
use semver::Version;
use std::{collections::HashMap, future::Future, ops::Range, sync::Arc};
use subscription::{DeduplicatedSubscription, UniqueSubscription};
use tokio::{sync::broadcast, task::JoinHandle};

#[derive(Clone, Default)]
pub struct ExtensionsWasiRuntime(Option<Arc<WasiExtensionsInner>>);

type Subscriptions = Arc<DashMap<Vec<u8>, broadcast::Sender<Result<Arc<Data>, GraphqlError>>>>;

struct WasiExtensionsInner {
    instance_pools: HashMap<ExtensionPoolId, Pool>,
    subscriptions: Subscriptions,
}

impl ExtensionsWasiRuntime {
    pub async fn new<T: serde::Serialize + Send + 'static>(
        shared_resources: SharedResources,
        extensions: Vec<ExtensionConfig<T>>,
    ) -> crate::Result<Self> {
        if extensions.is_empty() {
            return Ok(Self(None));
        }

        let instance_pools = create_pools(shared_resources, extensions).await?;
        let subscriptions = Arc::new(DashMap::new());

        let inner = WasiExtensionsInner {
            instance_pools,
            subscriptions,
        };

        Ok(Self(Some(Arc::new(inner))))
    }

    async fn get(&self, id: ExtensionPoolId) -> Result<ExtensionGuard, GraphqlError> {
        let pool = self
            .0
            .as_ref()
            .and_then(|inner| inner.instance_pools.get(&id))
            .ok_or_else(GraphqlError::internal_extension_error)?;
        Ok(pool.get().await)
    }

    fn subscriptions(&self) -> Result<Subscriptions, GraphqlError> {
        let subscriptions = self
            .0
            .as_ref()
            .ok_or_else(GraphqlError::internal_extension_error)?
            .subscriptions
            .clone();

        Ok(subscriptions)
    }
}

async fn create_pools<T: serde::Serialize + Send + 'static>(
    shared_resources: SharedResources,
    extensions: Vec<ExtensionConfig<T>>,
) -> crate::Result<HashMap<ExtensionPoolId, Pool>> {
    type Handle = JoinHandle<crate::Result<(ExtensionPoolId, Pool)>>;

    let mut creating_pools: Vec<Handle> = Vec::new();

    for config in extensions {
        let shared = shared_resources.clone();

        creating_pools.push(tokio::task::spawn_blocking(move || {
            tracing::info!("Loading extension {}", config.manifest_id);

            let loader = ExtensionLoader::new(shared, config.wasi_config, config.guest_config, config.sdk_version)?;

            Ok((config.id, Pool::new(loader, config.max_pool_size)))
        }));
    }

    let mut pools = HashMap::new();

    let mut creating_pools = stream::iter(creating_pools)
        .buffer_unordered(std::thread::available_parallelism().map(|i| i.get()).unwrap_or(1));

    while let Some(result) = creating_pools.next().await {
        match result.unwrap() {
            Ok((id, pool)) => {
                pools.insert(id, pool);
            }
            Err(error) => return Err(error),
        }
    }

    Ok(pools)
}

impl ExtensionRuntime for ExtensionsWasiRuntime {
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
                .map_err(|err| err.into_graphql_error(ErrorCode::ExtensionError))
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
            .map_err(|err| err.into_graphql_error_response(ErrorCode::Unauthenticated))
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
            .map_err(|err| err.into_graphql_error(ErrorCode::ExtensionError))?;
        let headers = headers.into_inner().unwrap();

        match key {
            Some(key) => {
                let subscription = DeduplicatedSubscription {
                    subscriptions: self.subscriptions()?,
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
                            Err(err) => Err(err.into_graphql_error_response(ErrorCode::Unauthorized)),
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
                .map_err(|err| err.into_graphql_error(ErrorCode::Unauthorized))
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExtensionPoolId {
    Resolver(ExtensionId),
    Authorizer(ExtensionId, AuthorizerId),
    Authorization(ExtensionId),
}

impl From<ExtensionId> for ExtensionPoolId {
    fn from(id: ExtensionId) -> Self {
        Self::Resolver(id)
    }
}

impl From<(ExtensionId, AuthorizerId)> for ExtensionPoolId {
    fn from((id, authorizer_id): (ExtensionId, AuthorizerId)) -> Self {
        Self::Authorizer(id, authorizer_id)
    }
}

pub struct ExtensionConfig<T> {
    pub id: ExtensionPoolId,
    pub manifest_id: extension_catalog::Id,
    pub max_pool_size: Option<usize>,
    pub wasi_config: WasiExtensionsConfig,
    pub guest_config: ExtensionGuestConfig<T>,
    pub sdk_version: Version,
}
