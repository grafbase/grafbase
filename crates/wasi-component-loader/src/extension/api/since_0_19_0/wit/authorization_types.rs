use wasmtime::component::{ComponentType, Lift, Lower, Resource};

pub use crate::extension::api::since_0_17_0::wit::authorization_types::{
    self as wit17, AuthorizationDecisions, AuthorizationDecisionsDenySome, Host, ResponseElement, ResponseElements,
    add_to_linker,
};
use crate::extension::api::{since_0_19_0::world::Headers, wit::DirectiveSite};

#[derive(ComponentType, Lift)]
#[component(record)]
pub struct AuthorizationOutput {
    pub decisions: AuthorizationDecisions,
    pub state: Vec<u8>,
    pub headers: Resource<Headers>,
}

#[derive(Clone, Debug, ComponentType, Lower)]
#[component(record)]
pub struct QueryElement<'a> {
    pub id: u32,
    pub site: DirectiveSite<'a>,
    pub arguments: Vec<u8>,
    #[component(name = "subgraph-name")]
    pub subgraph_name: Option<&'a str>,
}

#[derive(Debug, ComponentType, Lower)]
#[component(record)]
pub struct QueryElements<'a> {
    #[component(name = "directive-names")]
    pub directive_names: &'a [(&'a str, u32, u32)],
    pub elements: &'a [QueryElement<'a>],
}

impl<'a> From<&'a QueryElement<'a>> for wit17::QueryElement<'a> {
    fn from(element: &'a QueryElement<'a>) -> Self {
        wit17::QueryElement {
            id: element.id,
            site: &element.site,
            arguments: element.arguments.as_ref(),
        }
    }
}

impl<'a> From<QueryElements<'a>> for wit17::QueryElements<'a> {
    fn from(elements: QueryElements<'a>) -> Self {
        wit17::QueryElements {
            directive_names: elements.directive_names,
            elements: elements.elements.iter().map(Into::into).collect(),
        }
    }
}
