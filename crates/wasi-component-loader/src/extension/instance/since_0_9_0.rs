use wasmtime::Store;

use crate::extension::wit;
use crate::extension::wit::since_0_8_0::resolver::{FieldOutput, Headers};
use crate::extension::wit::since_0_9_0::authentication::Token;
use crate::state::WasiState;

pub struct ExtensionInstance {
    pub(super) store: Store<WasiState>,
    pub(super) inner: wit::since_0_9_0::Sdk,
    pub(super) poisoned: bool,
}

/// List of inputs to be provided to the extension.
/// The data itself is fully custom and thus will be serialized with serde to cross the Wasm
/// boundary.
#[derive(Default)]
pub struct InputList(Vec<Vec<u8>>);

impl<S: serde::Serialize> FromIterator<S> for InputList {
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        Self(
            iter.into_iter()
                .map(|input| crate::cbor::to_vec(&input).unwrap())
                .collect(),
        )
    }
}

impl From<wit::since_0_9_0::resolver::FieldOutput> for FieldOutput {
    fn from(value: wit::since_0_9_0::resolver::FieldOutput) -> Self {
        let wit::since_0_9_0::resolver::FieldOutput { outputs } = value;

        Self { outputs }
    }
}

impl super::ExtensionInstance for ExtensionInstance {
    fn recycle(&mut self) -> crate::Result<()> {
        if self.poisoned {
            return Err(anyhow::anyhow!("this instance is poisoned").into());
        }

        Ok(())
    }

    async fn resolve_field(
        &mut self,
        headers: http::HeaderMap,
        subgraph_name: &str,
        directive: wit::FieldDefinitionDirective<'_>,
        inputs: super::since_0_8_0::InputList,
    ) -> crate::Result<FieldOutput> {
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
    }

    async fn subscription_key(
        &mut self,
        headers: http::HeaderMap,
        subgraph_name: &str,
        directive: wit::FieldDefinitionDirective<'_>,
    ) -> Result<(http::HeaderMap, Option<Vec<u8>>), crate::Error> {
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
    }

    async fn resolve_subscription(
        &mut self,
        headers: http::HeaderMap,
        subgraph_name: &str,
        directive: wit::FieldDefinitionDirective<'_>,
    ) -> Result<(), crate::Error> {
        self.poisoned = true;

        let headers = self.store.data_mut().push_resource(Headers::borrow(headers))?;

        self.inner
            .grafbase_sdk_resolver()
            .call_resolve_subscription(&mut self.store, headers, subgraph_name, directive)
            .await??;

        self.poisoned = false;

        Ok(())
    }

    async fn resolve_next_subscription_item(&mut self) -> Result<Option<FieldOutput>, crate::Error> {
        self.poisoned = true;

        let output = self
            .inner
            .grafbase_sdk_resolver()
            .call_resolve_next_subscription_item(&mut self.store)
            .await??;

        self.poisoned = false;

        Ok(output.map(Into::into))
    }

    async fn authenticate(&mut self, headers: http::HeaderMap) -> crate::GatewayResult<(http::HeaderMap, Token)> {
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
    }

    async fn authorize_query<'a>(
        &mut self,
        ctx: wit::since_0_9_0::AuthorizationContext<'a>,
        elements: wit::QueryElements<'a>,
    ) -> Result<wit::since_0_9_0::AuthorizationDecisions, crate::ErrorResponse> {
        todo!()
    }
}

impl ExtensionInstance {
    pub async fn subscription_key(
        &mut self,
        headers: http::HeaderMap,
        subgraph_name: &str,
        directive: wit::FieldDefinitionDirective<'_>,
    ) -> Result<(http::HeaderMap, Option<Vec<u8>>), crate::Error> {
    }

    pub async fn resolve_subscription(
        &mut self,
        headers: http::HeaderMap,
        subgraph_name: &str,
        directive: wit::FieldDefinitionDirective<'_>,
    ) -> Result<(), crate::Error> {
    }

    pub async fn resolve_next_subscription_item(&mut self) -> Result<Option<FieldOutput>, crate::Error> {}

    pub async fn authenticate(
        &mut self,
        headers: http::HeaderMap,
    ) -> crate::GatewayResult<(http::HeaderMap, wit::Token)> {
    }

    pub async fn authorize_query(
        &mut self,
        ctx: wit::since_0_9_0::AuthorizationContext,
        elements: wit::since_0_8_0::QueryElements<'_>,
    ) -> Result<wit::AuthorizationDecisions, crate::ErrorResponse> {
        // Futures may be canceled, so we pro-actively mark the instance as poisoned until proven
        // otherwise.
        self.poisoned = true;
        let ctx = self.store.data_mut().push_resource(ctx)?;

        let result = self
            .inner
            .grafbase_sdk_extension()
            .call_authorize_query(&mut self.store, ctx, elements)
            .await?;

        self.poisoned = false;
        result.map_err(Into::into)
    }

    pub fn recycle(&mut self) -> crate::Result<()> {}
}
