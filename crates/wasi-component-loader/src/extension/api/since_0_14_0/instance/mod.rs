mod authentication;
mod authorization;
mod field_resolver;
mod selection_set_resolver;

use anyhow::Context as _;
use engine_schema::Schema;
use extension_catalog::TypeDiscriminants;
use wasmtime::{
    Store,
    component::{Component, Linker},
};

use crate::{
    Error, WasiState, cbor,
    extension::{ExtensionConfig, ExtensionInstance},
};

use super::wit;
use wit::schema as ws;

pub struct SdkPre0_14_0 {
    pre: wit::SdkPre<crate::WasiState>,
    guest_config: Vec<u8>,
    subgraph_schemas: Vec<(String, ws::Schema)>,
}

impl SdkPre0_14_0 {
    pub(crate) fn new<T: serde::Serialize>(
        schema: &Schema,
        config: &ExtensionConfig<T>,
        component: Component,
        mut linker: Linker<WasiState>,
    ) -> crate::Result<Self> {
        let mut subgraph_schemas = Vec::new();
        match config.r#type {
            TypeDiscriminants::Authentication | TypeDiscriminants::Authorization => (),
            TypeDiscriminants::FieldResolver => {
                for subgraph in schema.subgraphs() {
                    let mut directives = Vec::new();

                    for schema_directive in subgraph.extension_schema_directives() {
                        if schema_directive.extension_id != config.id {
                            continue;
                        }

                        directives.push(ws::Directive {
                            name: schema_directive.name().to_string(),
                            arguments: cbor::to_vec(schema_directive.static_arguments()).unwrap(),
                        });
                    }

                    if !directives.is_empty() {
                        subgraph_schemas.push((
                            subgraph.name().to_string(),
                            ws::Schema {
                                directives,
                                definitions: Vec::new(),
                            },
                        ));
                    }
                }
            }
            TypeDiscriminants::SelectionSetResolver => {
                for subgraph in schema.subgraphs() {
                    let mut directives = Vec::new();
                    for schema_directive in subgraph.extension_schema_directives() {
                        if schema_directive.extension_id != config.id {
                            continue;
                        }
                        directives.push(ws::Directive {
                            name: schema_directive.name().to_string(),
                            arguments: cbor::to_vec(schema_directive.static_arguments()).unwrap(),
                        });
                    }
                    if !directives.is_empty() {
                        subgraph_schemas.push((
                            subgraph.name().to_string(),
                            ws::Schema {
                                directives,
                                definitions: Vec::new(),
                            },
                        ));
                    }
                }
            }
        }

        wit::Sdk::add_to_linker(&mut linker, |state| state)?;
        let instance_pre = linker.instantiate_pre(&component)?;
        Ok(Self {
            pre: wit::SdkPre::<WasiState>::new(instance_pre)?,
            guest_config: cbor::to_vec(&config.guest_config).context("Could not serialize configuration")?,
            subgraph_schemas,
        })
    }

    pub(crate) async fn instantiate(&self, state: WasiState) -> crate::Result<Box<dyn ExtensionInstance>> {
        let mut store = Store::new(self.pre.engine(), state);

        let inner = self.pre.instantiate_async(&mut store).await?;
        inner.call_register_extension(&mut store).await?;

        inner
            .call_init(&mut store, &self.subgraph_schemas, &self.guest_config)
            .await??;

        let instance = ExtensionInstanceSince0_14_0 {
            store,
            inner,
            poisoned: false,
        };

        Ok(Box::new(instance))
    }
}

struct ExtensionInstanceSince0_14_0 {
    store: Store<WasiState>,
    inner: super::wit::Sdk,
    poisoned: bool,
}

impl ExtensionInstance for ExtensionInstanceSince0_14_0 {
    fn recycle(&mut self) -> Result<(), Error> {
        if self.poisoned {
            return Err(anyhow::anyhow!("this instance is poisoned").into());
        }

        Ok(())
    }
}
