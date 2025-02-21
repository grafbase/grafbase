mod types;

use anyhow::anyhow;
use http::HeaderMap;
use serde::de::DeserializeOwned;
pub(crate) use types::NatsAuth;
use types::Token;
pub use types::{Directive, ExtensionType, FieldDefinition, FieldOutput};
use wasmtime::component::{ComponentNamedList, Lift, Lower, Resource, TypedFunc};

use super::ComponentInstance;
use crate::{
    ChannelLogSender, ComponentLoader, GuestError, SharedContext,
    error::guest::ErrorResponse,
    headers::HttpHeaders,
    names::{
        AUTEHNTICATE_EXTENSION_FUNCTION, INIT_GATEWAY_EXTENSION_FUNCTION, REGISTER_EXTENSION_FUNCTION,
        RESOLVE_FIELD_EXTENSION_FUNCTION,
    },
};

/// An instance of an extensions component.
pub struct ExtensionsComponentInstance {
    component: ComponentInstance,
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

impl ExtensionsComponentInstance {
    /// Creates a new extension component instance.
    pub async fn new(
        loader: &ComponentLoader,
        r#type: ExtensionType,
        schema_directives: Vec<Directive>,
        configuration: Vec<u8>,
        access_log: ChannelLogSender,
    ) -> crate::Result<Self> {
        let mut component = ComponentInstance::new(loader, access_log).await?;

        let register = component
            .get_typed_func::<(), ()>(REGISTER_EXTENSION_FUNCTION)
            .ok_or_else(|| anyhow!("register-extension function not found"))?;

        register.call_async(component.store_mut(), ()).await?;
        register.post_return_async(component.store_mut()).await?;

        let mut this = Self { component };

        this.init_gateway_extension(r#type, schema_directives, configuration)
            .await?;

        Ok(this)
    }

    /// A field resolver extension call.
    pub async fn resolve_field(
        &mut self,
        context: SharedContext,
        directive: Directive,
        definition: FieldDefinition,
        inputs: InputList,
    ) -> crate::Result<FieldOutput> {
        type Params = (Resource<SharedContext>, Directive, FieldDefinition, Vec<Vec<u8>>);
        type Response = Result<FieldOutput, GuestError>;

        let context = self.component.store_mut().data_mut().push_resource(context)?;
        let context_rep = context.rep();

        let result = self
            .call_typed_func::<Params, Response>(
                RESOLVE_FIELD_EXTENSION_FUNCTION,
                (context, directive, definition, inputs.0),
            )
            .await?;

        self.component
            .store_mut()
            .data_mut()
            .take_resource::<SharedContext>(context_rep)?;

        Ok(result?)
    }

    /// Performs authentication based on the provided request headers.
    pub async fn authenticate<S>(&mut self, headers: HeaderMap) -> crate::GatewayResult<(HeaderMap, S)>
    where
        S: DeserializeOwned,
    {
        type Params = (Resource<HttpHeaders>,);
        type Response = Result<Token, ErrorResponse>;

        let headers = self
            .component
            .store_mut()
            .data_mut()
            .push_resource(HttpHeaders::from(headers))?;
        let headers_rep = headers.rep();

        let result = self
            .call_typed_func::<Params, Response>(AUTEHNTICATE_EXTENSION_FUNCTION, (headers,))
            .await?;

        let headers = self
            .component
            .store_mut()
            .data_mut()
            .take_resource::<HttpHeaders>(headers_rep)?
            .into_owned()
            .unwrap();

        let result = result?.deserialize()?;

        Ok((headers, result))
    }

    async fn init_gateway_extension(
        &mut self,
        r#type: ExtensionType,
        schema_directives: Vec<Directive>,
        configuration: Vec<u8>,
    ) -> crate::Result<()> {
        type Params = (ExtensionType, Vec<Directive>, Vec<u8>);

        let params = (r#type, schema_directives, configuration);

        let result = self
            .call_typed_func::<Params, Result<(), String>>(INIT_GATEWAY_EXTENSION_FUNCTION, params)
            .await?;

        Ok(result.map_err(|e| anyhow!(e))?)
    }

    async fn call_typed_func<Params, Response>(
        &mut self,
        function_name: &'static str,
        params: Params,
    ) -> crate::Result<Response>
    where
        Params: ComponentNamedList + Lower + Send + Sync + 'static,
        Response: Lift + Send + Sync + 'static,
    {
        let func = self.get_typed_func::<Params, (Response,)>(function_name).unwrap();

        let result = func.call_async(self.component.store_mut(), params).await;

        if result.is_err() {
            self.component.poisoned = true;
        } else {
            func.post_return_async(self.component.store_mut()).await?;
        }

        Ok(result?.0)
    }

    fn get_typed_func<Params, Results>(
        &mut self,
        function_name: &'static str,
    ) -> crate::Result<TypedFunc<Params, Results>>
    where
        Params: ComponentNamedList + Lower + Send + Sync + 'static,
        Results: ComponentNamedList + Lift + Send + Sync + 'static,
    {
        let func = self
            .component
            .get_typed_func(function_name)
            .ok_or_else(|| anyhow!("{function_name} function not found"))?;

        Ok(func)
    }

    /// Checks if the instance can be recycled.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure. On success, it returns `Ok(())`.
    /// On failure, it returns an error if the instance is poisoned.
    pub fn recycle(&mut self) -> crate::Result<()> {
        if self.component.poisoned() {
            return Err(anyhow!("this instance is poisoned").into());
        }

        Ok(())
    }
}
