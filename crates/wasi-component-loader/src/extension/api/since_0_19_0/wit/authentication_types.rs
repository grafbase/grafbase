use wasmtime::component::{ComponentType, Lift, Lower, Resource};

use crate::resources::Headers;

// We shouldn't need to generate this one, but somehow wasmtime generates a weird lifetime.
#[derive(ComponentType, Lift, Lower)]
#[component(record)]
pub struct PublicMetadataEndpoint {
    #[component(name = "path")]
    pub path: String,
    #[component(name = "response-body")]
    pub response_body: Vec<u8>,
    #[component(name = "response-headers")]
    pub response_headers: Resource<Headers>,
}

// For Wasmtime bindgen, does nothing.
pub trait Host {}
impl Host for crate::InstanceState {}

pub fn add_to_linker<T, U>(
    _linker: &mut wasmtime::component::Linker<T>,
    _get: impl Fn(&mut T) -> &mut U + Send + Sync + Copy + 'static,
) -> wasmtime::Result<()>
where
    U: Host + Send,
    T: Send,
{
    Ok(())
}
