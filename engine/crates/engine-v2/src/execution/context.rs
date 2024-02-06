use engine::RequestHeaders;

use crate::Engine;

use super::Variables;

/// Data available during the executor life during its build & execution phases.
#[derive(Clone, Copy)]
pub(crate) struct ExecutionContext<'ctx> {
    pub engine: &'ctx Engine,
    pub variables: &'ctx Variables,
    pub(super) request_headers: &'ctx RequestHeaders,
}

impl<'ctx> ExecutionContext<'ctx> {
    pub fn header(&self, name: &str) -> Option<&'ctx str> {
        self.request_headers.find(name)
    }
}
