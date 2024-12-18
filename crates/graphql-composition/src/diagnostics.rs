/// Warnings and errors produced by composition.
#[derive(Default, Debug)]
pub struct Diagnostics(Vec<Diagnostic>);

impl Diagnostics {
    pub(crate) fn any_fatal(&self) -> bool {
        self.0.iter().any(|diagnostic| diagnostic.is_fatal)
    }

    pub(crate) fn clone_all_from(&mut self, other: &Diagnostics) {
        self.0.extend(other.0.iter().cloned())
    }

    /// Iterate over all diagnostic messages.
    pub fn iter_messages(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(|diagnostic| diagnostic.message.as_str())
    }

    pub(crate) fn push_warning(&mut self, message: String) {
        self.0.push(Diagnostic {
            message,
            is_fatal: false,
        });
    }

    pub(crate) fn push_fatal(&mut self, message: String) {
        self.0.push(Diagnostic {
            message,
            is_fatal: true,
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
