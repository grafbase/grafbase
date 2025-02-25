use crate::{error::guest::guest_error_as_gql, extension::ExtensionLoader, resources::SharedResources};

use super::{ExtensionGuestConfig, InputList, pool::Pool, wit};
use deadpool::managed::Object;
use extension_catalog::ExtensionId;
use futures::stream::BoxStream;
use futures_util::StreamExt;
use gateway_config::WasiExtensionsConfig;
use runtime::{
    error::{ErrorResponse, PartialErrorCode, PartialGraphqlError},
    extension::{AuthorizerId, Data, ExtensionFieldDirective, ExtensionRuntime},
    hooks::Anything,
};
use std::{collections::HashMap, future::Future, sync::Arc};
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_stream::wrappers::ReceiverStream;

#[derive(Clone, Default)]
pub struct ExtensionsWasiRuntime(Option<Arc<WasiExtensionsInner>>);

struct WasiExtensionsInner {
    instance_pools: HashMap<ExtensionPoolId, Pool>,
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
        let inner = WasiExtensionsInner { instance_pools };

        Ok(Self(Some(Arc::new(inner))))
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

    let mut creating_pools = futures_util::stream::iter(creating_pools)
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
        context: &'ctx Self::SharedContext,
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
            let Some(inner) = self.0.as_ref() else {
                return Err(PartialGraphqlError::internal_extension_error());
            };

            let id = ExtensionPoolId::Resolver(extension_id);

            let Some(pool) = inner.instance_pools.get(&id) else {
                return Err(PartialGraphqlError::internal_extension_error());
            };

            let mut instance = pool.get().await;

            let arguments = minicbor_serde::to_vec(arguments).unwrap();
            let directive = wit::Directive {
                name,
                subgraph_name: subgraph.name(),
                arguments: &arguments,
            };

            let definition = wit::FieldDefinition {
                type_name: field.parent_entity().name(),
                name: field.name(),
            };

            let result = instance
                .resolve_field(context.clone(), directive, definition, inputs)
                .await;

            match result {
                Ok(output) => {
                    let mut results = Vec::new();

                    for result in output.outputs {
                        match result {
                            Ok(data) => results.push(Ok(Data::CborBytes(data))),
                            Err(error) => {
                                let error = guest_error_as_gql(error, PartialErrorCode::InternalServerError);
                                results.push(Err(error))
                            }
                        }
                    }

                    Ok(results)
                }
                Err(error) => match error {
                    crate::Error::Guest(error) => {
                        let error = guest_error_as_gql(error, PartialErrorCode::InternalServerError);
                        Err(error)
                    }
                    _ => Err(PartialGraphqlError::internal_extension_error()),
                },
            }
        }
    }

    async fn authenticate(
        &self,
        extension_id: ExtensionId,
        authorizer_id: AuthorizerId,
        headers: http::HeaderMap,
    ) -> Result<(http::HeaderMap, Vec<u8>), ErrorResponse> {
        let Some(inner) = self.0.as_ref() else {
            return Err(ErrorResponse {
                status: http::StatusCode::INTERNAL_SERVER_ERROR,
                errors: vec![PartialGraphqlError::internal_extension_error()],
            });
        };

        let id = ExtensionPoolId::Authorizer(extension_id, authorizer_id);

        let Some(pool) = inner.instance_pools.get(&id) else {
            return Err(ErrorResponse {
                status: http::StatusCode::INTERNAL_SERVER_ERROR,
                errors: vec![PartialGraphqlError::internal_extension_error()],
            });
        };

        let mut instance = pool.get().await;

        let result = instance.authenticate(headers).await;

        match result {
            Ok((headers, token)) => Ok((headers, token.raw)),
            Err(crate::GatewayError::Guest(error)) => {
                let status =
                    http::StatusCode::from_u16(error.status_code).unwrap_or(http::StatusCode::INTERNAL_SERVER_ERROR);

                let errors = error
                    .errors
                    .into_iter()
                    .map(|error| guest_error_as_gql(error, PartialErrorCode::Unauthorized))
                    .collect();

                Err(ErrorResponse { status, errors })
            }
            Err(crate::GatewayError::Internal(_)) => Err(ErrorResponse {
                status: http::StatusCode::INTERNAL_SERVER_ERROR,
                errors: vec![PartialGraphqlError::internal_extension_error()],
            }),
        }
    }

    async fn resolve_subscription<'ctx, 'f>(
        &'ctx self,
        context: &'ctx Self::SharedContext,
        directive: ExtensionFieldDirective<'ctx, impl Anything<'ctx>>,
    ) -> Result<BoxStream<'f, Result<Data, PartialGraphqlError>>, PartialGraphqlError>
    where
        'ctx: 'f,
    {
        let Some(inner) = self.0.as_ref() else {
            return Err(PartialGraphqlError::internal_extension_error());
        };

        let ExtensionFieldDirective {
            extension_id,
            subgraph,
            field,
            name,
            arguments,
        } = directive;

        let id = ExtensionPoolId::Resolver(extension_id);

        let Some(pool) = inner.instance_pools.get(&id) else {
            return Err(PartialGraphqlError::internal_extension_error());
        };

        let mut instance = Object::take(pool.get().await.into_inner());
        let arguments = minicbor_serde::to_vec(arguments).unwrap();

        let directive = wit::Directive {
            name,
            subgraph_name: subgraph.name(),
            arguments: &arguments,
        };

        let definition = wit::FieldDefinition {
            type_name: field.parent_entity().name(),
            name: field.name(),
        };

        let result = instance
            .resolve_subscription(context.clone(), directive, definition)
            .await;

        match result {
            Ok(()) => {
                let (tx, rx) = mpsc::channel(10000);

                tokio::spawn(async move {
                    'outer: loop {
                        let item = match instance.resolve_next_subscription_item().await {
                            Ok(Some(item)) => item,
                            Ok(None) => {
                                tracing::debug!("subscription completed");
                                break;
                            }
                            Err(e) => {
                                tracing::error!("error resolving subscription item: {}", e);
                                break;
                            }
                        };

                        for item in item.outputs {
                            match item {
                                Ok(item) => {
                                    if tx.send(Ok(Data::CborBytes(item))).await.is_err() {
                                        tracing::debug!("channel closed");
                                        break 'outer;
                                    }
                                }
                                Err(error) => {
                                    let error = guest_error_as_gql(error, PartialErrorCode::InternalServerError);

                                    if tx.send(Err(error)).await.is_err() {
                                        tracing::debug!("channel closed");
                                        break 'outer;
                                    }
                                }
                            }
                        }
                    }
                });

                Ok(Box::pin(ReceiverStream::new(rx)))
            }
            Err(error) => match error {
                crate::Error::Guest(error) => {
                    let error = guest_error_as_gql(error, PartialErrorCode::InternalServerError);
                    Err(error)
                }
                _ => Err(PartialGraphqlError::internal_extension_error()),
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub enum ExtensionPoolId {
    Resolver(ExtensionId),
    Authorizer(ExtensionId, AuthorizerId),
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
