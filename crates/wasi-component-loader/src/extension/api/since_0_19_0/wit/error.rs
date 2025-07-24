use wasmtime::component::{ComponentType, Lift, Resource};

pub use crate::extension::api::since_0_17_0::wit::error::{Error, Host, add_to_linker};
use crate::resources::Headers;

#[derive(ComponentType, Lift)]
#[component(record)]
pub struct ErrorResponse {
    #[component(name = "status-code")]
    pub status_code: u16,
    pub errors: Vec<Error>,
    pub headers: Option<Resource<Headers>>,
}
