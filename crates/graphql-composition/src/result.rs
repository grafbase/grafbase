use crate::{Diagnostics, diagnostics::CompositeSchemasErrorCode, federated_graph::FederatedGraph};

/// The result of a [`compose()`](crate::compose()) invocation.
pub struct CompositionResult {
    pub(crate) federated_graph: Option<FederatedGraph>,
    pub(crate) diagnostics: Diagnostics,
}

impl CompositionResult {
    /// Treat all warnings as fatal.
    #[doc(hidden)]
    pub fn warnings_are_fatal(mut self) -> Self {
        if self.diagnostics.iter().any(|diagnostic| {
            diagnostic.composite_shemas_error_code() != Some(CompositeSchemasErrorCode::LookupReturnsNonNullableType)
        }) {
            self.federated_graph = None;
        }
        self
    }
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
