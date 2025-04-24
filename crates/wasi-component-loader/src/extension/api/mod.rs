mod since_0_10_0;
mod since_0_14_0;
mod since_0_15_0;
mod since_0_8_0;
mod since_0_9_0;

use std::sync::Arc;

use engine_schema::Schema;
use since_0_8_0::instance::SdkPre080;
use since_0_9_0::instance::SdkPre090;
use since_0_10_0::SdkPre0_10_0;
use since_0_14_0::SdkPre0_14_0;
use since_0_15_0::SdkPre0_15_0;
pub use since_0_15_0::world as wit;

use super::{ExtensionConfig, ExtensionInstance};
use crate::WasiState;
use semver::Version;
use wasmtime::component::{Component, Linker};

pub(crate) enum SdkPre {
    Since0_8_0(SdkPre080),
    Since0_9_0(SdkPre090),
    Since0_10_0(SdkPre0_10_0),
    Since0_14_0(SdkPre0_14_0),
    Since0_15_0(SdkPre0_15_0),
}

impl SdkPre {
    pub(crate) fn new<T: serde::Serialize>(
        schema: Arc<Schema>,
        config: &ExtensionConfig<T>,
        component: Component,
        linker: Linker<WasiState>,
    ) -> crate::Result<SdkPre> {
        Ok(match &config.sdk_version {
            v if v < &Version::new(0, 9, 0) => SdkPre::Since0_8_0(SdkPre080::new(schema, config, component, linker)?),
            v if v < &Version::new(0, 10, 0) => SdkPre::Since0_9_0(SdkPre090::new(schema, config, component, linker)?),
            v if v < &Version::new(0, 14, 0) => {
                SdkPre::Since0_10_0(SdkPre0_10_0::new(schema, config, component, linker)?)
            }
            v if v < &Version::new(0, 15, 0) => {
                SdkPre::Since0_14_0(SdkPre0_14_0::new(schema, config, component, linker)?)
            }
            _ => SdkPre::Since0_15_0(SdkPre0_15_0::new(schema, config, component, linker)?),
        })
    }

    pub(crate) async fn instantiate(&self, state: WasiState) -> crate::Result<Box<dyn ExtensionInstance>> {
        match self {
            SdkPre::Since0_8_0(sdk_pre) => sdk_pre.instantiate(state).await,
            SdkPre::Since0_9_0(sdk_pre) => sdk_pre.instantiate(state).await,
            SdkPre::Since0_10_0(sdk_pre) => sdk_pre.instantiate(state).await,
            SdkPre::Since0_14_0(sdk_pre) => sdk_pre.instantiate(state).await,
            SdkPre::Since0_15_0(sdk_pre) => sdk_pre.instantiate(state).await,
        }
    }
}
