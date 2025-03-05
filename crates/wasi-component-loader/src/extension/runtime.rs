mod authorization;
mod subscription;

use crate::{cbor, extension::ExtensionLoader, resources::SharedResources};

use super::{
    ExtensionGuestConfig, InputList,
    pool::{ExtensionGuard, Pool},
    wit,
};
use dashmap::DashMap;
use extension_catalog::ExtensionId;
use futures::stream::BoxStream;
use futures_util::{StreamExt, stream};
use gateway_config::WasiExtensionsConfig;
use runtime::{
    error::{ErrorResponse, PartialErrorCode, PartialGraphqlError},
    extension::{AuthorizationDecisions, AuthorizerId, Data, ExtensionFieldDirective, ExtensionRuntime, QueryElement},
    hooks::Anything,
};
use std::{collections::HashMap, future::Future, sync::Arc};
use subscription::{DeduplicatedSubscription, UniqueSubscription};
use tokio::{sync::broadcast, task::JoinHandle};

#[derive(Clone, Default)]
pub struct ExtensionsWasiRuntime(Option<Arc<WasiExtensionsInner>>);

type Subscriptions = Arc<DashMap<Vec<u8>, broadcast::Sender<Result<Arc<Data>, PartialGraphqlError>>>>;

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

    fn subscriptions(&self) -> Result<Subscriptions, PartialGraphqlError> {
        let subscriptions = self
            .0
            .as_ref()
            .ok_or_else(PartialGraphqlError::internal_extension_error)?
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
                arguments: &cbor::to_vec(arguments).unwrap(),
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
    ) -> Result<BoxStream<'f, Result<Arc<Data>, PartialGraphqlError>>, PartialGraphqlError>
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
            .subscription_key(headers, subgraph.name(), directive.clone())
            .await
            .map_err(|err| err.into_graphql_error(PartialErrorCode::ExtensionError))?;

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
