mod authorization;

use crate::{cbor, extension::ExtensionLoader, resources::SharedResources};

use super::{
    ExtensionGuestConfig, InputList,
    pool::{ExtensionGuard, Pool},
    wit,
};
use dashmap::DashMap;
use engine_schema::Subgraph;
use extension_catalog::ExtensionId;
use futures::stream::BoxStream;
use futures_util::{StreamExt, stream};
use gateway_config::WasiExtensionsConfig;
use runtime::{
    error::{ErrorResponse, PartialErrorCode, PartialGraphqlError},
    extension::{AuthorizationDecisions, AuthorizerId, Data, ExtensionFieldDirective, ExtensionRuntime, QueryElement},
    hooks::Anything,
};
use std::{
    collections::{HashMap, VecDeque},
    future::Future,
    sync::Arc,
};
use tokio::{sync::broadcast, task::JoinHandle};
use tokio_stream::wrappers::BroadcastStream;

#[derive(Clone, Default)]
pub struct ExtensionsWasiRuntime(Option<Arc<WasiExtensionsInner>>);

type Subscriptions = Arc<DashMap<Vec<u8>, Option<broadcast::Sender<Result<Data, PartialGraphqlError>>>>>;

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

    async fn get(&self, id: ExtensionPoolId) -> Result<ExtensionGuard, PartialGraphqlError> {
        let pool = self
            .0
            .as_ref()
            .and_then(|inner| inner.instance_pools.get(&id))
            .ok_or_else(PartialGraphqlError::internal_extension_error)?;
        Ok(pool.get().await)
    }

    /// Resolves a unique subscription that creates a new subscription stream for each caller.
    /// This is used when the extension does not provide a way to deduplicate subscription calls.
    async fn resolve_unique_subscription<'f, 'ctx>(
        &self,
        mut instance: ExtensionGuard,
        headers: http::HeaderMap,
        subgraph: Subgraph<'ctx>,
        directive: wit::FieldDefinitionDirective<'ctx>,
    ) -> Result<BoxStream<'f, Result<Data, PartialGraphqlError>>, PartialGraphqlError>
    where
        'ctx: 'f,
    {
        instance
            .resolve_subscription(headers, subgraph.name(), directive)
            .await
            .map_err(|err| err.into_graphql_error(PartialErrorCode::ExtensionError))?;

        let stream = stream::unfold((instance, VecDeque::new()), async move |(mut instance, mut tail)| {
            if let Some(data) = tail.pop_front() {
                return Some((data, (instance, tail)));
            }

            let item = loop {
                match instance.resolve_next_subscription_item().await {
                    Ok(Some(item)) if item.outputs.is_empty() => {
                        continue;
                    }
                    Ok(Some(item)) => {
                        tracing::debug!("subscription item resolved");
                        break item;
                    }
                    Ok(None) => {
                        tracing::debug!("subscription completed");
                        return None;
                    }
                    Err(e) => {
                        tracing::error!("Error resolving subscription item: {e}");
                        return Some((Err(PartialGraphqlError::internal_extension_error()), (instance, tail)));
                    }
                }
            };

            for item in item.outputs {
                match item {
                    Ok(item) => {
                        tail.push_back(Ok(Data::CborBytes(item)));
                    }
                    Err(error) => {
                        let error = error.into_graphql_error(PartialErrorCode::InternalServerError);
                        tail.push_back(Err(error));
                    }
                }
            }

            tail.pop_front().map(|item| (item, (instance, tail)))
        });

        Ok(Box::pin(stream))
    }

    /// Resolves a deduplicated subscription that reuses the same subscription stream for multiple callers
    /// when they use the same identifier. This is more efficient for subscriptions that can be shared.
    async fn resolve_deduplicated_subscription<'f, 'ctx>(
        &self,
        mut instance: ExtensionGuard,
        headers: http::HeaderMap,
        identifier: Vec<u8>,
        subgraph: Subgraph<'ctx>,
        directive: wit::FieldDefinitionDirective<'ctx>,
    ) -> Result<BoxStream<'f, Result<Data, PartialGraphqlError>>, PartialGraphqlError>
    where
        'ctx: 'f,
    {
        let channels = self
            .0
            .as_ref()
            .ok_or_else(PartialGraphqlError::internal_extension_error)?
            .subscriptions
            .clone();

        let receiver = channels
            .get(&identifier)
            .as_ref()
            .and_then(|channel| channel.as_ref())
            .map(|channel| channel.subscribe());

        let (sender, receiver) = match receiver {
            Some(receiver) => {
                tracing::debug!("reuse existing channel");

                let stream = BroadcastStream::new(receiver).map(|result| match result {
                    Ok(data) => data,
                    Err(_) => Err(PartialGraphqlError::stream_lag()),
                });

                return Ok(Box::pin(stream));
            }
            None => {
                tracing::debug!("create new channel");
                broadcast::channel(1000)
            }
        };

        channels.insert(identifier.clone(), Some(sender.clone()));

        instance
            .resolve_subscription(headers, subgraph.name(), directive)
            .await
            .map_err(|err| err.into_graphql_error(PartialErrorCode::ExtensionError))?;

        tokio::spawn(async move {
            let mut deadge = false;

            loop {
                let item = loop {
                    if deadge && sender.receiver_count() == 0 {
                        return;
                    }

                    match instance.resolve_next_subscription_item().await {
                        Ok(Some(item)) if item.outputs.is_empty() => {
                            continue;
                        }
                        Ok(Some(item)) => {
                            tracing::debug!("subscription item resolved");

                            break item;
                        }
                        Ok(None) => {
                            tracing::debug!("subscription ended");
                            channels.remove(&identifier);

                            return;
                        }
                        Err(err) => {
                            tracing::error!("subscription item error: {}", err);

                            return;
                        }
                    }
                };

                for item in item.outputs {
                    let data = match item {
                        Ok(item) => Ok(Data::CborBytes(item)),
                        Err(err) => Err(err.into_graphql_error(PartialErrorCode::InternalServerError)),
                    };

                    if sender.send(data).is_err() {
                        tracing::debug!("all subscribers are gone");
                        channels.remove(&identifier);

                        deadge = true;

                        break;
                    }
                }
            }
        });

        let stream = BroadcastStream::new(receiver).map(|result| match result {
            Ok(data) => data,
            Err(_) => Err(PartialGraphqlError::stream_lag()),
        });

        Ok(Box::pin(stream))
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
            let loader = ExtensionLoader::new(shared, config.wasi_config, config.guest_config)?;
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
    type SharedContext = wit::SharedContext;

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
    ) -> impl Future<Output = Result<Vec<Result<Data, PartialGraphqlError>>, PartialGraphqlError>> + Send + 'f
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
                arguments: cbor::to_vec(arguments).unwrap(),
            };

            let output = instance
                .resolve_field(headers, subgraph.name(), directive, inputs)
                .await
                .map_err(|err| err.into_graphql_error(PartialErrorCode::ExtensionError))?;

            let mut results = Vec::new();

            for result in output.outputs {
                match result {
                    Ok(data) => results.push(Ok(Data::CborBytes(data))),
                    Err(error) => {
                        let error = error.into_graphql_error(PartialErrorCode::InternalServerError);
                        results.push(Err(error))
                    }
                }
            }

            Ok(results)
        }
    }

    async fn authenticate(
        &self,
        extension_id: ExtensionId,
        authorizer_id: AuthorizerId,
        headers: http::HeaderMap,
    ) -> Result<(http::HeaderMap, Vec<u8>), ErrorResponse> {
        let mut instance = self
            .get(ExtensionPoolId::Authorizer(extension_id, authorizer_id))
            .await?;

        match instance.authenticate(headers).await {
            Ok((headers, token)) => Ok((headers, token.raw)),
            Err(err) => Err(err.into_graphql_error_response(PartialErrorCode::Unauthenticated)),
        }
    }

    async fn resolve_subscription<'ctx, 'f>(
        &'ctx self,
        headers: http::HeaderMap,
        directive: ExtensionFieldDirective<'ctx, impl Anything<'ctx>>,
    ) -> Result<BoxStream<'f, Result<Data, PartialGraphqlError>>, PartialGraphqlError>
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
        let arguments = cbor::to_vec(arguments).unwrap();

        let site = wit::FieldDefinitionDirectiveSite {
            parent_type_name: field.parent_entity().name(),
            field_name: field.name(),
        };

        let directive = wit::FieldDefinitionDirective { name, site, arguments };

        let (headers, identifier) = instance
            .subscription_identifier(headers, subgraph.name(), directive.clone())
            .await
            .map_err(|err| err.into_graphql_error(PartialErrorCode::ExtensionError))?;

        match identifier {
            Some(identifier) => {
                self.resolve_deduplicated_subscription(instance, headers, identifier, subgraph, directive)
                    .await
            }
            None => {
                self.resolve_unique_subscription(instance, headers, subgraph, directive)
                    .await
            }
        }
    }

    fn authorize_query<'ctx, 'fut, Groups, QueryElements, Arguments>(
        &'ctx self,
        extension_id: ExtensionId,
        elements_grouped_by_directive_name: Groups,
    ) -> impl Future<Output = Result<AuthorizationDecisions, ErrorResponse>> + Send + 'fut
    where
        'ctx: 'fut,
        Groups: IntoIterator<Item = (&'ctx str, QueryElements)>,
        QueryElements: IntoIterator<Item = QueryElement<'ctx, Arguments>>,
        Arguments: Anything<'ctx>,
    {
        let mut directive_names = Vec::<(&'ctx str, u32, u32)>::new();
        let mut query_elements = Vec::new();
        for (directive_name, elements) in elements_grouped_by_directive_name {
            let start = query_elements.len();
            for element in elements {
                // Some help for rust-analyzer who struggles for some reason.
                let element: QueryElement<'_, _> = element;
                let arguments = cbor::to_vec(element.arguments).unwrap();

                query_elements.push(wit::QueryElement {
                    site: element.site.into(),
                    arguments,
                });
            }
            let end = query_elements.len();
            directive_names.push((directive_name, start as u32, end as u32));
        }

        async move {
            let mut instance = self.get(ExtensionPoolId::Authorization(extension_id)).await?;
            instance
                .authorize_query(wit::QueryElements {
                    directive_names,
                    elements: query_elements,
                })
                .await
                .map(Into::into)
                .map_err(|err| err.into_graphql_error_response(PartialErrorCode::Unauthorized))
        }
    }
}

#[derive(Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord)]
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
}
