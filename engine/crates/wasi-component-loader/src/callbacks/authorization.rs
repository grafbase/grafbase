use anyhow::anyhow;
use grafbase_tracing::span::GRAFBASE_TARGET;
use wasmtime::{
    component::{Resource, TypedFunc},
    Store,
};

use crate::{names::AUTHORIZATION_CALLBACK_FUNCTION, state::WasiState, ComponentLoader, ContextMap, ErrorResponse};

pub(crate) type CallbackParameters = (Resource<ContextMap>, Vec<String>);

pub(crate) type CallbackResponse = (Result<Vec<Option<ErrorResponse>>, ErrorResponse>,);

pub struct AuthorizationInstance {
    store: Store<WasiState>,
    context: Resource<ContextMap>,
    callback: Option<TypedFunc<CallbackParameters, CallbackResponse>>,
}

impl AuthorizationInstance {
    pub(crate) async fn new(loader: &ComponentLoader, context: ContextMap) -> crate::Result<Self> {
        let mut store = super::initialize_store(loader.config(), loader.engine())?;

        let context = store.data_mut().push_resource(context)?;

        let instance = loader
            .linker()
            .instantiate_async(&mut store, loader.component())
            .await?;

        let callback = match instance.get_typed_func(&mut store, AUTHORIZATION_CALLBACK_FUNCTION) {
            Ok(callback) => {
                tracing::debug!(target: GRAFBASE_TARGET, "instantized the authorization callback WASM function");

                Some(callback)
            }
            Err(e) => {
                tracing::debug!(target: GRAFBASE_TARGET, "error instantizing the authorization callback WASM function: {e}");

                None
            }
        };

        Ok(Self {
            store,
            context,
            callback,
        })
    }

    pub(crate) async fn call(mut self, input: Vec<String>) -> crate::Result<(ContextMap, Vec<Option<ErrorResponse>>)> {
        let context_rep = self.context.rep();

        let result = match self.callback {
            Some(callback) => callback.call_async(&mut self.store, (self.context, input)).await?.0?,
            None => {
                return Err(crate::Error::Internal(anyhow!(
                    "on-authorization hook must be defined if using the @authorization directive"
                )))
            }
        };

        let context = self.store.data_mut().take_resource(context_rep)?;

        Ok((context, result))
    }
}
