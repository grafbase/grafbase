//! Composition warnings and errors.

use std::fmt;

/// Warnings and errors produced by composition.
#[derive(Default, Debug)]
pub struct Diagnostics(Vec<Diagnostic>);

impl Diagnostics {
    /// Is any of the diagnostics fatal, i.e. a hard error?
    pub fn any_fatal(&self) -> bool {
        self.0.iter().any(|diagnostic| diagnostic.severity.is_error())
    }

    /// Is there any diagnostic warning or error
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Iterate over all diagnostics.
    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic> {
        self.0.iter()
    }

    /// Iterate non-fatal diagnostics.
    pub fn iter_warnings(&self) -> impl Iterator<Item = &str> {
        self.0
            .iter()
            .filter(|diagnostic| !diagnostic.severity.is_warning())
            .map(|diagnostic| diagnostic.message.as_str())
    }

    /// Iterate fatal diagnostics.
    pub fn iter_errors(&self) -> impl Iterator<Item = &str> {
        self.0
            .iter()
            .filter(|diagnostic| diagnostic.severity.is_error())
            .map(|diagnostic| diagnostic.message.as_str())
    }

    pub(crate) fn clone_all_from(&mut self, other: &Diagnostics) {
        self.0.extend(other.0.iter().cloned())
    }

    /// Iterate over all diagnostic messages.
    pub fn iter_messages(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(|diagnostic| diagnostic.message.as_str())
    }

    pub(crate) fn push_composite_schemas_source_schema_validation_error(
        &mut self,
        source_schema_name: &str,
        message: fmt::Arguments<'_>,
        error_code: CompositeSchemasErrorCode,
    ) {
        self.0.push(Diagnostic {
            message: format!("[{source_schema_name}] {message}"),
            severity: Severity::Error,
            error_code: Some(error_code),
        });
    }

    pub(crate) fn push_fatal(&mut self, message: String) {
        self.0.push(Diagnostic {
            message,
            severity: Severity::Error,
            error_code: None,
        });
    }

    pub(crate) fn push_warning(&mut self, message: String) {
        self.0.push(Diagnostic {
            message,
            severity: Severity::Warning,
            error_code: None,
        });
    }
}

/// A composition diagnostic.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    message: String,
    severity: Severity,
    #[expect(unused)]
    error_code: Option<CompositeSchemasErrorCode>,
}

impl Diagnostic {
    /// The warning or error message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// See [Severity].
    pub fn severity(&self) -> Severity {
        self.severity
    }
}

/// The severity of a [Diagnostic].
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// A fatal error.
    Error,
    /// A warning to be displayed to the user.
    Warning,
}

impl Severity {
    /// Returns `true` if the severity is [`Error`].
    ///
    /// [`Error`]: Severity::Error
    #[must_use]
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error)
    }

    /// Returns `true` if the severity is [`Warning`].
    ///
    /// [`Warning`]: Severity::Warning
    #[must_use]
    pub fn is_warning(&self) -> bool {
        matches!(self, Self::Warning)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CompositeSchemasErrorCode {
    /// https://graphql.github.io/composite-schemas-spec/draft/#sec-Query-Root-Type-Inaccessible
    QueryRootTypeInaccessible,
}
