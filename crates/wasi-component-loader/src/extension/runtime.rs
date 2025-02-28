mod authorization;

use crate::{cbor, extension::ExtensionLoader, resources::SharedResources};

use super::{
    ExtensionGuestConfig, InputList,
    pool::{ExtensionGuard, Pool},
    wit,
};
use engine_schema::Definition;
use extension_catalog::ExtensionId;
use futures::stream::BoxStream;
use futures_util::{StreamExt, stream};
use gateway_config::WasiExtensionsConfig;
use runtime::{
    error::{ErrorResponse, PartialErrorCode, PartialGraphqlError},
    extension::{AuthorizationDecisions, AuthorizerId, Data, DirectiveSite, ExtensionFieldDirective, ExtensionRuntime},
    hooks::Anything,
};
use std::{
    collections::{HashMap, VecDeque},
    future::Future,
    sync::Arc,
};
use tokio::task::JoinHandle;

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

    async fn get(&self, id: ExtensionPoolId) -> Result<ExtensionGuard, PartialGraphqlError> {
        let pool = self
            .0
            .as_ref()
            .and_then(|inner| inner.instance_pools.get(&id))
            .ok_or_else(PartialGraphqlError::internal_extension_error)?;
        Ok(pool.get().await)
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
            let mut instance = self.get(ExtensionPoolId::Resolver(extension_id)).await?;

            let directive = wit::FieldDefinitionDirective {
                name,
                site: wit::FieldDefinitionDirectiveSite {
                    parent_type_name: field.parent_entity().name(),
                    field_name: field.name(),
                    arguments: cbor::to_vec(arguments).unwrap(),
                },
            };

            let output = instance
                .resolve_field(context.clone(), subgraph.name(), directive, inputs)
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
        context: &'ctx Self::SharedContext,
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
        let directive = wit::FieldDefinitionDirective {
            name,
            site: wit::FieldDefinitionDirectiveSite {
                parent_type_name: field.parent_entity().name(),
                field_name: field.name(),
                arguments: cbor::to_vec(arguments).unwrap(),
            },
        };

        instance
            .resolve_subscription(context.clone(), subgraph.name(), directive)
            .await
            .map_err(|err| err.into_graphql_error(PartialErrorCode::ExtensionError))?;

        let stream = stream::unfold((instance, VecDeque::new()), async move |(mut instance, mut tail)| {
            if let Some(data) = tail.pop_front() {
                return Some((data, (instance, tail)));
            }

            let item = match instance.resolve_next_subscription_item().await {
                Ok(Some(item)) => {
                    tracing::debug!("subscription item resolved");
                    item
                }
                Ok(None) => {
                    tracing::debug!("subscription completed");
                    return None;
                }
                Err(e) => {
                    tracing::error!("Error resolving subscription item: {e}");
                    return Some((Err(PartialGraphqlError::internal_extension_error()), (instance, tail)));
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

    async fn authorize_query<'ctx>(
        &'ctx self,
        context: &'ctx Self::SharedContext,
        extension_id: ExtensionId,
        // (directive name, (definition, arguments))
        elements: impl IntoIterator<
            Item = (
                &'ctx str,
                impl IntoIterator<Item = DirectiveSite<'ctx, impl Anything<'ctx>>> + Send + 'ctx,
            ),
        > + Send
        + 'ctx,
    ) -> Result<AuthorizationDecisions, ErrorResponse> {
        let mut instance = self.get(ExtensionPoolId::Authorization(extension_id)).await?;
        let mut directive_names = Vec::<(&str, u32, u32)>::new();
        let mut query_elements = Vec::new();
        for (name, sites) in elements {
            let start = query_elements.len();
            for site in sites {
                // Some help for rust-analyzer who struggles for some reason.
                let site: DirectiveSite<'_, _> = site;
                let arguments = cbor::to_vec(site.arguments).unwrap();

                let query_element = match site.definition {
                    Definition::InputObject(_) => unreachable!("We don't authorize inputs."),
                    Definition::Enum(def) => wit::DirectiveSite::Enum(wit::EnumDirectiveSite {
                        enum_name: def.name(),
                        arguments,
                    }),
                    Definition::Interface(def) => wit::DirectiveSite::Interface(wit::InterfaceDirectiveSite {
                        interface_name: def.name(),
                        arguments,
                    }),
                    Definition::Object(def) => wit::DirectiveSite::Object(wit::ObjectDirectiveSite {
                        object_name: def.name(),
                        arguments,
                    }),
                    Definition::Scalar(def) => wit::DirectiveSite::Scalar(wit::ScalarDirectiveSite {
                        scalar_name: def.name(),
                        arguments,
                    }),
                    Definition::Union(def) => wit::DirectiveSite::Union(wit::UnionDirectiveSite {
                        union_name: def.name(),
                        arguments,
                    }),
                };

                query_elements.push(query_element);
            }
            let end = query_elements.len();
            directive_names.push((name, start as u32, end as u32));
        }

        instance
            .authorize_query(
                context.clone(),
                wit::QueryElements {
                    directive_names: directive_names.as_slice(),
                    elements: query_elements.as_slice(),
                },
            )
            .await
            .map(Into::into)
            .map_err(|err| err.into_graphql_error_response(PartialErrorCode::Unauthorized))
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
