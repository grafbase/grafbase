use wasmtime::component::Resource;

use crate::WasiState;

pub use super::grafbase::sdk::access_log::*;

impl Host for WasiState {}

impl HostAccessLog for WasiState {
    async fn send(&mut self, _: Vec<u8>) -> wasmtime::Result<Result<(), LogError>> {
        todo!("sorry, we do not have this anymore.")
    }

    async fn drop(&mut self, _: Resource<()>) -> wasmtime::Result<()> {
        // Singleton that is never allocated
        Ok(())
    }
}
