mod authentication;
mod authorization;
mod field_resolver;

use std::sync::Arc;

use extension_catalog::TypeDiscriminants;
use wasmtime::Store;

use crate::{
    WasiState, cbor,
    extension::{
        ContractsExtensionInstance, ExtensionConfig, ExtensionInstance, HooksExtensionInstance,
        ResolverExtensionInstance, SelectionSetResolverExtensionInstance,
    },
};

use anyhow::Context as _;
use engine_schema::Schema;
use wasmtime::component::{Component, Linker};

use super::wit;
use wit::grafbase::sdk::directive::SchemaDirective;

pub struct SdkPre0_10_0 {
    pre: wit::SdkPre<crate::WasiState>,
    guest_config: Vec<u8>,
    #[allow(unused)]
    schema: Arc<Schema>,
    // self-reference to schema
    schema_directives: Vec<SchemaDirective<'static>>,
}

impl SdkPre0_10_0 {
    pub(crate) fn new<T: serde::Serialize>(
        schema: Arc<Schema>,
        config: &ExtensionConfig<T>,
        component: Component,
        mut linker: Linker<WasiState>,
    ) -> wasmtime::Result<Self> {
        let mut schema_directives = Vec::new();
        if matches!(
            config.r#type,
            TypeDiscriminants::FieldResolver | TypeDiscriminants::Authorization
        ) {
            for subgraph in schema.subgraphs() {
                let directives = subgraph.extension_schema_directives();

                for schema_directive in directives {
                    if schema_directive.extension_id != config.id {
                        continue;
                    }

                    let directive: SchemaDirective<'_> = SchemaDirective {
                        name: schema_directive.name(),
                        subgraph_name: subgraph.name(),
                        arguments: cbor::to_vec(schema_directive.static_arguments()).unwrap(),
                    };
                    // SAFETY: We keep an owned Arc<Schema> which is immutable (without inner
                    //         mutability), so all refs we take are kept. Ideally we wouldn't use such
                    //         tricks, but wasmtime bindgen requires either every argument or none at all
                    //         to be references. And we definitely want references for most argumnets...
                    let directive: SchemaDirective<'static> = unsafe { std::mem::transmute(directive) };
                    schema_directives.push(directive);
                }
            }
        }

        wit::Sdk::add_to_linker(&mut linker, |state| state)?;
        let instance_pre = linker.instantiate_pre(&component)?;
        Ok(Self {
            pre: wit::SdkPre::<WasiState>::new(instance_pre)?,
            guest_config: cbor::to_vec(&config.guest_config).context("Could not serialize configuration")?,
            schema,
            schema_directives,
        })
    }

    pub(crate) async fn instantiate(&self, state: WasiState) -> wasmtime::Result<Box<dyn ExtensionInstance>> {
        let mut store = Store::new(self.pre.engine(), state);

        let inner = self.pre.instantiate_async(&mut store).await?;
        inner.call_register_extension(&mut store).await?;

        inner
            .grafbase_sdk_init()
            .call_init_gateway_extension(&mut store, &self.schema_directives, &self.guest_config)
            .await?
            .map_err(wasmtime::Error::msg)?;

        let instance = ExtensionInstanceSince0_10_0 { store, inner };

        Ok(Box::new(instance))
    }
}

struct ExtensionInstanceSince0_10_0 {
    store: Store<WasiState>,
    inner: super::wit::Sdk,
}

impl ExtensionInstance for ExtensionInstanceSince0_10_0 {
    fn store(&self) -> &Store<WasiState> {
        &self.store
    }
}

impl ContractsExtensionInstance for ExtensionInstanceSince0_10_0 {}
impl HooksExtensionInstance for ExtensionInstanceSince0_10_0 {}
impl SelectionSetResolverExtensionInstance for ExtensionInstanceSince0_10_0 {}
impl ResolverExtensionInstance for ExtensionInstanceSince0_10_0 {}
