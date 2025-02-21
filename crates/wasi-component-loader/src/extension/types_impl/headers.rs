use wasmtime::component::Resource;

use crate::{WasiState, extension::wit::HostHeaders, headers::Headers};

impl HostHeaders for WasiState {
    async fn get(&mut self, self_: Resource<Headers>, name: String) -> wasmtime::Result<Option<String>> {
        let headers = WasiState::get_ref(self, &self_)?;

        let value = headers
            .get(&name)
            .map(|val| String::from_utf8_lossy(val.as_bytes()).into_owned());

        Ok(value)
    }

    async fn drop(&mut self, rep: Resource<Headers>) -> wasmtime::Result<()> {
        if !WasiState::get(self, &rep)?.is_host_borrowed() {
            self.table.delete(rep)?;
        }

        Ok(())
    }
}
