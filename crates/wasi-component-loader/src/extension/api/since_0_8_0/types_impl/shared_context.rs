use wasmtime::component::Resource;

use super::super::wit::grafbase::sdk::types;
use crate::{SharedContext, WasiState};

impl types::HostSharedContext for WasiState {
    async fn get(&mut self, self_: Resource<SharedContext>, name: String) -> wasmtime::Result<Option<String>> {
        let ctx = WasiState::get(self, &self_)?;
        Ok(ctx.kv.get(&name).cloned())
    }

    async fn trace_id(&mut self, self_: Resource<SharedContext>) -> wasmtime::Result<String> {
        let ctx = WasiState::get(self, &self_)?;
        Ok(ctx.trace_id.to_string())
    }

    async fn drop(&mut self, rep: Resource<SharedContext>) -> wasmtime::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}
