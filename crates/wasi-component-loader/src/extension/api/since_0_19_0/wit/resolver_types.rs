use engine_error::ErrorCode;
use wasmtime::component::{ComponentType, Lift};

pub use crate::extension::api::{
    since_0_14_0::wit::{resolver_types::Data, selection_set_resolver_types::*},
    wit::Error,
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

impl From<crate::extension::api::since_0_17_0::wit::resolver_types::Response> for Response {
    fn from(response: crate::extension::api::since_0_17_0::wit::resolver_types::Response) -> Self {
        Self {
            data: response.data,
            errors: response.errors.into_iter().collect(),
        }
    }
}

impl From<crate::extension::api::since_0_18_0::wit::resolver_types::Response> for Response {
    fn from(response: crate::extension::api::since_0_18_0::wit::resolver_types::Response) -> Self {
        Self {
            data: response.data,
            errors: response.errors,
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

impl From<crate::extension::api::since_0_17_0::wit::resolver_types::SubscriptionItem> for SubscriptionItem {
    fn from(value: crate::extension::api::since_0_17_0::wit::resolver_types::SubscriptionItem) -> Self {
        match value {
            crate::extension::api::since_0_17_0::world::SubscriptionItem::Single(response) => {
                Self::Single(response.into())
            }
            crate::extension::api::since_0_17_0::world::SubscriptionItem::Multiple(responses) => {
                Self::Multiple(responses.into_iter().map(Into::into).collect())
            }
        }
    }
}

impl From<crate::extension::api::since_0_18_0::wit::resolver_types::SubscriptionItem> for SubscriptionItem {
    fn from(value: crate::extension::api::since_0_18_0::wit::resolver_types::SubscriptionItem) -> Self {
        match value {
            crate::extension::api::since_0_18_0::wit::resolver_types::SubscriptionItem::Single(response) => {
                Self::Single(response.into())
            }
            crate::extension::api::since_0_18_0::wit::resolver_types::SubscriptionItem::Multiple(responses) => {
                Self::Multiple(responses.into_iter().map(Into::into).collect())
            }
        }
    }
}
