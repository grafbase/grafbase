mod since_0_10_0;
mod since_0_8_0;
mod since_0_9_0;

use since_0_8_0::instance::ExtensionInstanceSince080;
use since_0_9_0::instance::ExtensionInstanceSince090;
use since_0_10_0::ExtensionInstanceSince0_10_0;
pub use since_0_10_0::world as wit;

use super::ExtensionInstance;
use crate::WasiState;
use semver::Version;
use wasmtime::{
    Store,
    component::{Component, Linker},
};

pub(crate) enum SdkPre {
    Since0_8_0(since_0_8_0::wit::SdkPre<WasiState>),
    Since0_9_0(since_0_9_0::wit::SdkPre<WasiState>),
    Since0_10_0(since_0_10_0::SdkPre<WasiState>),
}

impl SdkPre {
    pub(crate) fn initialize(
        sdk_version: &Version,
        component: Component,
        mut linker: Linker<WasiState>,
    ) -> crate::Result<SdkPre> {
        Ok(match sdk_version {
            v if v < &Version::new(0, 9, 0) => {
                use since_0_8_0::wit::grafbase::sdk;

                sdk::types::add_to_linker(&mut linker, |state| state)?;
                let instance_pre = linker.instantiate_pre(&component)?;
                SdkPre::Since0_8_0(since_0_8_0::wit::SdkPre::<WasiState>::new(instance_pre)?)
            }
            v if v < &Version::new(0, 10, 0) => {
                since_0_9_0::wit::Sdk::add_to_linker(&mut linker, |state| state)?;
                let instance_pre = linker.instantiate_pre(&component)?;
                SdkPre::Since0_9_0(since_0_9_0::wit::SdkPre::<WasiState>::new(instance_pre)?)
            }
            _ => {
                since_0_10_0::Sdk::add_to_linker(&mut linker, |state| state)?;
                let instance_pre = linker.instantiate_pre(&component)?;
                SdkPre::Since0_10_0(since_0_10_0::SdkPre::<WasiState>::new(instance_pre)?)
            }
        })
    }

    pub(crate) async fn instantiate(
        &self,
        state: WasiState,
        schema_directives: &[wit::SchemaDirective<'static>],
        guest_config: &[u8],
    ) -> crate::Result<Box<dyn ExtensionInstance>> {
        match self {
            SdkPre::Since0_8_0(sdk_pre) => {
                let mut store = Store::new(sdk_pre.engine(), state);

                let inner = sdk_pre.instantiate_async(&mut store).await?;
                inner.call_register_extension(&mut store).await?;

                inner
                    .grafbase_sdk_extension()
                    .call_init_gateway_extension(&mut store, schema_directives, guest_config)
                    .await??;

                let instance = ExtensionInstanceSince080 {
                    store,
                    inner,
                    poisoned: false,
                };

                Ok(Box::new(instance))
            }
            SdkPre::Since0_9_0(sdk_pre) => {
                let mut store = Store::new(sdk_pre.engine(), state);

                let inner = sdk_pre.instantiate_async(&mut store).await?;
                inner.call_register_extension(&mut store).await?;

                inner
                    .grafbase_sdk_init()
                    .call_init_gateway_extension(&mut store, schema_directives, guest_config)
                    .await??;

                let instance = ExtensionInstanceSince090 {
                    store,
                    inner,
                    poisoned: false,
                };

                Ok(Box::new(instance))
            }
            SdkPre::Since0_10_0(sdk_pre) => {
                let mut store = Store::new(sdk_pre.engine(), state);

                let inner = sdk_pre.instantiate_async(&mut store).await?;
                inner.call_register_extension(&mut store).await?;

                inner
                    .grafbase_sdk_init()
                    .call_init_gateway_extension(&mut store, schema_directives, guest_config)
                    .await??;

                let instance = ExtensionInstanceSince0_10_0 {
                    store,
                    inner,
                    poisoned: false,
                };

                Ok(Box::new(instance))
            }
        }
    }
}
