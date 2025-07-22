use wasmtime::component::Resource;

use crate::InstanceState;

pub use super::grafbase::sdk::access_log::*;

impl Host for InstanceState {}

impl HostAccessLog for InstanceState {
    async fn send(&mut self, _: Vec<u8>) -> wasmtime::Result<Result<(), LogError>> {
        todo!("sorry, we do not have this anymore.")
    }

    async fn drop(&mut self, _: Resource<()>) -> wasmtime::Result<()> {
        // Singleton that is never allocated
        Ok(())
    }
}
