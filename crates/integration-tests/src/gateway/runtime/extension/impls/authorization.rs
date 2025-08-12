use std::{ops::Range, sync::Arc};

use engine::{ErrorResponse, GraphqlError};
use engine_schema::DirectiveSite;
use extension_catalog::{ExtensionId, Id};
use futures::{FutureExt as _, TryFutureExt as _, TryStreamExt as _, stream::FuturesUnordered};
use runtime::extension::{
    Anything, AuthorizationDecisions, AuthorizationExtension, AuthorizeQuery, QueryAuthorizationDecisions,
    QueryElement, TokenRef,
};
use tokio::sync::RwLock;

use crate::gateway::{
    DispatchRule, EngineTestExtensions, ExtensionsBuilder, TestExtensions, TestManifest,
    runtime::extension::builder::AnyExtension,
};

impl AuthorizationExtension<engine::EngineRequestContext, engine::EngineOperationContext> for EngineTestExtensions {
    fn authorize_query<'ctx, 'fut, Extensions, Arguments>(
        &'ctx self,
        ctx: engine::EngineRequestContext,
        subgraph_headers: http::HeaderMap,
        extensions: Extensions,
        // (directive name, range within query_elements)
        directives: impl ExactSizeIterator<Item = (&'ctx str, Range<usize>)>,
        query_elements: impl ExactSizeIterator<Item = QueryElement<'ctx, Arguments>>,
    ) -> impl Future<Output = Result<AuthorizeQuery, engine::ErrorResponse>> + Send + 'fut
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
                    ctx,
                    subgraph_headers,
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
                    test_extensions,
                    directives,
                    query_elements,
                )
                .boxed()
        }
    }

    fn authorize_response<'ctx, 'fut>(
        &'ctx self,
        ctx: engine::EngineOperationContext,
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
                .authorize_response(ctx, extension_id, directive_name, directive_site, items)
                .boxed(),
            DispatchRule::Test => self
                .test
                .authorize_response(ctx, extension_id, directive_name, directive_site, items)
                .boxed(),
        }
    }
}

impl TestExtensions {
    #[allow(clippy::manual_async_fn)]
    fn authorize_query<'ctx, 'fut, Extensions, Arguments>(
        &'ctx self,
        ctx: engine::EngineRequestContext,
        headers: http::HeaderMap,
        extensions: Extensions,
        // (directive name, range within query_elements)
        directives: impl ExactSizeIterator<Item = (&'ctx str, Range<usize>)>,
        query_elements: impl ExactSizeIterator<Item = QueryElement<'ctx, Arguments>>,
    ) -> impl Future<Output = Result<AuthorizeQuery, ErrorResponse>> + Send + 'fut
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
                subgraph: element.subgraph,
            })
            .collect::<Vec<_>>();

        async move {
            let headers = RwLock::new(headers);
            let ctx_ref = &ctx;
            let headers_ref = &headers;
            let directives = &directives;
            let query_elements = &query_elements;

            let (decisions, state): (Vec<_>, Vec<_>) = extensions
                .into_iter()
                .map(
                    move |(extension_id, directive_range, query_elements_range)| async move {
                        let instance = self.state.lock().await.get_authorization_ext(extension_id);

                        instance
                            .authorize_query(
                                ctx_ref.clone(),
                                headers_ref,
                                ctx_ref.token().as_ref(),
                                directives[directive_range]
                                    .iter()
                                    .map(|(name, range)| (*name, query_elements[range.clone()].to_vec()))
                                    .collect(),
                            )
                            .and_then(|(decisions, state)| async {
                                Ok((
                                    QueryAuthorizationDecisions {
                                        extension_id,
                                        query_elements_range,
                                        decisions,
                                    },
                                    (extension_id, state),
                                ))
                            })
                            .await
                    },
                )
                .collect::<FuturesUnordered<_>>()
                .try_collect::<Vec<_>>()
                .await?
                .into_iter()
                .unzip();

            let headers = headers.into_inner();
            Ok(AuthorizeQuery {
                headers,
                decisions,
                context: Default::default(),
                state,
            })
        }
        .boxed()
    }

    fn authorize_response<'ctx, 'fut>(
        &'ctx self,
        ctx: engine::EngineOperationContext,
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
            let state = ctx
                .authorization_state()
                .iter()
                .find_map(|(id, state)| if *id == extension_id { Some(state) } else { None })
                .cloned()
                .unwrap_or_default();
            instance
                .authorize_response(ctx, &state, directive_name, directive_site, items)
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
                group_by: None,
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
        ctx: engine::EngineRequestContext,
        headers: &RwLock<http::HeaderMap>,
        token: TokenRef<'_>,
        elements_grouped_by_directive_name: Vec<(&str, Vec<QueryElement<'_, serde_json::Value>>)>,
    ) -> Result<(AuthorizationDecisions, Vec<u8>), ErrorResponse>;

    #[allow(clippy::manual_async_fn)]
    async fn authorize_response(
        &self,
        ctx: engine::EngineOperationContext,
        state: &[u8],
        directive_name: &str,
        directive_site: DirectiveSite<'_>,
        items: Vec<serde_json::Value>,
    ) -> Result<AuthorizationDecisions, GraphqlError> {
        Err(GraphqlError::internal_extension_error())
    }
}
