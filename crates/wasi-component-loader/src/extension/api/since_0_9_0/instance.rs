use futures::future::BoxFuture;
use wasmtime::Store;

use crate::WasiState;

use super::wit::{
    Sdk,
    authorization::{AuthorizationContext, AuthorizationDecisions},
    directive::{FieldDefinitionDirective, QueryElements},
    headers::Headers,
    resolver::FieldOutput,
    token::Token,
};

pub struct ExtensionInstance {
    pub(crate) store: Store<WasiState>,
    pub(crate) inner: Sdk,
    pub(crate) poisoned: bool,
}

/// List of inputs to be provided to the extension.
/// The data itself is fully custom and thus will be serialized with serde to cross the Wasm
/// boundary.
#[derive(Default)]
pub struct InputList(pub(crate) Vec<Vec<u8>>);

impl<S: serde::Serialize> FromIterator<S> for InputList {
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        Self(
            iter.into_iter()
                .map(|input| crate::cbor::to_vec(&input).unwrap())
                .collect(),
        )
    }
}

impl crate::extension::instance::ExtensionInstance for ExtensionInstance {
    fn recycle(&mut self) -> crate::Result<()> {
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
        inputs: InputList,
    ) -> BoxFuture<'a, crate::Result<FieldOutput>> {
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

            Ok(output)
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
    ) -> BoxFuture<'a, Result<(), crate::Error>> {
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

    fn resolve_next_subscription_item(&mut self) -> BoxFuture<'_, Result<Option<FieldOutput>, crate::Error>> {
        Box::pin(async move {
            self.poisoned = true;

            let output = self
                .inner
                .grafbase_sdk_resolver()
                .call_resolve_next_subscription_item(&mut self.store)
                .await??;

            self.poisoned = false;

            Ok(output)
        })
    }

    fn authenticate(
        &mut self,
        headers: http::HeaderMap,
    ) -> BoxFuture<'_, crate::GatewayResult<(http::HeaderMap, Token)>> {
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

            Ok((headers, token))
        })
    }

    fn authorize_query<'a>(
        &'a mut self,
        ctx: AuthorizationContext,
        elements: QueryElements<'a>,
    ) -> BoxFuture<'a, Result<AuthorizationDecisions, crate::ErrorResponse>> {
        Box::pin(async move {
            // Futures may be canceled, so we pro-actively mark the instance as poisoned until proven
            // otherwise.
            self.poisoned = true;
            let ctx = self.store.data_mut().push_resource(ctx)?;

            let result = self
                .inner
                .grafbase_sdk_authorization()
                .call_authorize_query(&mut self.store, ctx, elements)
                .await?;

            self.poisoned = false;

            match result {
                Ok((decisions, _)) => Ok(decisions),
                Err(err) => Err(err.into()),
            }
        })
    }
}
