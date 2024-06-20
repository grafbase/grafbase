use wasmtime::component::Resource;
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiView};
use wasmtime_wasi_http::{WasiHttpCtx, WasiHttpView};

pub(crate) struct WasiState {
    ctx: WasiCtx,
    http_ctx: WasiHttpCtx,
    table: ResourceTable,
}

impl WasiState {
    pub fn new(ctx: WasiCtx) -> Self {
        Self {
            ctx,
            http_ctx: WasiHttpCtx::new(),
            table: ResourceTable::new(),
        }
    }

    /// Add a resource to the shared memory.
    pub fn push_resource<T: Send + 'static>(&mut self, entry: T) -> crate::Result<Resource<T>> {
        Ok(self.table.push(entry).map_err(anyhow::Error::from)?)
    }

    /// Takes a resource back from the shared memory.
    pub fn take_resource<T: 'static>(&mut self, rep: u32) -> crate::Result<T> {
        let resource = self
            .table
            .delete(Resource::<T>::new_own(rep))
            .map_err(anyhow::Error::from)?;

        Ok(resource)
    }

    /// Gets a mutable reference to the given resource.
    pub fn get_mut<T: 'static>(&mut self, resource: &Resource<T>) -> crate::Result<&mut T> {
        let entry = self.table.get_mut(resource).map_err(anyhow::Error::from)?;

        Ok(entry)
    }

    /// Gets an immutable reference to the given resource.
    pub fn get<T: 'static>(&self, resource: &Resource<T>) -> crate::Result<&T> {
        let entry = self.table.get(resource).map_err(anyhow::Error::from)?;

        Ok(entry)
    }
}

impl WasiView for WasiState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
}

impl WasiHttpView for WasiState {
    fn ctx(&mut self) -> &mut WasiHttpCtx {
        &mut self.http_ctx
    }

    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}
