use anyhow::anyhow;
use futures::future::BoxFuture;
use wasmtime::Store;

use crate::{
    Error, ErrorResponse,
    extension::{
        api::{
            instance::InputList,
            wit::{
                authorization::{AuthorizationContext, AuthorizationDecisions},
                directive::{QueryElements, ResponseElements},
                resolver::{FieldDefinitionDirective, FieldOutput},
                token::Token,
            },
        },
        instance::ExtensionInstance,
    },
    state::WasiState,
};

use crate::extension::api::since_0_8_0::wit::grafbase::sdk::types;

pub struct ExtensionInstanceSince080 {
    pub(crate) store: Store<WasiState>,
    pub(crate) inner: super::wit::Sdk,
    pub(crate) poisoned: bool,
}

impl ExtensionInstance for ExtensionInstanceSince080 {
    fn resolve_field<'a>(
        &'a mut self,
        headers: http::HeaderMap,
        subgraph_name: &'a str,
        directive: FieldDefinitionDirective<'a>,
        inputs: InputList,
    ) -> BoxFuture<'a, Result<FieldOutput, Error>> {
        Box::pin(async move {
            self.poisoned = true;

            let headers = self.store.data_mut().push_resource(types::Headers::borrow(headers))?;
            let inputs = inputs.0.iter().map(Vec::as_slice).collect::<Vec<_>>();

            let output = self
                .inner
                .grafbase_sdk_extension()
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

            let headers = self.store.data_mut().push_resource(types::Headers::borrow(headers))?;

            let headers_rep = headers.rep();

            let key = self
                .inner
                .grafbase_sdk_extension()
                .call_subscription_key(&mut self.store, headers, subgraph_name, directive)
                .await??;

            let headers = self
                .store
                .data_mut()
                .take_resource::<types::Headers>(headers_rep)?
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

            let headers = self.store.data_mut().push_resource(types::Headers::borrow(headers))?;

            self.inner
                .grafbase_sdk_extension()
                .call_resolve_subscription(&mut self.store, headers, subgraph_name, directive)
                .await??;

            self.poisoned = false;

            Ok(())
        })
    }

    fn resolve_next_subscription_item(&mut self) -> BoxFuture<'_, Result<Option<FieldOutput>, Error>> {
        Box::pin(async move {
            self.poisoned = true;

            let output = self
                .inner
                .grafbase_sdk_extension()
                .call_resolve_next_subscription_item(&mut self.store)
                .await??;

            self.poisoned = false;

            Ok(output.map(Into::into))
        })
    }

    fn authenticate(
        &mut self,
        headers: http::HeaderMap,
    ) -> BoxFuture<'_, Result<(http::HeaderMap, Token), ErrorResponse>> {
        Box::pin(async move {
            self.poisoned = true;

            let headers = self.store.data_mut().push_resource(types::Headers::borrow(headers))?;

            let headers_rep = headers.rep();

            let token = self
                .inner
                .grafbase_sdk_extension()
                .call_authenticate(&mut self.store, headers)
                .await??;

            let headers = self
                .store
                .data_mut()
                .take_resource::<types::Headers>(headers_rep)?
                .into_owned()
                .unwrap();

            self.poisoned = false;

            Ok((headers, token.into()))
        })
    }

    fn authorize_query<'a>(
        &'a mut self,
        _: AuthorizationContext,
        elements: QueryElements<'a>,
    ) -> BoxFuture<'a, Result<(AuthorizationDecisions, Vec<u8>), ErrorResponse>> {
        Box::pin(async move {
            // Futures may be canceled, so we pro-actively mark the instance as poisoned until proven
            // otherwise.
            self.poisoned = true;

            let result = self
                .inner
                .grafbase_sdk_extension()
                .call_authorize_query(&mut self.store, elements.into())
                .await?;

            self.poisoned = false;

            result
                .map(|decisions| (decisions.into(), Vec::new()))
                .map_err(Into::into)
        })
    }

    fn authorize_response<'a>(
        &'a mut self,
        _ctx: AuthorizationContext,
        _state: &'a [u8],
        _elements: ResponseElements<'a>,
    ) -> BoxFuture<'a, Result<AuthorizationDecisions, Error>> {
        Box::pin(async move { Err(anyhow!("authorize_response is not supported by sdk 0.8.*").into()) })
    }

    fn recycle(&mut self) -> crate::Result<()> {
        if self.poisoned {
            return Err(anyhow::anyhow!("this instance is poisoned").into());
        }

        Ok(())
    }
}
