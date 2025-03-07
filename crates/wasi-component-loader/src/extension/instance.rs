use since_0_9_0::InputList;

use super::wit::{self, since_0_9_0::resolver::FieldOutput};

pub mod since_0_8_0;
pub mod since_0_9_0;

pub(crate) trait ExtensionInstance {
    fn recycle(&mut self) -> crate::Result<()>;

    async fn resolve_field(
        &mut self,
        headers: http::HeaderMap,
        subgraph_name: &str,
        directive: wit::FieldDefinitionDirective<'_>,
        inputs: InputList,
    ) -> crate::Result<FieldOutput>;

    async fn subscription_key(
        &mut self,
        headers: http::HeaderMap,
        subgraph_name: &str,
        directive: wit::FieldDefinitionDirective<'_>,
    ) -> Result<(http::HeaderMap, Option<Vec<u8>>), crate::Error>;

    async fn resolve_subscription(
        &mut self,
        headers: http::HeaderMap,
        subgraph_name: &str,
        directive: wit::FieldDefinitionDirective<'_>,
    ) -> Result<(), crate::Error>;

    async fn resolve_next_subscription_item(&mut self) -> Result<Option<FieldOutput>, crate::Error>;

    async fn authenticate(
        &mut self,
        headers: http::HeaderMap,
    ) -> crate::GatewayResult<(http::HeaderMap, wit::since_0_9_0::token::Token)>;

    async fn authorize_query<'a>(
        &mut self,
        ctx: wit::since_0_9_0::AuthorizationContext<'a>,
        elements: wit::QueryElements<'a>,
    ) -> Result<wit::since_0_9_0::AuthorizationDecisions, crate::ErrorResponse>;
}
