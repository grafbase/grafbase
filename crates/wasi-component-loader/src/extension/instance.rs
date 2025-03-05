use wasmtime::Store;

use super::wit::{self, FieldOutput};
use crate::state::WasiState;

pub struct ExtensionInstance {
    pub(super) store: Store<WasiState>,
    pub(super) inner: wit::Sdk,
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

impl ExtensionInstance {
    pub async fn resolve_field(
        &mut self,
        headers: http::HeaderMap,
        subgraph_name: &str,
        directive: wit::FieldDefinitionDirective<'_>,
        inputs: InputList,
    ) -> crate::Result<FieldOutput> {
        self.poisoned = true;

        let headers = self.store.data_mut().push_resource(wit::Headers::borrow(headers))?;
        let inputs = inputs.0.iter().map(Vec::as_slice).collect::<Vec<_>>();

        let output = self
            .inner
            .grafbase_sdk_extension()
            .call_resolve_field(&mut self.store, headers, subgraph_name, directive, &inputs)
            .await??;

        self.poisoned = false;

        Ok(output)
    }

    pub async fn subscription_key(
        &mut self,
        headers: http::HeaderMap,
        subgraph_name: &str,
        directive: wit::FieldDefinitionDirective<'_>,
    ) -> Result<(http::HeaderMap, Option<Vec<u8>>), crate::Error> {
        self.poisoned = true;

        let headers = self.store.data_mut().push_resource(wit::Headers::borrow(headers))?;
        let headers_rep = headers.rep();

        let key = self
            .inner
            .grafbase_sdk_extension()
            .call_subscription_key(&mut self.store, headers, subgraph_name, directive)
            .await??;

        let headers = self
            .store
            .data_mut()
            .take_resource::<wit::Headers>(headers_rep)?
            .into_owned()
            .unwrap();

        self.poisoned = false;

        Ok((headers, key))
    }

    pub async fn resolve_subscription(
        &mut self,
        headers: http::HeaderMap,
        subgraph_name: &str,
        directive: wit::FieldDefinitionDirective<'_>,
    ) -> Result<(), crate::Error> {
        self.poisoned = true;

        let headers = self.store.data_mut().push_resource(wit::Headers::borrow(headers))?;

        self.inner
            .grafbase_sdk_extension()
            .call_resolve_subscription(&mut self.store, headers, subgraph_name, directive)
            .await??;

        self.poisoned = false;

        Ok(())
    }

    pub async fn resolve_next_subscription_item(&mut self) -> Result<Option<FieldOutput>, crate::Error> {
        self.poisoned = true;

        let output = self
            .inner
            .grafbase_sdk_extension()
            .call_resolve_next_subscription_item(&mut self.store)
            .await??;

        self.poisoned = false;

        Ok(output)
    }

    pub async fn authenticate(
        &mut self,
        headers: http::HeaderMap,
    ) -> crate::GatewayResult<(http::HeaderMap, wit::Token)> {
        self.poisoned = true;

        let headers = self.store.data_mut().push_resource(wit::Headers::borrow(headers))?;
        let headers_rep = headers.rep();

        let token = self
            .inner
            .grafbase_sdk_extension()
            .call_authenticate(&mut self.store, headers)
            .await??;

        let headers = self
            .store
            .data_mut()
            .take_resource::<wit::Headers>(headers_rep)?
            .into_owned()
            .unwrap();

        self.poisoned = false;

        Ok((headers, token))
    }

    pub async fn authorize_query(
        &mut self,
        elements: wit::QueryElements<'_>,
    ) -> Result<wit::AuthorizationDecisions, crate::ErrorResponse> {
        // Futures may be canceled, so we pro-actively mark the instance as poisoned until proven
        // otherwise.
        self.poisoned = true;

        let result = self
            .inner
            .grafbase_sdk_extension()
            .call_authorize_query(&mut self.store, elements)
            .await?;

        self.poisoned = false;
        result.map_err(Into::into)
    }

    pub fn recycle(&mut self) -> crate::Result<()> {
        if self.poisoned {
            return Err(anyhow::anyhow!("this instance is poisoned").into());
        }

        Ok(())
    }
}
