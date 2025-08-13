use engine_error::ErrorCode;
use wasmtime::component::{ComponentType, Lift};

pub use crate::extension::api::{
    since_0_14_0::wit::{resolver_types::Data, selection_set_resolver_types::*},
    since_0_14_0::world::Error,
};

#[derive(Clone, Debug, ComponentType, Lift)]
#[component(record)]
pub struct Response {
    pub data: Option<Data>,
    pub errors: Vec<Error>,
}

impl From<Response> for runtime::extension::Response {
    fn from(response: Response) -> Self {
        runtime::extension::Response {
            data: response.data.map(Into::into),
            errors: response
                .errors
                .into_iter()
                .map(|err| err.into_graphql_error(ErrorCode::ExtensionError))
                .collect(),
        }
    }
}

#[derive(Clone, Debug, ComponentType, Lift)]
#[component(variant)]
pub enum SubscriptionItem {
    #[component(name = "single")]
    Single(Response),
    #[component(name = "multiple")]
    Multiple(Vec<Response>),
}
