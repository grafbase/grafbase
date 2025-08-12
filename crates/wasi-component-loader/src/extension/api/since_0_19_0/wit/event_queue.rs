use wasmtime::component::Resource;

use crate::{InstanceState, resources::EventQueueResource};

pub use super::grafbase::sdk::event_queue::*;

impl Host for InstanceState {}

impl HostEventQueue for InstanceState {
    async fn pop(&mut self, self_: Resource<EventQueueResource>) -> wasmtime::Result<Option<Event>> {
        let this = self.resources.get(&self_)?;

        match this.0.pop() {
            Some(event) => Ok(Some(super::event_types::convert_event(self, event)?)),
            None => Ok(None),
        }
    }

    async fn drop(&mut self, res: Resource<EventQueueResource>) -> wasmtime::Result<()> {
        self.resources.delete(res)?;

        Ok(())
    }
}
