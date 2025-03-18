use engine::GraphqlError;
use futures::future::BoxFuture;
use runtime::extension::{AuthorizationDecisions, Data, Token, TokenRef};

use crate::{
    Error, ErrorResponse,
    extension::api::wit::{FieldDefinitionDirective, QueryElements, ResponseElements},
    resources::Lease,
};

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
pub type QueryAuthorizationResult = Result<(Lease<http::HeaderMap>, AuthorizationDecisions, Vec<u8>), ErrorResponse>;

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
        headers: Lease<http::HeaderMap>,
        subgraph_name: &'a str,
        directive: FieldDefinitionDirective<'a>,
    ) -> BoxFuture<'a, Result<(Lease<http::HeaderMap>, Option<Vec<u8>>), Error>>;

    fn resolve_subscription<'a>(
        &'a mut self,
        headers: http::HeaderMap,
        subgraph_name: &'a str,
        directive: FieldDefinitionDirective<'a>,
    ) -> BoxFuture<'a, Result<(), Error>>;

    fn resolve_next_subscription_item(&mut self) -> BoxFuture<'_, Result<Option<SubscriptionItem>, Error>>;

    fn authenticate(
        &mut self,
        headers: Lease<http::HeaderMap>,
    ) -> BoxFuture<'_, Result<(Lease<http::HeaderMap>, Token), ErrorResponse>>;

    fn authorize_query<'a>(
        &'a mut self,
        headers: Lease<http::HeaderMap>,
        token: TokenRef<'a>,
        elements: QueryElements<'a>,
    ) -> BoxFuture<'a, QueryAuthorizationResult>;

    fn authorize_response<'a>(
        &'a mut self,
        state: &'a [u8],
        elements: ResponseElements<'a>,
    ) -> BoxFuture<'a, Result<AuthorizationDecisions, Error>>;
}
