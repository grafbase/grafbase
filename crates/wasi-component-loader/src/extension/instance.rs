use std::sync::Arc;

use engine::{GraphqlError, RequestContext};
use futures::future::BoxFuture;
use runtime::extension::{AuthorizationDecisions, Data, Token};

use crate::{Error, ErrorResponse};

use super::api::wit::{FieldDefinitionDirective, QueryElements, ResponseElements};

/// List of inputs to be provided to the extension.
/// The data itself is fully custom and thus will be serialized with serde to cross the Wasm
/// boundary.
#[derive(Default)]
pub struct InputList(pub(crate) Vec<Vec<u8>>);

impl<S: serde::Serialize> FromIterator<S> for InputList {
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        Self(
            iter.into_iter()
                .map(|input| crate::cbor::to_vec(&input).unwrap())
                .collect(),
        )
    }
}

pub type SubscriptionItem = Vec<Result<Data, GraphqlError>>;

pub trait ExtensionInstance: Send + 'static {
    fn recycle(&mut self) -> Result<(), Error>;

    fn resolve_field<'a>(
        &'a mut self,
        headers: http::HeaderMap,
        subgraph_name: &'a str,
        directive: FieldDefinitionDirective<'a>,
        inputs: InputList,
    ) -> BoxFuture<'a, Result<Vec<Result<Data, GraphqlError>>, Error>>;

    #[allow(clippy::type_complexity)]
    fn subscription_key<'a>(
        &'a mut self,
        headers: http::HeaderMap,
        subgraph_name: &'a str,
        directive: FieldDefinitionDirective<'a>,
    ) -> BoxFuture<'a, Result<(http::HeaderMap, Option<Vec<u8>>), Error>>;

    fn resolve_subscription<'a>(
        &'a mut self,
        headers: http::HeaderMap,
        subgraph_name: &'a str,
        directive: FieldDefinitionDirective<'a>,
    ) -> BoxFuture<'a, Result<(), Error>>;

    fn resolve_next_subscription_item(&mut self) -> BoxFuture<'_, Result<Option<SubscriptionItem>, Error>>;

    fn authenticate(
        &mut self,
        headers: http::HeaderMap,
    ) -> BoxFuture<'_, Result<(http::HeaderMap, Token), ErrorResponse>>;

    fn authorize_query<'a>(
        &'a mut self,
        ctx: &'a Arc<RequestContext>,
        elements: QueryElements<'a>,
    ) -> BoxFuture<'a, Result<(AuthorizationDecisions, Vec<u8>), ErrorResponse>>;

    fn authorize_response<'a>(
        &'a mut self,
        state: &'a [u8],
        elements: ResponseElements<'a>,
    ) -> BoxFuture<'a, Result<AuthorizationDecisions, Error>>;
}
