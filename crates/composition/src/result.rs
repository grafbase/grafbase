use crate::Diagnostics;
use grafbase_federated_graph::FederatedGraph;

/// The result of a [`compose()`](crate::compose()) invocation.
pub struct CompositionResult {
    pub(crate) federated_graph: FederatedGraph,
    pub(crate) diagnostics: Diagnostics,
}

impl CompositionResult {
    /// Simplify the result data to a yes-no answer: did composition succeed?
    ///
    /// `Ok()` contains the [FederatedGraph].
    /// `Err()` contains all [Diagnostics].
    pub fn into_result(self) -> Result<FederatedGraph, Diagnostics> {
        if self.diagnostics.any_fatal() {
            Err(self.diagnostics)
        } else {
            Ok(self.federated_graph)
        }
    }

    /// Composition warnings and errors.
    pub fn diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }
}
