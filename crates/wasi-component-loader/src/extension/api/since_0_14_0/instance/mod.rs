mod authentication;
mod authorization;
mod field_resolver;
mod selection_set_resolver;

use anyhow::Context as _;
use engine_schema::Schema;
use extension_catalog::TypeDiscriminants;
use std::sync::Arc;
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
    #[allow(unused)]
    schema: Arc<Schema>,
    // self-reference to schema
    subgraph_schemas: Vec<(&'static str, ws::Schema<'static>)>,
}

impl SdkPre0_14_0 {
    pub(crate) fn new<T: serde::Serialize>(
        schema: Arc<Schema>,
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

                        let directive = ws::Directive {
                            name: schema_directive.name(),
                            arguments: cbor::to_vec(schema_directive.static_arguments()).unwrap(),
                        };
                        // SAFETY: We keep an owned Arc<Schema> which is immutable (without inner
                        //         mutability), so all refs we take are kept. Ideally we wouldn't use such
                        //         tricks, but wasmtime bindgen requires either every argument or none at all
                        //         to be references. And we definitely want references for most argumnets...
                        let directive: ws::Directive<'static> = unsafe { std::mem::transmute(directive) };
                        directives.push(directive);
                    }

                    if !directives.is_empty() {
                        // SAFETY: We keep an owned Arc<Schema> which is immutable
                        let name: &'static str = unsafe { std::mem::transmute(subgraph.name()) };
                        subgraph_schemas.push((
                            name,
                            ws::Schema {
                                directives,
                                definitions: Vec::new(),
                            },
                        ));
                    }
                }
            }
            TypeDiscriminants::SelectionSetResolver => {
                let mut subgraph_ids = schema
                    .resolvers()
                    .filter_map(|resolver| match resolver.variant() {
                        engine_schema::ResolverDefinitionVariant::SelectionSetResolverExtension(res)
                            if res.extension_id == config.id =>
                        {
                            Some(res.subgraph_id)
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                subgraph_ids.sort_unstable();
                subgraph_ids.dedup();
            }
        }

        wit::Sdk::add_to_linker(&mut linker, |state| state)?;
        let instance_pre = linker.instantiate_pre(&component)?;
        Ok(Self {
            pre: wit::SdkPre::<WasiState>::new(instance_pre)?,
            guest_config: cbor::to_vec(&config.guest_config).context("Could not serialize configuration")?,
            schema,
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
