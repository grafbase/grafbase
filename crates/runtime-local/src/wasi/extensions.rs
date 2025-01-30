mod pool;

use engine_schema::Subgraph;
use extension_catalog::ExtensionId;
use gateway_config::WasiExtensionsConfig;
use runtime::{
    error::{PartialErrorCode, PartialGraphqlError},
    extension::{Data, ExtensionDirective, ExtensionRuntime},
    hooks::{Anything, EdgeDefinition},
};
use std::{collections::HashMap, sync::Arc};
use wasi_component_loader::{ChannelLogSender, ComponentLoader, FieldDefinition, SharedContext};
pub use wasi_component_loader::{Directive, ExtensionType};

use pool::Pool;

use super::guest_error_as_gql;

#[derive(Clone, Default)]
pub struct WasiExtensions(Option<Arc<WasiExtensionsInner>>);

impl WasiExtensions {
    pub fn new(
        access_log: ChannelLogSender,
        extensions: Vec<ExtensionConfig>,
    ) -> Result<Self, wasi_component_loader::Error> {
        if extensions.is_empty() {
            return Ok(Self(None));
        }

        let mut instance_pools = HashMap::new();

        for config in extensions {
            let manager_config = pool::ComponentManagerConfig {
                extension_type: config.extension_type,
                schema_directives: config.schema_directives,
            };

            let Some(loader) = ComponentLoader::extensions(config.name, config.wasi_config)? else {
                continue;
            };

            let pool = Pool::new(loader, manager_config, config.max_pool_size, access_log.clone());
            instance_pools.insert(config.id, pool);
        }

        let inner = WasiExtensionsInner { instance_pools };

        Ok(Self(Some(Arc::new(inner))))
    }
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

        let Some(pool) = inner.instance_pools.get(&extension_id) else {
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
}

struct WasiExtensionsInner {
    instance_pools: HashMap<ExtensionId, Pool>,
}

#[derive(Debug, Clone)]
pub struct ExtensionConfig {
    pub id: ExtensionId,
    pub name: String,
    pub extension_type: ExtensionType,
    pub schema_directives: Vec<Directive>,
    pub max_pool_size: Option<usize>,
    pub wasi_config: WasiExtensionsConfig,
}
