use wasmtime::component::Resource;

pub use super::grafbase::sdk::headers::*;
use crate::state::InstanceState;

impl Host for InstanceState {}

impl HostHeaders for InstanceState {
    async fn get(&mut self, self_: Resource<Headers>, name: String) -> wasmtime::Result<Vec<Vec<u8>>> {
        let headers = self.resources.get(&self_)?;
        Ok(headers.get(&name).await)
    }

    async fn has(&mut self, self_: Resource<Headers>, name: String) -> wasmtime::Result<bool> {
        let headers = self.resources.get(&self_)?;
        Ok(headers.has(&name).await)
    }

    async fn set(
        &mut self,
        self_: Resource<Headers>,
        name: String,
        value: Vec<Vec<u8>>,
    ) -> wasmtime::Result<Result<(), HeaderError>> {
        let headers = self.resources.get_mut(&self_)?;
        Ok(headers.set(name, value).await)
    }

    async fn delete(&mut self, self_: Resource<Headers>, name: String) -> wasmtime::Result<Result<(), HeaderError>> {
        let headers = self.resources.get_mut(&self_)?;
        Ok(headers.delete(&name).await)
    }

    async fn get_and_delete(
        &mut self,
        self_: Resource<Headers>,
        name: String,
    ) -> wasmtime::Result<Result<Vec<Vec<u8>>, HeaderError>> {
        let headers = self.resources.get_mut(&self_)?;
        Ok(headers.get_and_delete(&name).await)
    }

    async fn append(
        &mut self,
        self_: Resource<Headers>,
        name: String,
        value: Vec<u8>,
    ) -> wasmtime::Result<Result<(), HeaderError>> {
        let headers = self.resources.get_mut(&self_)?;
        Ok(headers.append(name, value).await)
    }

    async fn entries(&mut self, self_: Resource<Headers>) -> wasmtime::Result<Vec<(String, Vec<u8>)>> {
        let headers = self.resources.get(&self_)?;
        Ok(headers.entries().await)
    }

    async fn drop(&mut self, rep: Resource<Headers>) -> wasmtime::Result<()> {
        self.resources.delete(rep)?;
        Ok(())
    }

    async fn new(&mut self) -> wasmtime::Result<Resource<Headers>> {
        let headers = self.resources.push(Headers::from(http::HeaderMap::default()))?;
        Ok(headers)
    }
}
