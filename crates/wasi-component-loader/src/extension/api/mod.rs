mod since_0_10_0;
mod since_0_14_0;
mod since_0_15_0;
mod since_0_16_0;
mod since_0_17_0;
mod since_0_18_0;
mod since_0_19_0;
mod since_0_9_0;

use std::sync::Arc;

use engine_schema::Schema;
use since_0_9_0::instance::SdkPre090;
use since_0_10_0::SdkPre0_10_0;
use since_0_14_0::SdkPre0_14_0;
use since_0_15_0::SdkPre0_15_0;
use since_0_16_0::SdkPre0_16_0;
use since_0_17_0::SdkPre0_17_0;
use since_0_18_0::SdkPre0_18_0;
use since_0_19_0::SdkPre0_19_0;
pub use since_0_19_0::world as wit;

use super::{ExtensionConfig, ExtensionInstance};
use crate::WasiState;
use semver::Version;
use wasmtime::component::{Component, Linker};

pub(crate) enum SdkPre {
    Since0_9_0(SdkPre090),
    Since0_10_0(SdkPre0_10_0),
    Since0_14_0(SdkPre0_14_0),
    Since0_15_0(SdkPre0_15_0),
    Since0_16_0(SdkPre0_16_0),
    Since0_17_0(SdkPre0_17_0),
    Since0_18_0(SdkPre0_18_0),
    Since0_19_0(SdkPre0_19_0),
}

impl SdkPre {
    pub(crate) fn new<T: serde::Serialize>(
        schema: Arc<Schema>,
        config: &ExtensionConfig<T>,
        component: Component,
        linker: Linker<WasiState>,
    ) -> crate::Result<SdkPre> {
        Ok(match &config.sdk_version {
            v if v < &Version::new(0, 9, 0) => unimplemented!("nobody should use the pre version of SDK at this point"),
            v if v < &Version::new(0, 10, 0) => SdkPre::Since0_9_0(SdkPre090::new(schema, config, component, linker)?),
            v if v < &Version::new(0, 14, 0) => {
                SdkPre::Since0_10_0(SdkPre0_10_0::new(schema, config, component, linker)?)
            }
            v if v < &Version::new(0, 15, 0) => {
                SdkPre::Since0_14_0(SdkPre0_14_0::new(schema, config, component, linker)?)
            }
            v if v < &Version::new(0, 16, 0) => {
                SdkPre::Since0_15_0(SdkPre0_15_0::new(schema, config, component, linker)?)
            }
            v if v < &Version::new(0, 17, 0) => {
                SdkPre::Since0_16_0(SdkPre0_16_0::new(schema, config, component, linker)?)
            }
            v if v < &Version::new(0, 18, 0) => {
                SdkPre::Since0_17_0(SdkPre0_17_0::new(schema, config, component, linker)?)
            }
            v if v < &Version::new(0, 19, 0) => {
                SdkPre::Since0_18_0(SdkPre0_18_0::new(schema, config, component, linker)?)
            }
            _ => SdkPre::Since0_19_0(SdkPre0_19_0::new(schema, config, component, linker)?),
        })
    }

    pub(crate) async fn instantiate(&self, state: WasiState) -> crate::Result<Box<dyn ExtensionInstance>> {
        match self {
            SdkPre::Since0_9_0(sdk_pre) => sdk_pre.instantiate(state).await,
            SdkPre::Since0_10_0(sdk_pre) => sdk_pre.instantiate(state).await,
            SdkPre::Since0_14_0(sdk_pre) => sdk_pre.instantiate(state).await,
            SdkPre::Since0_15_0(sdk_pre) => sdk_pre.instantiate(state).await,
            SdkPre::Since0_16_0(sdk_pre) => sdk_pre.instantiate(state).await,
            SdkPre::Since0_17_0(sdk_pre) => sdk_pre.instantiate(state).await,
            SdkPre::Since0_18_0(sdk_pre) => sdk_pre.instantiate(state).await,
            SdkPre::Since0_19_0(sdk_pre) => sdk_pre.instantiate(state).await,
        }
    }
}
