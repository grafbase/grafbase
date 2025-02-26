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
                .map(|input| minicbor_serde::to_vec(&input).unwrap())
                .collect(),
        )
    }
}

impl ExtensionInstance {
    pub async fn resolve_field(
        &mut self,
        context: wit::SharedContext,
        directive: wit::Directive<'_>,
        definition: wit::FieldDefinition<'_>,
        inputs: InputList,
    ) -> crate::Result<FieldOutput> {
        let context = self.store.data_mut().push_resource(context)?;
        let inputs = inputs.0.iter().map(Vec::as_slice).collect::<Vec<_>>();

        let result = self
            .inner
            .grafbase_sdk_extension()
            .call_resolve_field(&mut self.store, context, directive, definition, &inputs)
            .await;

        match result {
            Ok(output) => output.map_err(|e| e.into()),
            Err(e) => {
                self.poisoned = true;
                Err(e.into())
            }
        }
    }

    pub async fn resolve_subscription(
        &mut self,
        context: wit::SharedContext,
        directive: wit::Directive<'_>,
        definition: wit::FieldDefinition<'_>,
    ) -> Result<(), crate::Error> {
        let context = self.store.data_mut().push_resource(context)?;

        let result = self
            .inner
            .grafbase_sdk_extension()
            .call_resolve_subscription(&mut self.store, context, directive, definition)
            .await;

        match result {
            Ok(output) => output.map_err(Into::into),
            Err(e) => {
                self.poisoned = true;
                Err(e.into())
            }
        }
    }

    pub async fn resolve_next_subscription_item(&mut self) -> Result<Option<FieldOutput>, crate::Error> {
        let result = self
            .inner
            .grafbase_sdk_extension()
            .call_resolve_next_subscription_item(&mut self.store)
            .await;

        match result {
            Ok(output) => output.map_err(Into::into),
            Err(e) => {
                self.poisoned = true;
                Err(e.into())
            }
        }
    }

    pub async fn authenticate(
        &mut self,
        headers: http::HeaderMap,
    ) -> crate::GatewayResult<(http::HeaderMap, wit::Token)> {
        let headers = self.store.data_mut().push_resource(wit::Headers::borrow(headers))?;
        let headers_rep = headers.rep();

        let result = self
            .inner
            .grafbase_sdk_extension()
            .call_authenticate(&mut self.store, headers)
            .await;

        let headers = self
            .store
            .data_mut()
            .take_resource::<wit::Headers>(headers_rep)?
            .into_owned()
            .unwrap();
        match result {
            Ok(result) => result.map(|token| (headers, token)).map_err(Into::into),
            Err(e) => {
                self.poisoned = true;
                Err(e.into())
            }
        }
    }

    pub fn recycle(&mut self) -> crate::Result<()> {
        if self.poisoned {
            return Err(anyhow::anyhow!("this instance is poisoned").into());
        }

        Ok(())
    }
}
