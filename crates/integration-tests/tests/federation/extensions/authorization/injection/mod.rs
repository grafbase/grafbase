mod complex;
mod field;
mod interface;
mod object;

use engine::{ErrorResponse, GraphqlError};
use engine_schema::DirectiveSite;
use integration_tests::federation::{AuthorizationTestExtension, DynHookContext};
use runtime::extension::{AuthorizationDecisions, QueryElement, TokenRef};

#[derive(Default)]
pub(super) struct EchoInjections;

#[async_trait::async_trait]
impl AuthorizationTestExtension for EchoInjections {
    #[allow(clippy::manual_async_fn)]
    async fn authorize_query(
        &self,
        wasm_context: &DynHookContext,
        _headers: &tokio::sync::RwLock<http::HeaderMap>,
        _token: TokenRef<'_>,
        elements_grouped_by_directive_name: Vec<(&str, Vec<QueryElement<'_, serde_json::Value>>)>,
    ) -> Result<AuthorizationDecisions, ErrorResponse> {
        wasm_context.insert(
            "query",
            elements_grouped_by_directive_name
                .into_iter()
                .map(|(name, elements)| {
                    let elements = elements
                        .into_iter()
                        .map(|element| (element.site.to_string(), element.arguments))
                        .collect::<serde_json::Map<_, _>>()
                        .into();
                    (name.to_string(), elements)
                })
                .collect::<serde_json::Map<_, _>>(),
        );
        Ok(AuthorizationDecisions::GrantAll)
    }

    async fn authorize_response(
        &self,
        ctx: &DynHookContext,
        directive_name: &str,
        directive_site: DirectiveSite<'_>,
        items: Vec<serde_json::Value>,
    ) -> Result<AuthorizationDecisions, GraphqlError> {
        let data = serde_json::json!({
            "query": ctx.get("query").unwrap_or_default(),
            "response": {
                "directive_name": directive_name,
                "directive_site": directive_site.to_string(),
                "items": items,
            }
        });
        Ok(AuthorizationDecisions::DenyAll(
            GraphqlError::new("Injection time!", engine::ErrorCode::Unauthorized).with_extension("injections", data),
        ))
    }
}
