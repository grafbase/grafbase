/// Warnings and errors produced by composition.
#[derive(Default, Debug)]
pub struct Diagnostics(Vec<Diagnostic>);

impl Diagnostics {
    /// Is any of the diagnostics fatal, i.e. a hard error?
    pub fn any_fatal(&self) -> bool {
        self.0.iter().any(|diagnostic| diagnostic.is_fatal)
    }

    /// Is there any diagnostic warning or error
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Iterate non-fatal diagnostics.
    pub fn iter_warnings(&self) -> impl Iterator<Item = &str> {
        self.0
            .iter()
            .filter(|diagnostic| !diagnostic.is_fatal)
            .map(|diagnostic| diagnostic.message.as_str())
    }

    /// Iterate fatal diagnostics.
    pub fn iter_errors(&self) -> impl Iterator<Item = &str> {
        self.0
            .iter()
            .filter(|diagnostic| diagnostic.is_fatal)
            .map(|diagnostic| diagnostic.message.as_str())
    }

    pub(crate) fn clone_all_from(&mut self, other: &Diagnostics) {
        self.0.extend(other.0.iter().cloned())
    }

    /// Iterate over all diagnostic messages.
    pub fn iter_messages(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(|diagnostic| diagnostic.message.as_str())
    }

    pub(crate) fn push_fatal(&mut self, message: String) {
        self.0.push(Diagnostic {
            message,
            is_fatal: true,
        });
    }

    pub(crate) fn push_warning(&mut self, message: String) {
        self.0.push(Diagnostic {
            message,
            is_fatal: false,
        });
    }
}

/// A composition diagnostic.
#[derive(Debug, Clone)]
pub(crate) struct Diagnostic {
    message: String,
    /// Should this diagnostic be interpreted as a composition failure?
    is_fatal: bool,
}
