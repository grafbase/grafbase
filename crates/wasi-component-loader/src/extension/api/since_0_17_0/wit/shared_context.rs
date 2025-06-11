use wasmtime::component::Resource;

use crate::WasiState;

pub use super::grafbase::sdk::shared_context::*;

impl Host for WasiState {}

impl HostSharedContext for WasiState {
    async fn trace_id(&mut self, self_: Resource<SharedContext>) -> wasmtime::Result<String> {
        let context = self.get(&self_)?;
        Ok(context.trace_id.to_string())
    }

    async fn event_queue(&mut self, self_: Resource<SharedContext>) -> wasmtime::Result<Resource<EventQueue>> {
        let context = self.get(&self_)?;
        let event_queue = self.table.push(context.event_queue.clone())?;

        Ok(event_queue)
    }

    async fn drop(&mut self, rep: Resource<SharedContext>) -> wasmtime::Result<()> {
        self.table.delete(rep)?;

        Ok(())
    }
}
