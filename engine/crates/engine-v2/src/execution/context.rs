use engine::RequestHeaders;
use runtime::auth::AccessToken;

use crate::Engine;

/// Data available during the executor life during its build & execution phases.
#[derive(Clone, Copy)]
pub(crate) struct ExecutionContext<'ctx> {
    pub engine: &'ctx Engine,
    pub headers: &'ctx RequestHeaders,
    pub access_token: &'ctx AccessToken,
}

impl<'ctx> ExecutionContext<'ctx> {
    pub fn header(&self, name: &str) -> Option<&'ctx str> {
        self.headers.find(name)
    }
}
