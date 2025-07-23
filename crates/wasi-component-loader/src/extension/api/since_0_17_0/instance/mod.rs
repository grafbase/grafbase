mod authentication;
mod authorization;
mod hooks;
mod resolver;
pub mod schema;

use anyhow::Context as _;
use engine_schema::Schema;
use extension_catalog::TypeDiscriminants;
use std::sync::Arc;
use wasmtime::{
    Store,
    component::{Component, Linker},
};

use crate::{
    InstanceState, cbor,
    extension::{
        ContractsExtensionInstance, ExtensionConfig, ExtensionInstance, FieldResolverExtensionInstance,
        SelectionSetResolverExtensionInstance,
    },
};

use super::wit;

pub struct SdkPre0_17_0 {
    pre: wit::SdkPre<crate::InstanceState>,
    guest_config: Vec<u8>,
    #[allow(unused)]
    schema: Arc<Schema>,
    // self-reference to schema
    subgraph_schemas: Vec<(&'static str, wit::schema::Schema<'static>)>,
    can_skip_sending_events: bool,
}

impl SdkPre0_17_0 {
    pub(crate) fn new<T: serde::Serialize>(
        schema: Arc<Schema>,
        config: &ExtensionConfig<T>,
        component: Component,
        mut linker: Linker<InstanceState>,
    ) -> wasmtime::Result<Self> {
        let subgraph_schemas: Vec<(&str, wit::schema::Schema<'_>)> = match config.r#type {
            TypeDiscriminants::Resolver => schema::create_complete_subgraph_schemas(&schema, config.id),
            TypeDiscriminants::FieldResolver | TypeDiscriminants::SelectionSetResolver => {
                unreachable!("Not supported anymore in the SDK.")
            }
            TypeDiscriminants::Authentication
            | TypeDiscriminants::Authorization
            | TypeDiscriminants::Hooks
            | TypeDiscriminants::Contracts => Vec::new(),
        };

        // SAFETY: We keep an owned Arc<Schema> which is immutable (without inner
        //         mutability), so all refs we take are kept. Ideally we wouldn't use such
        //         tricks, but wasmtime bindgen requires either every argument or none at all
        //         to be references. And we definitely want references for most argumnets...
        let subgraph_schemas: Vec<(&'static str, wit::schema::Schema<'static>)> =
            unsafe { std::mem::transmute(subgraph_schemas) };

        super::wit::shared_context::add_to_linker_impl(&mut linker)?;
        wit::Sdk::add_to_linker(&mut linker, |state| state)?;

        let instance_pre = linker.instantiate_pre(&component)?;

        Ok(Self {
            pre: wit::SdkPre::<InstanceState>::new(instance_pre)?,
            guest_config: cbor::to_vec(&config.guest_config).context("Could not serialize configuration")?,
            schema,
            subgraph_schemas,
            can_skip_sending_events: config.can_skip_sending_events,
        })
    }

    pub(crate) async fn instantiate(&self, state: InstanceState) -> wasmtime::Result<Box<dyn ExtensionInstance>> {
        let mut store = Store::new(self.pre.engine(), state);

        let inner = self.pre.instantiate_async(&mut store).await?;
        inner.call_register_extension(&mut store).await?;

        inner
            .call_init(
                &mut store,
                &self.subgraph_schemas,
                &self.guest_config,
                self.can_skip_sending_events,
            )
            .await?
            .map_err(wasmtime::Error::msg)?;

        let instance = ExtensionInstanceSince0_17_0 { store, inner };

        Ok(Box::new(instance))
    }
}

struct ExtensionInstanceSince0_17_0 {
    store: Store<InstanceState>,
    inner: super::wit::Sdk,
}

impl ExtensionInstance for ExtensionInstanceSince0_17_0 {
    fn store(&self) -> &Store<InstanceState> {
        &self.store
    }
}

impl ContractsExtensionInstance for ExtensionInstanceSince0_17_0 {}
impl FieldResolverExtensionInstance for ExtensionInstanceSince0_17_0 {}
impl SelectionSetResolverExtensionInstance for ExtensionInstanceSince0_17_0 {}
