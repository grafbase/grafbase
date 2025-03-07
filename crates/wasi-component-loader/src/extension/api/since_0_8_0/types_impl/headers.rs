use wasmtime::component::Resource;

use super::super::wit::grafbase::sdk::types;
use crate::{WasiState, headers::Headers};

impl types::HostHeaders for WasiState {
    async fn get(&mut self, self_: Resource<Headers>, name: String) -> wasmtime::Result<Option<String>> {
        let headers = WasiState::get_ref(self, &self_)?;

        let value = headers
            .get(&name)
            .map(|val| String::from_utf8_lossy(val.as_bytes()).into_owned());

        Ok(value)
    }

    async fn entries(&mut self, self_: Resource<Headers>) -> wasmtime::Result<Vec<(String, String)>> {
        let headers = WasiState::get_ref(self, &self_)?;

        let entries = headers
            .iter()
            .map(|(name, value)| (name.to_string(), String::from_utf8_lossy(value.as_bytes()).into_owned()))
            .collect();

        Ok(entries)
    }

    async fn drop(&mut self, rep: Resource<Headers>) -> wasmtime::Result<()> {
        if !WasiState::get(self, &rep)?.is_host_borrowed() {
            self.table.delete(rep)?;
        }

        Ok(())
    }
}
