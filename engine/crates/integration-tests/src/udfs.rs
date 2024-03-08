use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use runtime::udf::{CustomResolverRequestPayload, UdfError, UdfRequest, UdfResponse};

/// A UdfInvoker implementation that calls into some rust functions.
///
/// Useful for testing the engine parts of Udfs.  It's possible we'll want
/// another implementation of this that calls into some _actual_ JS somehow at some
/// point, but I am not doing that just now.
#[derive(Default)]
#[must_use]
pub struct RustUdfs {
    custom_resolvers: Arc<Mutex<HashMap<String, Box<dyn RustResolver>>>>,
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

/// A trait for resolvers implemented in rust
///
/// This is implemented for:
/// - any Fn with the signature `Fn(CustomResolverRequestPayload) -> Result<CustomResolverResponse, CustomResolverError>`
/// - CustomResolverResponse (if you just want to hard code a response)
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
