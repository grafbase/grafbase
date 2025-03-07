use futures::future::BoxFuture;
use wasmtime::Store;

use crate::{
    extension::api::{
        instance::InputList,
        wit::{
            authorization::{AuthorizationContext, AuthorizationDecisions},
            directive::QueryElements,
            resolver::{FieldDefinitionDirective, FieldOutput},
            token::Token,
        },
    },
    state::WasiState,
};

use crate::extension::api::since_0_8_0::wit::grafbase::sdk::types;

pub struct ExtensionInstance {
    pub(crate) store: Store<WasiState>,
    pub(crate) inner: super::wit::Sdk,
    pub(crate) poisoned: bool,
}

impl crate::extension::instance::ExtensionInstance for ExtensionInstance {
    fn resolve_field<'a>(
        &'a mut self,
        headers: http::HeaderMap,
        subgraph_name: &'a str,
        directive: FieldDefinitionDirective<'a>,
        inputs: InputList,
    ) -> BoxFuture<'a, crate::Result<FieldOutput>> {
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
    ) -> BoxFuture<'a, Result<(http::HeaderMap, Option<Vec<u8>>), crate::Error>> {
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
    ) -> BoxFuture<'a, Result<(), crate::Error>> {
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

    fn resolve_next_subscription_item(&mut self) -> BoxFuture<'_, Result<Option<FieldOutput>, crate::Error>> {
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
    ) -> BoxFuture<'_, crate::GatewayResult<(http::HeaderMap, Token)>> {
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
    ) -> BoxFuture<'a, Result<AuthorizationDecisions, crate::ErrorResponse>> {
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

            result.map(Into::into).map_err(Into::into)
        })
    }

    fn recycle(&mut self) -> crate::Result<()> {
        if self.poisoned {
            return Err(anyhow::anyhow!("this instance is poisoned").into());
        }

        Ok(())
    }
}
