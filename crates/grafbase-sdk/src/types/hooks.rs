use crate::wit;

/// Output type for the `on_request` hook.
pub struct OnRequestOutput(wit::OnRequestOutput);

impl Default for OnRequestOutput {
    fn default() -> Self {
        Self(wit::OnRequestOutput { contract_key: None })
    }
}

impl OnRequestOutput {
    /// Creates a new `OnRequestOutput` instance with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the contract key for the request.
    pub fn contract_key(&mut self, contract_key: String) -> &mut Self {
        self.0.contract_key = Some(contract_key);
        self
    }
}

impl From<OnRequestOutput> for wit::OnRequestOutput {
    fn from(output: OnRequestOutput) -> Self {
        output.0
    }
}
