mod authentication;
mod authorization;
mod field_resolver;
mod hooks;
mod selection_set_resolver;

use crate::extension::api::since_0_15_0::instance::utils::{
    create_complete_subgraph_schemas, create_subgraph_schema_directives,
};
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
use crate::extension::api::since_0_15_0::wit::schema as ws;

pub struct SdkPre0_17_0 {
    pre: wit::SdkPre<crate::WasiState>,
    guest_config: Vec<u8>,
    #[allow(unused)]
    schema: Arc<Schema>,
    // self-reference to schema
    subgraph_schemas: Vec<(&'static str, ws::Schema<'static>)>,
}

impl SdkPre0_17_0 {
    pub(crate) fn new<T: serde::Serialize>(
        schema: Arc<Schema>,
        config: &ExtensionConfig<T>,
        component: Component,
        mut linker: Linker<WasiState>,
    ) -> crate::Result<Self> {
        let subgraph_schemas: Vec<(&str, ws::Schema<'_>)> = match config.r#type {
            TypeDiscriminants::Authentication | TypeDiscriminants::Authorization | TypeDiscriminants::Hooks => {
                Vec::new()
            }
            TypeDiscriminants::FieldResolver => create_subgraph_schema_directives(&schema, config.id),
            TypeDiscriminants::SelectionSetResolver => create_complete_subgraph_schemas(&schema, config.id),
            TypeDiscriminants::Resolver => create_subgraph_schema_directives(&schema, config.id),
        };

        // SAFETY: We keep an owned Arc<Schema> which is immutable (without inner
        //         mutability), so all refs we take are kept. Ideally we wouldn't use such
        //         tricks, but wasmtime bindgen requires either every argument or none at all
        //         to be references. And we definitely want references for most argumnets...
        let subgraph_schemas: Vec<(&'static str, ws::Schema<'static>)> =
            unsafe { std::mem::transmute(subgraph_schemas) };

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

        // TODO: Actual implementation
        let access_log = store.data_mut().push_resource(())?;

        inner
            .call_init(&mut store, &self.subgraph_schemas, &self.guest_config, access_log)
            .await??;

        let instance = ExtensionInstanceSince0_17_0 {
            store,
            inner,
            poisoned: false,
        };

        Ok(Box::new(instance))
    }
}

struct ExtensionInstanceSince0_17_0 {
    store: Store<WasiState>,
    inner: super::wit::Sdk,
    poisoned: bool,
}

impl ExtensionInstance for ExtensionInstanceSince0_17_0 {
    fn recycle(&mut self) -> Result<(), Error> {
        if self.poisoned {
            return Err(anyhow::anyhow!("this instance is poisoned").into());
        }

        Ok(())
    }
}
