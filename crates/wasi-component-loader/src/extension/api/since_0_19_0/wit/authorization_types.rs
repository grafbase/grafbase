use wasmtime::component::{ComponentType, Lift, Resource};

pub use crate::extension::api::since_0_17_0::wit::authorization_types::*;
use crate::extension::api::since_0_19_0::world::Headers;

#[derive(ComponentType, Lift)]
#[component(record)]
pub struct AuthorizationOutput {
    pub decisions: AuthorizationDecisions,
    pub state: Vec<u8>,
    pub headers: Resource<Headers>,
}
