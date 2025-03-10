use futures::future::BoxFuture;

use crate::{Error, ErrorResponse};

use super::api::{
    instance::InputList,
    wit::{
        authorization::{AuthorizationDecisions, ResponseElements},
        context::AuthorizationContext,
        directive::{FieldDefinitionDirective, QueryElements},
        resolver::FieldOutput,
        token::Token,
    },
};

pub trait ExtensionInstance {
    fn recycle(&mut self) -> Result<(), Error>;

    fn resolve_field<'a>(
        &'a mut self,
        headers: http::HeaderMap,
        subgraph_name: &'a str,
        directive: FieldDefinitionDirective<'a>,
        inputs: InputList,
    ) -> BoxFuture<'a, Result<FieldOutput, Error>>;

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

    fn resolve_next_subscription_item(&mut self) -> BoxFuture<'_, Result<Option<FieldOutput>, Error>>;

    fn authenticate(
        &mut self,
        headers: http::HeaderMap,
    ) -> BoxFuture<'_, Result<(http::HeaderMap, Token), ErrorResponse>>;

    fn authorize_query<'a>(
        &'a mut self,
        ctx: AuthorizationContext,
        elements: QueryElements<'a>,
    ) -> BoxFuture<'a, Result<(AuthorizationDecisions, Vec<u8>), ErrorResponse>>;

    fn authorize_response<'a>(
        &'a mut self,
        ctx: AuthorizationContext,
        state: &'a [u8],
        elements: ResponseElements<'a>,
    ) -> BoxFuture<'a, Result<AuthorizationDecisions, Error>>;
}
