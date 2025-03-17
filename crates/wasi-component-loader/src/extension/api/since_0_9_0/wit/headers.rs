use wasmtime::component::Resource;

use crate::WasiState;

pub use super::grafbase::sdk::headers::*;

impl Host for WasiState {}

impl HostHeaders for WasiState {
    async fn get(&mut self, self_: Resource<Headers>, name: String) -> wasmtime::Result<Option<String>> {
        let headers = WasiState::get(self, &self_)?;

        let value = headers
            .get(&name)
            .await
            .into_iter()
            .next()
            .map(|val| String::from_utf8_lossy(&val).into_owned());

        Ok(value)
    }

    async fn entries(&mut self, self_: Resource<Headers>) -> wasmtime::Result<Vec<(String, String)>> {
        let headers = WasiState::get(self, &self_)?;

        let entries = headers
            .entries()
            .await
            .into_iter()
            .map(|(name, value)| (name, String::from_utf8_lossy(&value).into_owned()))
            .collect();

        Ok(entries)
    }

    async fn drop(&mut self, rep: Resource<Headers>) -> wasmtime::Result<()> {
        if WasiState::get(self, &rep)?.is_owned() {
            self.table.delete(rep)?;
        }

        Ok(())
    }
}
