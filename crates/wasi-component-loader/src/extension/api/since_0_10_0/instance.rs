use engine::GraphqlError;
use futures::future::BoxFuture;
use runtime::extension::{AuthorizationDecisions, Data};
use wasmtime::Store;

use crate::{Error, ErrorResponse, WasiState, extension::instance::ExtensionInstance};

use crate::extension::api::wit::{FieldDefinitionDirective, Headers, QueryElements, ResponseElements};

pub struct ExtensionInstanceSince0_10_0 {
    pub(crate) store: Store<WasiState>,
    pub(crate) inner: super::wit::Sdk,
    pub(crate) poisoned: bool,
}

impl ExtensionInstance for ExtensionInstanceSince0_10_0 {
    fn recycle(&mut self) -> Result<(), Error> {
        if self.poisoned {
            return Err(anyhow::anyhow!("this instance is poisoned").into());
        }

        Ok(())
    }

    fn resolve_field<'a>(
        &'a mut self,
        headers: http::HeaderMap,
        subgraph_name: &'a str,
        directive: FieldDefinitionDirective<'a>,
        inputs: crate::extension::InputList,
    ) -> BoxFuture<'a, Result<Vec<Result<Data, GraphqlError>>, Error>> {
        Box::pin(async move {
            self.poisoned = true;

            let headers = self.store.data_mut().push_resource(Headers::borrow(headers))?;
            let inputs = inputs.0.iter().map(Vec::as_slice).collect::<Vec<_>>();

            let output = self
                .inner
                .grafbase_sdk_resolver()
                .call_resolve_field(&mut self.store, headers, subgraph_name, directive, &inputs)
                .await??;

            self.poisoned = false;

            Ok(output.into())
        })
    }

    fn subscription_key<'a>(
        &'a mut self,
        headers: http::HeaderMap,
        subgraph_name: &'a str,
        directive: FieldDefinitionDirective<'a>,
    ) -> BoxFuture<'a, Result<(http::HeaderMap, Option<Vec<u8>>), Error>> {
        Box::pin(async move {
            self.poisoned = true;

            let headers = self.store.data_mut().push_resource(Headers::borrow(headers))?;
            let headers_rep = headers.rep();

            let key = self
                .inner
                .grafbase_sdk_resolver()
                .call_subscription_key(&mut self.store, headers, subgraph_name, directive)
                .await??;

            let headers = self
                .store
                .data_mut()
                .take_resource::<Headers>(headers_rep)?
                .into_owned()
                .unwrap();

            self.poisoned = false;

            Ok((headers, key))
        })
    }

    fn resolve_subscription<'a>(
        &'a mut self,
        headers: http::HeaderMap,
        subgraph_name: &'a str,
        directive: FieldDefinitionDirective<'a>,
    ) -> BoxFuture<'a, Result<(), Error>> {
        Box::pin(async move {
            self.poisoned = true;

            let headers = self.store.data_mut().push_resource(Headers::borrow(headers))?;

            self.inner
                .grafbase_sdk_resolver()
                .call_resolve_subscription(&mut self.store, headers, subgraph_name, directive)
                .await??;

            self.poisoned = false;

            Ok(())
        })
    }

    fn resolve_next_subscription_item(
        &mut self,
    ) -> BoxFuture<'_, Result<Option<Vec<Result<Data, GraphqlError>>>, Error>> {
        Box::pin(async move {
            self.poisoned = true;

            let output = self
                .inner
                .grafbase_sdk_resolver()
                .call_resolve_next_subscription_item(&mut self.store)
                .await??;

            self.poisoned = false;

            Ok(output.map(Into::into))
        })
    }

    fn authenticate(
        &mut self,
        headers: http::HeaderMap,
    ) -> BoxFuture<'_, Result<(http::HeaderMap, runtime::extension::Token), ErrorResponse>> {
        Box::pin(async move {
            self.poisoned = true;

            let headers = self.store.data_mut().push_resource(Headers::borrow(headers))?;
            let headers_rep = headers.rep();

            let token = self
                .inner
                .grafbase_sdk_authentication()
                .call_authenticate(&mut self.store, headers)
                .await??;

            let headers = self
                .store
                .data_mut()
                .take_resource::<Headers>(headers_rep)?
                .into_owned()
                .unwrap();

            self.poisoned = false;

            Ok((headers, token.into()))
        })
    }

    fn authorize_query<'a>(
        &'a mut self,
        _ctx: &'a std::sync::Arc<engine::RequestContext>,
        _elements: QueryElements<'a>,
    ) -> BoxFuture<'a, Result<(AuthorizationDecisions, Vec<u8>), ErrorResponse>> {
        todo!()
    }

    fn authorize_response<'a>(
        &'a mut self,
        state: &'a [u8],
        elements: ResponseElements<'a>,
    ) -> BoxFuture<'a, Result<AuthorizationDecisions, Error>> {
        Box::pin(async move {
            // Futures may be canceled, so we pro-actively mark the instance as poisoned until proven
            // otherwise.
            self.poisoned = true;

            let result = self
                .inner
                .grafbase_sdk_authorization()
                .call_authorize_response(&mut self.store, state, elements)
                .await?;

            self.poisoned = false;
            result.map(Into::into).map_err(Into::into)
        })
    }
}
