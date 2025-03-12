use wasmtime::component::Resource;

pub use super::grafbase::sdk::headers::*;
use crate::state::WasiState;

impl Host for WasiState {}

impl HostHeaders for WasiState {
    async fn get(&mut self, self_: Resource<Headers>, name: String) -> wasmtime::Result<Vec<Vec<u8>>> {
        let headers = WasiState::get_ref(self, &self_)?;

        let values = headers
            .get_all(&name)
            .into_iter()
            .map(|val| val.as_bytes().to_vec())
            .collect();

        Ok(values)
    }

    async fn has(&mut self, self_: Resource<Headers>, name: String) -> wasmtime::Result<bool> {
        let headers = WasiState::get_ref(self, &self_)?;
        Ok(headers.contains_key(&name))
    }

    async fn set(
        &mut self,
        self_: Resource<Headers>,
        name: String,
        value: Vec<Vec<u8>>,
    ) -> wasmtime::Result<Result<(), HeaderError>> {
        let headers = WasiState::get_ref_mut(self, &self_)?;
        let name: http::HeaderName = name.try_into().map_err(|_| HeaderError::InvalidSyntax)?;
        if value.len() == 1 {
            headers.insert(
                name,
                value
                    .into_iter()
                    .next()
                    .unwrap()
                    .try_into()
                    .map_err(|_| HeaderError::InvalidSyntax)?,
            );
        } else {
            headers.remove(&name);
            for value in value {
                headers.append(name.clone(), value.try_into().map_err(|_| HeaderError::InvalidSyntax)?);
            }
        }
        Ok(Ok(()))
    }

    async fn delete(&mut self, self_: Resource<Headers>, name: String) -> wasmtime::Result<Result<(), HeaderError>> {
        let headers = WasiState::get_ref_mut(self, &self_)?;
        headers.remove(&name);

        Ok(Ok(()))
    }

    async fn get_and_delete(
        &mut self,
        self_: Resource<Headers>,
        name: String,
    ) -> wasmtime::Result<Result<Vec<Vec<u8>>, HeaderError>> {
        let headers = WasiState::get_ref_mut(self, &self_)?;
        let name: http::HeaderName = name.try_into().map_err(|_| HeaderError::InvalidSyntax)?;
        match headers.entry(name) {
            http::header::Entry::Occupied(entry) => {
                let (_, values) = entry.remove_entry_mult();
                Ok(Ok(values.into_iter().map(|val| val.as_bytes().to_vec()).collect()))
            }
            http::header::Entry::Vacant(_) => Ok(Ok(Vec::new())),
        }
    }

    async fn append(
        &mut self,
        self_: Resource<Headers>,
        name: String,
        value: Vec<u8>,
    ) -> wasmtime::Result<Result<(), HeaderError>> {
        let headers = WasiState::get_ref_mut(self, &self_)?;
        let name: http::HeaderName = name.try_into().map_err(|_| HeaderError::InvalidSyntax)?;
        headers.append(name, value.try_into().map_err(|_| HeaderError::InvalidSyntax)?);
        Ok(Ok(()))
    }

    async fn entries(&mut self, self_: Resource<Headers>) -> wasmtime::Result<Vec<(String, Vec<u8>)>> {
        let headers = WasiState::get_ref(self, &self_)?;
        let entries = headers
            .iter()
            .map(|(name, values)| (name.to_string(), values.as_bytes().to_vec()))
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
