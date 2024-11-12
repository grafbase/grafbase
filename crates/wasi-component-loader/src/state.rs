use wasmtime::component::Resource;
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiView};
use wasmtime_wasi_http::{WasiHttpCtx, WasiHttpView};

/// Represents the state of the WASI environment.
///
/// This structure encapsulates the WASI context, HTTP context, and a resource table
/// for managing shared resources in memory. It provides methods to create new instances,
/// manage resources, and access the contexts.
pub(crate) struct WasiState {
    /// The WASI context that contains the state for the WASI environment.
    ctx: WasiCtx,

    /// The WASI HTTP context that handles HTTP-related operations.
    http_ctx: WasiHttpCtx,

    /// The resource table that manages shared resources in memory.
    table: ResourceTable,
}

impl WasiState {
    /// Creates a new instance of `WasiState` with the given WASI context.
    ///
    /// # Arguments
    ///
    /// * `ctx` - A `WasiCtx` instance that represents the WASI environment context.
    ///
    /// # Returns
    ///
    /// A new `WasiState` instance initialized with the provided context and default
    /// HTTP and resource table contexts.
    pub fn new(ctx: WasiCtx) -> Self {
        Self {
            ctx,
            http_ctx: WasiHttpCtx::new(),
            table: ResourceTable::new(),
        }
    }

    /// Pushes a resource into the shared memory, allowing it to be managed by the resource table.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of the resource being pushed.
    ///
    /// # Arguments
    ///
    /// * `entry` - The resource instance to be added to the resource table.
    ///
    /// # Returns
    ///
    /// A result containing the resource holding an instance of `T`, or an error if the operation fails.
    pub fn push_resource<T: Send + 'static>(&mut self, entry: T) -> crate::Result<Resource<T>> {
        Ok(self.table.push(entry).map_err(anyhow::Error::from)?)
    }

    /// Takes ownership of a resource identified by its representation ID from the shared memory.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of the resource being taken.
    ///
    /// # Arguments
    ///
    /// * `rep` - A unique identifier for the resource to be taken.
    ///
    /// # Returns
    ///
    /// A result containing the resource instance of type `T`, or an error if no instance of type `T`
    /// with the given representation ID was found.
    pub fn take_resource<T: 'static>(&mut self, rep: u32) -> crate::Result<T> {
        let resource = self
            .table
            .delete(Resource::<T>::new_own(rep))
            .map_err(anyhow::Error::from)?;

        Ok(resource)
    }

    /// Gets a mutable reference to the instance identified by the given resource.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of the resource being accessed.
    ///
    /// # Arguments
    ///
    /// * `resource` - A reference to the resource instance whose mutable reference is to be retrieved.
    ///
    /// # Returns
    ///
    /// A result containing a mutable reference to the resource of type `T`, or an error if the resource
    /// cannot be accessed.
    pub fn get_mut<T: 'static>(&mut self, resource: &Resource<T>) -> crate::Result<&mut T> {
        let entry = self.table.get_mut(resource).map_err(anyhow::Error::from)?;

        Ok(entry)
    }

    /// Retrieves a reference to the instance identified by the given resource.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of the resource being accessed.
    ///
    /// # Arguments
    ///
    /// * `resource` - A reference to the resource instance whose reference is to be retrieved.
    ///
    /// # Returns
    ///
    /// A result containing a reference to the resource of type `T`, or an error if the resource
    /// cannot be accessed.
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
