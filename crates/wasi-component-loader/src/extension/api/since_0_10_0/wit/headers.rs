use wasmtime::component::Resource;

pub use super::grafbase::sdk::headers::*;
use crate::{extension::api::wit, state::InstanceState};

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
        Ok(headers.set(name, value).await.map_err(Into::into))
    }

    async fn delete(&mut self, self_: Resource<Headers>, name: String) -> wasmtime::Result<Result<(), HeaderError>> {
        let headers = self.resources.get_mut(&self_)?;
        Ok(headers.delete(&name).await.map_err(Into::into))
    }

    async fn get_and_delete(
        &mut self,
        self_: Resource<Headers>,
        name: String,
    ) -> wasmtime::Result<Result<Vec<Vec<u8>>, HeaderError>> {
        let headers = self.resources.get_mut(&self_)?;
        Ok(headers.get_and_delete(&name).await.map_err(Into::into))
    }

    async fn append(
        &mut self,
        self_: Resource<Headers>,
        name: String,
        value: Vec<u8>,
    ) -> wasmtime::Result<Result<(), HeaderError>> {
        let headers = self.resources.get_mut(&self_)?;
        Ok(headers.append(name, value).await.map_err(Into::into))
    }

    async fn entries(&mut self, self_: Resource<Headers>) -> wasmtime::Result<Vec<(String, Vec<u8>)>> {
        let headers = self.resources.get(&self_)?;
        Ok(headers.entries().await)
    }

    async fn drop(&mut self, rep: Resource<Headers>) -> wasmtime::Result<()> {
        if self.resources.get(&rep)?.is_owned() {
            self.resources.delete(rep)?;
        }
        Ok(())
    }
}

impl From<wit::HeaderError> for HeaderError {
    fn from(err: wit::HeaderError) -> Self {
        match err {
            wit::HeaderError::Immutable => HeaderError::Immutable,
            wit::HeaderError::InvalidSyntax => HeaderError::InvalidSyntax,
            wit::HeaderError::Forbidden => HeaderError::Forbidden,
        }
    }
}
