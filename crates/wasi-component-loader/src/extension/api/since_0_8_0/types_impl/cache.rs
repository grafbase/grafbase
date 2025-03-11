use wasmtime::component::Resource;

use crate::WasiState;

use super::super::wit::grafbase::sdk::types;

impl types::HostCache for WasiState {
    async fn get(&mut self, key: String) -> wasmtime::Result<Option<Vec<u8>>> {
        Ok(self.cache().get(&key).await)
    }

    async fn set(&mut self, key: String, value: Vec<u8>, ttl_ms: Option<u64>) -> wasmtime::Result<()> {
        self.cache().set(&key, value, ttl_ms).await;
        Ok(())
    }

    async fn drop(&mut self, _: Resource<types::Cache>) -> wasmtime::Result<()> {
        // Singleton that is never allocated
        Ok(())
    }
}
