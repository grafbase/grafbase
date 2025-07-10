mod authentication;
mod authorization;
mod field_resolver;
mod hooks;
mod resolver;
mod selection_set_resolver;

use crate::extension::api::since_0_17_0::wit::schema::Schema as WitSchema;
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

pub struct SdkPre0_19_0 {
    pre: wit::SdkPre<crate::WasiState>,
    guest_config: Vec<u8>,
    #[allow(unused)]
    schema: Arc<Schema>,
    // self-reference to schema
    subgraph_schemas: Vec<(&'static str, WitSchema<'static>)>,
    can_skip_sending_events: bool,
    logging_filter: String,
}

impl SdkPre0_19_0 {
    pub(crate) fn new<T: serde::Serialize>(
        schema: Arc<Schema>,
        config: &ExtensionConfig<T>,
        component: Component,
        mut linker: Linker<WasiState>,
    ) -> crate::Result<Self> {
        let subgraph_schemas: Vec<(&str, WitSchema<'_>)> = match config.r#type {
            TypeDiscriminants::Authentication | TypeDiscriminants::Authorization | TypeDiscriminants::Hooks => {
                Vec::new()
            }
            TypeDiscriminants::Resolver => {
                crate::extension::api::since_0_17_0::instance::schema::create_complete_subgraph_schemas(
                    &schema, config.id,
                )
            }
            TypeDiscriminants::FieldResolver | TypeDiscriminants::SelectionSetResolver => {
                unreachable!("Not supported anymore in the SDK.")
            }
        };

        // SAFETY: We keep an owned Arc<Schema> which is immutable (without inner
        //         mutability), so all refs we take are kept. Ideally we wouldn't use such
        //         tricks, but wasmtime bindgen requires either every argument or none at all
        //         to be references. And we definitely want references for most argumnets...
        let subgraph_schemas: Vec<(&'static str, WitSchema<'static>)> =
            unsafe { std::mem::transmute(subgraph_schemas) };

        super::wit::grafbase::sdk::shared_context::add_to_linker_impl(&mut linker)?;
        wit::Sdk::add_to_linker(&mut linker, |state| state)?;

        let instance_pre = linker.instantiate_pre(&component)?;

        Ok(Self {
            pre: wit::SdkPre::<WasiState>::new(instance_pre)?,
            guest_config: cbor::to_vec(&config.guest_config).context("Could not serialize configuration")?,
            schema,
            subgraph_schemas,
            can_skip_sending_events: config.can_skip_sending_events,
            logging_filter: config.logging_filter.clone(),
        })
    }

    pub(crate) async fn instantiate(&self, state: WasiState) -> crate::Result<Box<dyn ExtensionInstance>> {
        let mut store = Store::new(self.pre.engine(), state);

        let inner = self.pre.instantiate_async(&mut store).await?;
        inner.call_register_extension(&mut store).await?;

        inner
            .call_init(
                &mut store,
                &self.subgraph_schemas,
                &self.guest_config,
                self.can_skip_sending_events,
                &self.logging_filter,
            )
            .await??;

        let instance = ExtensionInstanceSince0_19_0 {
            store,
            inner,
            poisoned: false,
        };

        Ok(Box::new(instance))
    }
}

struct ExtensionInstanceSince0_19_0 {
    store: Store<WasiState>,
    inner: super::wit::Sdk,
    poisoned: bool,
}

impl ExtensionInstance for ExtensionInstanceSince0_19_0 {
    fn recycle(&mut self) -> Result<(), Error> {
        if self.poisoned {
            return Err(anyhow::anyhow!("this instance is poisoned").into());
        }

        Ok(())
    }
}
