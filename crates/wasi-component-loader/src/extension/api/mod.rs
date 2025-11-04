pub(crate) mod since_0_10_0;
pub(crate) mod since_0_14_0;
pub(crate) mod since_0_15_0;
pub(crate) mod since_0_16_0;
pub(crate) mod since_0_17_0;
pub(crate) mod since_0_18_0;
pub(crate) mod since_0_19_0;
pub(crate) mod since_0_21_0;
pub(crate) mod since_0_23_0;

use std::sync::Arc;

use engine_schema::Schema;
use since_0_10_0::SdkPre0_10_0;
use since_0_14_0::SdkPre0_14_0;
use since_0_15_0::SdkPre0_15_0;
use since_0_16_0::SdkPre0_16_0;
use since_0_17_0::SdkPre0_17_0;
use since_0_18_0::SdkPre0_18_0;
use since_0_19_0::SdkPre0_19_0;
use since_0_21_0::SdkPre0_21_0;
use since_0_23_0::SdkPre0_23_0;
pub use since_0_23_0::wit;

use super::{ExtensionConfig, ExtensionInstance};
use crate::InstanceState;
use wasmtime::component::{Component, Linker};

pub(crate) enum SdkPre {
    Since0_10_0(SdkPre0_10_0),
    Since0_14_0(SdkPre0_14_0),
    Since0_15_0(SdkPre0_15_0),
    Since0_16_0(SdkPre0_16_0),
    Since0_17_0(SdkPre0_17_0),
    Since0_18_0(SdkPre0_18_0),
    Since0_19_0(SdkPre0_19_0),
    Since0_21_0(SdkPre0_21_0),
    Since0_23_0(SdkPre0_23_0),
}

impl SdkPre {
    pub(crate) fn new<T: serde::Serialize>(
        schema: Arc<Schema>,
        config: &ExtensionConfig<T>,
        component: Component,
        linker: Linker<InstanceState>,
    ) -> wasmtime::Result<SdkPre> {
        Ok(match (config.sdk_version.major, config.sdk_version.minor) {
            (0, 10..=13) => SdkPre::Since0_10_0(SdkPre0_10_0::new(schema, config, component, linker)?),
            (0, 14) => SdkPre::Since0_14_0(SdkPre0_14_0::new(schema, config, component, linker)?),
            (0, 15) => SdkPre::Since0_15_0(SdkPre0_15_0::new(schema, config, component, linker)?),
            (0, 16) => SdkPre::Since0_16_0(SdkPre0_16_0::new(schema, config, component, linker)?),
            (0, 17) => SdkPre::Since0_17_0(SdkPre0_17_0::new(schema, config, component, linker)?),
            (0, 18) => SdkPre::Since0_18_0(SdkPre0_18_0::new(schema, config, component, linker)?),
            (0, 19..=20) => SdkPre::Since0_19_0(SdkPre0_19_0::new(schema, config, component, linker)?),
            (0, 21..=22) => SdkPre::Since0_21_0(SdkPre0_21_0::new(schema, config, component, linker)?),
            (0, 23..) => SdkPre::Since0_23_0(SdkPre0_23_0::new(schema, config, component, linker)?),
            (major, minor) => unimplemented!("SDK version {major}.{minor} is not supported",),
        })
    }

    pub(crate) async fn instantiate(&self, state: InstanceState) -> wasmtime::Result<Box<dyn ExtensionInstance>> {
        match self {
            SdkPre::Since0_10_0(sdk_pre) => sdk_pre.instantiate(state).await,
            SdkPre::Since0_14_0(sdk_pre) => sdk_pre.instantiate(state).await,
            SdkPre::Since0_15_0(sdk_pre) => sdk_pre.instantiate(state).await,
            SdkPre::Since0_16_0(sdk_pre) => sdk_pre.instantiate(state).await,
            SdkPre::Since0_17_0(sdk_pre) => sdk_pre.instantiate(state).await,
            SdkPre::Since0_18_0(sdk_pre) => sdk_pre.instantiate(state).await,
            SdkPre::Since0_19_0(sdk_pre) => sdk_pre.instantiate(state).await,
            SdkPre::Since0_21_0(sdk_pre) => sdk_pre.instantiate(state).await,
            SdkPre::Since0_23_0(sdk_pre) => sdk_pre.instantiate(state).await,
        }
    }
}
