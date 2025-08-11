use wasmtime::component::{ComponentType, Lift, Resource};

pub use crate::extension::api::since_0_19_0::wit::authorization_types::{
    AuthorizationDecisions, AuthorizationDecisionsDenySome, Host, QueryElement, QueryElements, ResponseElement,
    ResponseElements, add_to_linker,
};
use crate::resources::Headers;

#[derive(Debug, ComponentType, Lift)]
#[component(record)]
pub struct AuthorizationOutput {
    #[component(name = "decisions")]
    pub decisions: AuthorizationDecisions,
    #[component(name = "state")]
    pub state: Vec<u8>,
    #[component(name = "subgraph-headers")]
    pub subgraph_headers: Resource<Headers>,
    #[component(name = "additional-headers")]
    pub additional_headers: Option<Resource<Headers>>,
}
