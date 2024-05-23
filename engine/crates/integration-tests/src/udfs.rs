use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use runtime::udf::{AuthorizerRequestPayload, CustomResolverRequestPayload, UdfError, UdfRequest, UdfResponse};

/// A UdfInvoker implementation that calls into some rust functions.
///
/// Useful for testing the engine parts of Udfs.  It's possible we'll want
/// another implementation of this that calls into some _actual_ JS somehow at some
/// point, but I am not doing that just now.
#[derive(Default)]
#[must_use]
pub struct RustUdfs {
    custom_resolvers: Arc<Mutex<HashMap<String, Box<dyn RustResolver>>>>,
    authorizers: Arc<Mutex<HashMap<String, Box<dyn RustAuthorizer>>>>,
}

impl RustUdfs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn resolver(self, name: impl Into<String>, resolver: impl RustResolver + 'static) -> Self {
        self.custom_resolvers
            .lock()
            .unwrap()
            .insert(name.into(), Box::new(resolver));
        self
    }

    pub fn authorizer(self, name: impl Into<String>, resolver: impl RustAuthorizer + 'static) -> Self {
        self.authorizers.lock().unwrap().insert(name.into(), Box::new(resolver));
        self
    }
}

#[async_trait::async_trait]
impl runtime::udf::UdfInvokerInner<CustomResolverRequestPayload> for RustUdfs {
    async fn invoke(
        &self,
        _ray_id: &str,
        request: UdfRequest<'_, CustomResolverRequestPayload>,
    ) -> Result<UdfResponse, UdfError>
    where
        CustomResolverRequestPayload: 'async_trait,
    {
        let name = request.name;
        // We're doing a synchronous lock inside an async context here which is sort of bad.
        // But it's tests so yolo: if this causes problems we can fix.
        self.custom_resolvers
            .lock()
            .unwrap()
            .get(name)
            .unwrap_or_else(|| panic!("Resolver named {name} doesn't exist"))
            .invoke(request.payload)
    }
}

#[async_trait::async_trait]
impl runtime::udf::UdfInvokerInner<AuthorizerRequestPayload> for RustUdfs {
    async fn invoke(
        &self,
        _ray_id: &str,
        request: UdfRequest<'_, AuthorizerRequestPayload>,
    ) -> Result<UdfResponse, UdfError>
    where
        AuthorizerRequestPayload: 'async_trait,
    {
        let name = request.name;
        // We're doing a synchronous lock inside an async context here which is sort of bad.
        // But it's tests so yolo: if this causes problems we can fix.
        self.authorizers
            .lock()
            .unwrap()
            .get(name)
            .unwrap_or_else(|| panic!("Authorizer named {name} doesn't exist"))
            .invoke(request.payload)
    }
}

/// A trait for resolvers implemented in rust
///
/// This is implemented for:
/// - any Fn with the signature `Fn(CustomResolverRequestPayload) -> Result<UdfResponse, UdfError>`
/// - UdfResponse if you just want to hard code a response
/// - serde_json::Value if you just want to hardcode a successful response
pub trait RustResolver: Send + Sync {
    fn invoke(&self, payload: CustomResolverRequestPayload) -> Result<UdfResponse, UdfError>;
}

impl<F> RustResolver for F
where
    F: Fn(CustomResolverRequestPayload) -> Result<UdfResponse, UdfError> + Send + Sync,
{
    fn invoke(&self, payload: CustomResolverRequestPayload) -> Result<UdfResponse, UdfError> {
        self(payload)
    }
}

impl RustResolver for UdfResponse {
    fn invoke(&self, _: CustomResolverRequestPayload) -> Result<UdfResponse, UdfError> {
        Ok(self.clone())
    }
}

impl RustResolver for serde_json::Value {
    fn invoke(&self, _: CustomResolverRequestPayload) -> Result<UdfResponse, UdfError> {
        Ok(UdfResponse::Success(self.clone()))
    }
}

/// A trait for authorizers implemented in rust
///
/// This is implemented for:
/// - any Fn with the signature `Fn(AuthorizerRequestPayload) -> Result<UdfResponse, UdfError>`
///
/// At the time of writing this isn't really being used.
///
/// If you start using it you should probably implement more things to make it easier to use,
/// similar to RustResolver above.
pub trait RustAuthorizer: Send + Sync {
    fn invoke(&self, payload: AuthorizerRequestPayload) -> Result<UdfResponse, UdfError>;
}

impl<F> RustAuthorizer for F
where
    F: Fn(AuthorizerRequestPayload) -> Result<UdfResponse, UdfError> + Send + Sync,
{
    fn invoke(&self, payload: AuthorizerRequestPayload) -> Result<UdfResponse, UdfError> {
        self(payload)
    }
}
