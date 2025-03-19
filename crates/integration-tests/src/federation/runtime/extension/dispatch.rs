use std::{collections::HashMap, ops::Range, sync::Arc};

use engine::{ErrorResponse, GraphqlError};
use engine_schema::DirectiveSite;
use extension_catalog::ExtensionId;
use futures::{FutureExt as _, stream::BoxStream};
use runtime::{
    extension::{
        AuthorizationDecisions, AuthorizerId, Data, ExtensionFieldDirective, ExtensionRuntime,
        QueryAuthorizationDecisions, QueryElement, Token, TokenRef,
    },
    hooks::Anything,
};
use wasi_component_loader::{AccessLogReceiver, extension::WasmExtensions};

use crate::federation::ExtContext;

use super::TestExtensions;

pub enum DispatchRule {
    Wasm,
    Test,
}

pub struct ExtensionsDispatcher {
    pub(super) dispatch: HashMap<ExtensionId, DispatchRule>,
    pub(super) test: TestExtensions,
    pub(super) wasm: WasmExtensions,
    pub access_log_receiver: AccessLogReceiver,
}

impl Default for ExtensionsDispatcher {
    fn default() -> Self {
        let (_, receiver) = crossbeam::channel::bounded(1);
        Self {
            dispatch: Default::default(),
            test: Default::default(),
            wasm: Default::default(),
            access_log_receiver: receiver,
        }
    }
}

impl ExtensionRuntime for ExtensionsDispatcher {
    type SharedContext = ExtContext;

    fn resolve_field<'ctx, 'resp, 'f>(
        &'ctx self,
        subgraph_headers: http::HeaderMap,
        directive: ExtensionFieldDirective<'ctx, impl Anything<'ctx>>,
        inputs: impl Iterator<Item: Anything<'resp>> + Send,
    ) -> impl Future<Output = Result<Vec<Result<Data, GraphqlError>>, GraphqlError>> + Send + 'f
    where
        'ctx: 'f,
    {
        match self.dispatch[&directive.extension_id] {
            DispatchRule::Wasm => self.wasm.resolve_field(subgraph_headers, directive, inputs).boxed(),
            DispatchRule::Test => self.test.resolve_field(subgraph_headers, directive, inputs).boxed(),
        }
    }

    async fn resolve_subscription<'ctx, 'f>(
        &'ctx self,
        subgraph_headers: http::HeaderMap,
        directive: ExtensionFieldDirective<'ctx, impl Anything<'ctx>>,
    ) -> Result<BoxStream<'f, Result<Arc<Data>, GraphqlError>>, GraphqlError>
    where
        'ctx: 'f,
    {
        match self.dispatch[&directive.extension_id] {
            DispatchRule::Wasm => self.wasm.resolve_subscription(subgraph_headers, directive).await,
            DispatchRule::Test => self.test.resolve_subscription(subgraph_headers, directive).await,
        }
    }

    async fn authenticate(
        &self,
        extension_id: ExtensionId,
        authorizer_id: AuthorizerId,
        gateway_headers: http::HeaderMap,
    ) -> Result<(http::HeaderMap, Token), ErrorResponse> {
        match self.dispatch[&extension_id] {
            DispatchRule::Wasm => {
                self.wasm
                    .authenticate(extension_id, authorizer_id, gateway_headers)
                    .await
            }
            DispatchRule::Test => {
                self.test
                    .authenticate(extension_id, authorizer_id, gateway_headers)
                    .await
            }
        }
    }

    fn authorize_query<'ctx, 'fut, Extensions, Arguments>(
        &'ctx self,
        wasm_context: &'ctx Self::SharedContext,
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
                    &wasm_context.wasm,
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
                    &wasm_context.test,
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
        extension_id: ExtensionId,
        wasm_context: &'ctx Self::SharedContext,
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
                .authorize_response(extension_id, &wasm_context.wasm, directive_name, directive_site, items)
                .boxed(),
            DispatchRule::Test => self
                .test
                .authorize_response(extension_id, &wasm_context.test, directive_name, directive_site, items)
                .boxed(),
        }
    }
}
