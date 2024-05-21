use runtime::auth::AccessToken;

use crate::{engine::RequestMetadata, Engine};

/// Data available during the executor life during its build & execution phases.
#[derive(Clone, Copy)]
pub(crate) struct ExecutionContext<'ctx> {
    pub engine: &'ctx Engine,
    pub request_metadata: &'ctx RequestMetadata,
}

impl<'ctx> std::ops::Deref for ExecutionContext<'ctx> {
    type Target = Engine;
    fn deref(&self) -> &'ctx Self::Target {
        self.engine
    }
}

impl<'ctx> ExecutionContext<'ctx> {
    pub fn access_token(&self) -> &'ctx AccessToken {
        &self.request_metadata.access_token
    }

    pub fn headers(&self) -> &'ctx http::HeaderMap {
        &self.request_metadata.headers
    }

    pub fn header(&self, name: &str) -> Option<&'ctx str> {
        self.headers().get(name).and_then(|v| v.to_str().ok())
    }
}
