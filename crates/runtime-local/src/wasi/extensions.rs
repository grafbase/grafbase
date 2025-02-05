mod pool;

use engine_schema::Subgraph;
use extension_catalog::ExtensionId;
use futures_util::StreamExt;
use gateway_config::WasiExtensionsConfig;
use runtime::{
    error::{ErrorResponse, PartialErrorCode, PartialGraphqlError},
    extension::{AuthorizerId, Data, ExtensionDirective, ExtensionRuntime},
    hooks::{Anything, EdgeDefinition},
};
use semver::Version;
use std::{collections::HashMap, sync::Arc};
use tokio::task::JoinHandle;
use wasi_component_loader::{ChannelLogSender, ComponentLoader, FieldDefinition, SharedContext};
pub use wasi_component_loader::{Directive, ExtensionType};

use pool::Pool;

use super::guest_error_as_gql;

#[derive(Clone, Default)]
pub struct WasiExtensions(Option<Arc<WasiExtensionsInner>>);

impl WasiExtensions {
    pub async fn new(
        access_log: ChannelLogSender,
        extensions: Vec<ExtensionConfig>,
    ) -> Result<Self, wasi_component_loader::Error> {
        if extensions.is_empty() {
            return Ok(Self(None));
        }

        let instance_pools = create_pools(access_log, extensions).await?;
        let inner = WasiExtensionsInner { instance_pools };

        Ok(Self(Some(Arc::new(inner))))
    }
}

async fn create_pools(
    access_log: ChannelLogSender,
    extensions: Vec<ExtensionConfig>,
) -> Result<HashMap<ExtensionPoolId, Pool>, wasi_component_loader::Error> {
    type Handle = JoinHandle<Result<Option<(ExtensionPoolId, Pool)>, wasi_component_loader::Error>>;

    let mut creating_pools: Vec<Handle> = Vec::new();

    for config in extensions {
        let access_log = access_log.clone();

        creating_pools.push(tokio::task::spawn_blocking(move || {
            let manager_config = pool::ComponentManagerConfig {
                extension_type: config.extension_type,
                schema_directives: config.schema_directives,
            };

            tracing::info!("Loading extension {} {}", config.name, config.version);

            match ComponentLoader::extensions(config.name, config.wasi_config)? {
                Some(loader) => {
                    let pool = Pool::new(
                        loader,
                        manager_config,
                        config.max_pool_size,
                        config.extension_config,
                        access_log,
                    );

                    Ok(Some((config.id, pool)))
                }
                None => Ok(None),
            }
        }));
    }

    let mut pools = HashMap::new();

    let mut creating_pools = futures_util::stream::iter(creating_pools)
        .buffer_unordered(std::thread::available_parallelism().map(|i| i.get()).unwrap_or(1));

    while let Some(result) = creating_pools.next().await {
        match result.unwrap() {
            Ok(Some((id, pool))) => {
                pools.insert(id, pool);
            }
            Ok(None) => {}
            Err(error) => return Err(error),
        }
    }

    Ok(pools)
}

impl ExtensionRuntime for WasiExtensions {
    type SharedContext = SharedContext;

    async fn resolve_field<'a>(
        &self,
        extension_id: ExtensionId,
        subgraph: Subgraph<'a>,
        context: &Self::SharedContext,
        field: EdgeDefinition<'a>,
        directive: ExtensionDirective<'a, impl Anything<'a>>,
        inputs: impl IntoIterator<Item: Anything<'a>> + Send,
    ) -> Result<Vec<Result<Data, PartialGraphqlError>>, PartialGraphqlError> {
        let Some(inner) = self.0.as_ref() else {
            return Err(PartialGraphqlError::internal_extension_error());
        };

        let id = ExtensionPoolId::Resolver(extension_id);

        let Some(pool) = inner.instance_pools.get(&id) else {
            return Err(PartialGraphqlError::internal_extension_error());
        };

        let mut instance = pool.get().await;

        let directive = Directive::new(
            directive.name.to_string(),
            subgraph.name().to_string(),
            &directive.static_arguments,
        );

        let definition = FieldDefinition {
            type_name: field.parent_type_name.to_string(),
            name: field.field_name.to_string(),
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
                            let error = guest_error_as_gql(error, PartialErrorCode::Unauthorized);

                            results.push(Err(error))
                        }
                    }
                }

                Ok(results)
            }
            Err(error) => match error {
                wasi_component_loader::Error::Guest(error) => {
                    let error = guest_error_as_gql(error, PartialErrorCode::Unauthorized);

                    Err(error)
                }
                _ => Err(PartialGraphqlError::internal_extension_error()),
            },
        }
    }

    async fn authenticate(
        &self,
        extension_id: ExtensionId,
        authorizer_id: AuthorizerId,
        headers: http::HeaderMap,
    ) -> Result<(http::HeaderMap, HashMap<String, serde_json::Value>), ErrorResponse> {
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
            Ok(result) => Ok(result),
            Err(wasi_component_loader::GatewayError::Guest(error)) => {
                let status =
                    http::StatusCode::from_u16(error.status_code).unwrap_or(http::StatusCode::INTERNAL_SERVER_ERROR);

                let errors = error
                    .errors
                    .into_iter()
                    .map(|error| guest_error_as_gql(error, PartialErrorCode::Unauthorized))
                    .collect();

                Err(ErrorResponse { status, errors })
            }
            Err(wasi_component_loader::GatewayError::Internal(_)) => Err(ErrorResponse {
                status: http::StatusCode::INTERNAL_SERVER_ERROR,
                errors: vec![PartialGraphqlError::internal_extension_error()],
            }),
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

struct WasiExtensionsInner {
    instance_pools: HashMap<ExtensionPoolId, Pool>,
}

pub struct ExtensionConfig {
    pub id: ExtensionPoolId,
    pub name: String,
    pub version: Version,
    pub extension_type: ExtensionType,
    pub schema_directives: Vec<Directive>,
    pub max_pool_size: Option<usize>,
    pub wasi_config: WasiExtensionsConfig,
    // CBOR encoded extension configuration
    pub extension_config: Vec<u8>,
}
