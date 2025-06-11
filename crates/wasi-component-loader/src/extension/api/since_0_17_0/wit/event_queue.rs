use wasmtime::component::Resource;

use crate::WasiState;

pub use super::grafbase::sdk::event_queue::*;

impl Host for WasiState {}

impl HostEventQueue for WasiState {
    async fn push(&mut self, _: Resource<EventQueue>, _: String, _: Vec<u8>) -> wasmtime::Result<()> {
        todo!("this is not implemented yet in the host")
    }

    async fn pop(&mut self, _: Resource<EventQueue>) -> wasmtime::Result<Option<Event>> {
        todo!("this is not implemented yet in the host")
    }

    async fn drop(&mut self, res: Resource<EventQueue>) -> wasmtime::Result<()> {
        self.table.delete(res)?;

        Ok(())
    }
}
