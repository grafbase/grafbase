use graphql_federated_graph::FederatedGraph;

use crate::Diagnostics;

/// The result of a [`compose()`](crate::compose()) invocation.
pub struct CompositionResult {
    pub(crate) federated_graph: Option<FederatedGraph>,
    pub(crate) diagnostics: Diagnostics,
}

impl CompositionResult {
    /// Simplify the result data to a yes-no answer: did composition succeed?
    ///
    /// `Ok()` contains the [FederatedGraph].
    /// `Err()` contains all [Diagnostics].
    pub fn into_result(self) -> Result<FederatedGraph, Diagnostics> {
        if let Some(federated_graph) = self.federated_graph {
            Ok(federated_graph)
        } else {
            // means a fatal error occured
            Err(self.diagnostics)
        }
    }

    /// Composition warnings and errors.
    pub fn diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }
}
