use wasmtime::component::{ComponentType, Lift, Lower, Resource};

use crate::{resources::Headers, state::InstanceState};

impl Host for InstanceState {}

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

// Typical Wasmtime bindgen! macro generated stuff
pub trait Host: Send + ::core::marker::Send {}
impl<_T: Host + ?Sized + Send> Host for &mut _T {}
pub fn add_to_linker<T, D>(
    _linker: &mut wasmtime::component::Linker<T>,
    _host_getter: fn(&mut T) -> D::Data<'_>,
) -> wasmtime::Result<()>
where
    D: wasmtime::component::HasData,
    for<'a> D::Data<'a>: Host,
    T: 'static + Send,
{
    Ok(())
}
