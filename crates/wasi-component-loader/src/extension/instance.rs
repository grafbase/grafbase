use futures::future::BoxFuture;

use super::api::{
    instance::InputList,
    wit::{
        authorization::AuthorizationDecisions,
        context::AuthorizationContext,
        directive::{FieldDefinitionDirective, QueryElements},
        resolver::FieldOutput,
        token::Token,
    },
};

pub trait ExtensionInstance {
    fn recycle(&mut self) -> crate::Result<()>;

    fn resolve_field<'a>(
        &'a mut self,
        headers: http::HeaderMap,
        subgraph_name: &'a str,
        directive: FieldDefinitionDirective<'a>,
        inputs: InputList,
    ) -> BoxFuture<'a, crate::Result<FieldOutput>>;

    #[allow(clippy::type_complexity)]
    fn subscription_key<'a>(
        &'a mut self,
        headers: http::HeaderMap,
        subgraph_name: &'a str,
        directive: FieldDefinitionDirective<'a>,
    ) -> BoxFuture<'a, Result<(http::HeaderMap, Option<Vec<u8>>), crate::Error>>;

    fn resolve_subscription<'a>(
        &'a mut self,
        headers: http::HeaderMap,
        subgraph_name: &'a str,
        directive: FieldDefinitionDirective<'a>,
    ) -> BoxFuture<'a, Result<(), crate::Error>>;

    fn resolve_next_subscription_item(&mut self) -> BoxFuture<'_, Result<Option<FieldOutput>, crate::Error>>;

    fn authenticate(
        &mut self,
        headers: http::HeaderMap,
    ) -> BoxFuture<'_, crate::GatewayResult<(http::HeaderMap, Token)>>;

    fn authorize_query<'a>(
        &'a mut self,
        ctx: AuthorizationContext,
        elements: QueryElements<'a>,
    ) -> BoxFuture<'a, Result<AuthorizationDecisions, crate::ErrorResponse>>;
}
