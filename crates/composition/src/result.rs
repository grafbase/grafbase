use crate::Diagnostics;

/// The result of a [`compose()`](crate::compose()) invocation.
pub struct CompositionResult {
    pub(crate) supergraph_sdl: String,
    pub(crate) diagnostics: Diagnostics,
}

impl CompositionResult {
    /// Simplify the result data to a yes-no answer: did composition succeed?
    ///
    /// `Ok()` contains the supergraph SDL.
    /// `Err()` contains all diagnostics.
    pub fn into_result(self) -> Result<String, Diagnostics> {
        if self.diagnostics.any_fatal() {
            Err(self.diagnostics)
        } else {
            Ok(self.supergraph_sdl)
        }
    }

    /// Composition warnings and errors.
    pub fn diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }
}
