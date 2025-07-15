use std::{ops::Range, sync::Arc};

use engine::{ErrorResponse, GraphqlError};
use engine_schema::DirectiveSite;
use extension_catalog::{ExtensionId, Id};
use futures::{FutureExt as _, TryFutureExt as _, TryStreamExt as _, stream::FuturesUnordered};
use runtime::{
    extension::Anything,
    extension::{AuthorizationDecisions, AuthorizationExtension, QueryAuthorizationDecisions, QueryElement, TokenRef},
};
use tokio::sync::RwLock;

use crate::gateway::{
    DispatchRule, EngineTestExtensions, ExtContext, ExtensionsBuilder, TestExtensions, TestManifest,
    runtime::extension::builder::AnyExtension,
};

impl AuthorizationExtension<ExtContext> for EngineTestExtensions {
    fn authorize_query<'ctx, 'fut, Extensions, Arguments>(
        &'ctx self,
        ctx: &'ctx ExtContext,
        subgraph_headers: http::HeaderMap,
        token: TokenRef<'ctx>,
        extensions: Extensions,
        // (directive name, range within query_elements)
        directives: impl ExactSizeIterator<Item = (&'ctx str, Range<usize>)>,
        query_elements: impl ExactSizeIterator<Item = QueryElement<'ctx, Arguments>>,
    ) -> impl Future<Output = Result<(http::HeaderMap, Vec<QueryAuthorizationDecisions>), engine::ErrorResponse>> + Send + 'fut
    where
        'ctx: 'fut,
        // (extension id, range within directives, range within query_elements)
        Extensions: IntoIterator<
                Item = (ExtensionId, Range<usize>, Range<usize>),
                IntoIter: ExactSizeIterator<Item = (ExtensionId, Range<usize>, Range<usize>)>,
            > + Send
            + Clone
            + 'ctx,
        Arguments: Anything<'ctx>,
    {
        let mut wasm_extensions = Vec::new();
        let mut test_extensions = Vec::new();

        for ext in extensions {
            match self.dispatch[&ext.0] {
                DispatchRule::Wasm => wasm_extensions.push(ext),
                DispatchRule::Test => test_extensions.push(ext),
            }
        }

        assert!(
            wasm_extensions.is_empty() ^ test_extensions.is_empty(),
            "Cannot mix test & wasm authorization extensions (yet?)"
        );

        if !wasm_extensions.is_empty() {
            self.wasm
                .authorize_query(
                    &ctx.wasm,
                    subgraph_headers,
                    token,
                    wasm_extensions,
                    directives,
                    query_elements,
                )
                .boxed()
        } else {
            self.test
                .authorize_query(
                    ctx,
                    subgraph_headers,
                    token,
                    test_extensions,
                    directives,
                    query_elements,
                )
                .boxed()
        }
    }

    fn authorize_response<'ctx, 'fut>(
        &'ctx self,
        ctx: &'ctx ExtContext,
        extension_id: ExtensionId,
        directive_name: &'ctx str,
        directive_site: DirectiveSite<'ctx>,
        items: impl IntoIterator<Item: Anything<'ctx>>,
    ) -> impl Future<Output = Result<AuthorizationDecisions, GraphqlError>> + Send + 'fut
    where
        'ctx: 'fut,
    {
        match self.dispatch[&extension_id] {
            DispatchRule::Wasm => self
                .wasm
                .authorize_response(&ctx.wasm, extension_id, directive_name, directive_site, items)
                .boxed(),
            DispatchRule::Test => self
                .test
                .authorize_response(ctx, extension_id, directive_name, directive_site, items)
                .boxed(),
        }
    }
}

impl AuthorizationExtension<ExtContext> for TestExtensions {
    #[allow(clippy::manual_async_fn)]
    fn authorize_query<'ctx, 'fut, Extensions, Arguments>(
        &'ctx self,
        ctx: &'ctx ExtContext,
        headers: http::HeaderMap,
        token: TokenRef<'ctx>,
        extensions: Extensions,
        // (directive name, range within query_elements)
        directives: impl ExactSizeIterator<Item = (&'ctx str, Range<usize>)>,
        query_elements: impl ExactSizeIterator<Item = QueryElement<'ctx, Arguments>>,
    ) -> impl Future<Output = Result<(http::HeaderMap, Vec<QueryAuthorizationDecisions>), ErrorResponse>> + Send + 'fut
    where
        'ctx: 'fut,
        // (extension id, range within directives, range within query_elements)
        Extensions: IntoIterator<
                Item = (ExtensionId, Range<usize>, Range<usize>),
                IntoIter: ExactSizeIterator<Item = (ExtensionId, Range<usize>, Range<usize>)>,
            > + Send
            + Clone
            + 'ctx,
        Arguments: Anything<'ctx>,
    {
        let directives = directives.collect::<Vec<_>>();

        let query_elements = query_elements
            .map(|element| QueryElement {
                site: element.site,
                arguments: serde_json::to_value(element.arguments).unwrap(),
            })
            .collect::<Vec<_>>();

        async move {
            let headers = RwLock::new(headers);
            let headers_ref = &headers;
            let directives = &directives;
            let query_elements = &query_elements;

            let decisions = extensions
                .into_iter()
                .map(
                    move |(extension_id, directive_range, query_elements_range)| async move {
                        let instance = self.state.lock().await.get_authorization_ext(extension_id);

                        instance
                            .authorize_query(
                                ctx,
                                headers_ref,
                                token,
                                directives[directive_range]
                                    .iter()
                                    .map(|(name, range)| (*name, query_elements[range.clone()].to_vec()))
                                    .collect(),
                            )
                            .and_then(|decisions| async {
                                Ok(QueryAuthorizationDecisions {
                                    extension_id,
                                    query_elements_range,
                                    decisions,
                                })
                            })
                            .await
                    },
                )
                .collect::<FuturesUnordered<_>>()
                .try_collect()
                .await?;
            let headers = headers.into_inner();
            Ok((headers, decisions))
        }
        .boxed()
    }

    fn authorize_response<'ctx, 'fut>(
        &'ctx self,
        ctx: &'ctx ExtContext,
        extension_id: ExtensionId,
        directive_name: &'ctx str,
        directive_site: DirectiveSite<'ctx>,
        items: impl IntoIterator<Item: Anything<'ctx>>,
    ) -> impl Future<Output = Result<AuthorizationDecisions, GraphqlError>> + Send + 'fut
    where
        'ctx: 'fut,
    {
        let items = items
            .into_iter()
            .map(serde_json::to_value)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        async move {
            let instance = self.state.lock().await.get_authorization_ext(extension_id);
            instance
                .authorize_response(ctx, directive_name, directive_site, items)
                .await
        }
        .boxed()
    }
}

pub struct AuthorizationExt {
    instance: Arc<dyn AuthorizationTestExtension>,
    name: &'static str,
    sdl: Option<&'static str>,
}

impl AuthorizationExt {
    pub fn new<T: AuthorizationTestExtension>(instance: T) -> Self {
        Self {
            instance: Arc::new(instance),
            name: "authorization",
            sdl: None,
        }
    }

    #[allow(unused)]
    #[must_use]
    pub fn with_sdl(mut self, sdl: &'static str) -> Self {
        self.sdl = Some(sdl);
        self
    }

    #[allow(unused)]
    #[must_use]
    pub fn with_name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }
}

impl AnyExtension for AuthorizationExt {
    fn register(self, state: &mut ExtensionsBuilder) {
        let id = state.push_test_extension(        TestManifest {
            id: Id {
                name: self.name.to_string(),
                version: "1.0.0".parse().unwrap(),
            },
            r#type: extension_catalog::Type::Authorization(extension_catalog::AuthorizationType {
                directives: None,
            }),
            sdl: self.sdl.or(Some(
                r#"
                extend schema @link(url: "https://specs.grafbase.com/grafbase", import: ["FieldSet", "InputValueSet"])

                scalar JSON

                directive @auth(input: JSON, fields: FieldSet, args: InputValueSet) on FIELD_DEFINITION | OBJECT | INTERFACE | SCALAR | ENUM
                "#,
            )),
        }
        );
        state.test.authorization.insert(id, self.instance);
    }
}

#[allow(unused_variables)] // makes it easier to copy-paste relevant functions
#[async_trait::async_trait]
pub trait AuthorizationTestExtension: Send + Sync + 'static {
    async fn authorize_query(
        &self,
        ctx: &ExtContext,
        headers: &RwLock<http::HeaderMap>,
        token: TokenRef<'_>,
        elements_grouped_by_directive_name: Vec<(&str, Vec<QueryElement<'_, serde_json::Value>>)>,
    ) -> Result<AuthorizationDecisions, ErrorResponse>;

    #[allow(clippy::manual_async_fn)]
    async fn authorize_response(
        &self,
        ctx: &ExtContext,
        directive_name: &str,
        directive_site: DirectiveSite<'_>,
        items: Vec<serde_json::Value>,
    ) -> Result<AuthorizationDecisions, GraphqlError> {
        Err(GraphqlError::internal_extension_error())
    }
}
