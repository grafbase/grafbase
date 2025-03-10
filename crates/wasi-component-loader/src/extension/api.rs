mod since_0_8_0;
mod since_0_9_0;

use since_0_8_0::instance::ExtensionInstanceSince080;
use since_0_9_0::instance::ExtensionInstanceSince090;
pub use since_0_9_0::*;

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
}

pub(crate) fn add_to_linker(sdk_version: &Version, linker: &mut Linker<WasiState>) -> crate::Result<()> {
    match sdk_version {
        v if v < &Version::new(0, 9, 0) => {
            use since_0_8_0::wit::grafbase::sdk;

            sdk::types::add_to_linker(linker, |state| state)?;
        }
        _ => {
            use since_0_9_0::wit::grafbase::sdk;

            sdk::access_log::add_to_linker(linker, |state| state)?;
            sdk::cache::add_to_linker(linker, |state| state)?;
            sdk::context::add_to_linker(linker, |state| state)?;
            sdk::error::add_to_linker(linker, |state| state)?;
            sdk::headers::add_to_linker(linker, |state| state)?;
            sdk::http_client::add_to_linker(linker, |state| state)?;
            sdk::nats_client::add_to_linker(linker, |state| state)?;
        }
    }

    Ok(())
}

pub(crate) fn intialize_sdk_pre(
    sdk_version: &Version,
    component: &Component,
    linker: &Linker<WasiState>,
) -> crate::Result<SdkPre> {
    let instance_pre = linker.instantiate_pre(component)?;

    let pre = match sdk_version {
        v if v < &Version::new(0, 9, 0) => {
            SdkPre::Since0_8_0(since_0_8_0::wit::SdkPre::<WasiState>::new(instance_pre)?)
        }
        _ => SdkPre::Since0_9_0(since_0_9_0::wit::SdkPre::<WasiState>::new(instance_pre)?),
    };

    Ok(pre)
}

pub(crate) async fn instantiate(
    pre: &SdkPre,
    state: WasiState,
    schema_directives: &[wit::directive::SchemaDirective<'static>],
    guest_config: &[u8],
) -> crate::Result<Box<dyn ExtensionInstance + Send + 'static>> {
    match pre {
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
    }
}
