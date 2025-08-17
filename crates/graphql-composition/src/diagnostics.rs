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
            .filter(|diagnostic| diagnostic.severity.is_warning())
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
        message: impl fmt::Display,
        error_code: CompositeSchemasSourceSchemaValidationErrorCode,
    ) {
        self.0.push(Diagnostic {
            message: format!("[{source_schema_name}] {message}"),
            severity: error_code.severity(),
            error_code: Some(error_code.into()),
        });
    }

    pub(crate) fn push_composite_schemas_pre_merge_validation_error(
        &mut self,
        message: String,
        error_code: CompositeSchemasPreMergeValidationErrorCode,
    ) {
        self.0.push(Diagnostic {
            message,
            severity: error_code.severity(),
            error_code: Some(error_code.into()),
        });
    }

    pub(crate) fn push_composite_schemas_post_merge_validation_error(
        &mut self,
        message: String,
        error_code: CompositeSchemasPostMergeValidationErrorCode,
    ) {
        self.0.push(Diagnostic {
            message,
            severity: error_code.severity(),
            error_code: Some(error_code.into()),
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

    /// The composite schemas error code
    pub fn composite_schemas_error_code(&self) -> Option<CompositeSchemasErrorCode> {
        self.error_code
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

/// Composite Schemas spec [error codes](https://graphql.github.io/composite-schemas-spec/draft/#sec-Schema-Composition).
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum CompositeSchemasErrorCode {
    /// See [CompositeSchemasSourceSchemaValidationErrorCode]
    SourceSchema(CompositeSchemasSourceSchemaValidationErrorCode),
    /// See [CompositeSchemasPreMergeValidationErrorCode]
    PreMerge(CompositeSchemasPreMergeValidationErrorCode),
    /// See [CompositeSchemasPostMergeValidationErrorCode]
    PostMerge(CompositeSchemasPostMergeValidationErrorCode),
}

impl From<CompositeSchemasPostMergeValidationErrorCode> for CompositeSchemasErrorCode {
    fn from(v: CompositeSchemasPostMergeValidationErrorCode) -> Self {
        Self::PostMerge(v)
    }
}

impl From<CompositeSchemasSourceSchemaValidationErrorCode> for CompositeSchemasErrorCode {
    fn from(v: CompositeSchemasSourceSchemaValidationErrorCode) -> Self {
        Self::SourceSchema(v)
    }
}

impl From<CompositeSchemasPreMergeValidationErrorCode> for CompositeSchemasErrorCode {
    fn from(v: CompositeSchemasPreMergeValidationErrorCode) -> Self {
        Self::PreMerge(v)
    }
}

/// Composite Schemas spec [source schema validation](https://graphql.github.io/composite-schemas-spec/draft/#sec-Validate-Source-Schemas) error codes.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum CompositeSchemasSourceSchemaValidationErrorCode {
    /// https://graphql.github.io/composite-schemas-spec/draft/#sec-Query-Root-Type-Inaccessible
    QueryRootTypeInaccessible,
    /// https://graphql.github.io/composite-schemas-spec/draft/#sec-Lookup-Returns-Non-Nullable-Type
    LookupReturnsNonNullableType,
    /// https://graphql.github.io/composite-schemas-spec/draft/#sec-Override-from-Self
    OverrideFromSelf,
}

impl CompositeSchemasSourceSchemaValidationErrorCode {
    fn severity(&self) -> Severity {
        use CompositeSchemasSourceSchemaValidationErrorCode::*;

        match self {
            QueryRootTypeInaccessible | OverrideFromSelf => Severity::Error,

            LookupReturnsNonNullableType => Severity::Warning,
        }
    }
}

/// Composite Schemas spec [pre-merge validation](https://graphql.github.io/composite-schemas-spec/draft/#sec-Pre-Merge-Validation) error codes.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum CompositeSchemasPreMergeValidationErrorCode {
    /// https://graphql.github.io/composite-schemas-spec/draft/#sec-Type-Kind-Mismatch
    TypeKindMismatch,
    /// https://graphql.github.io/composite-schemas-spec/draft/#sec-Override-Source-Has-Override
    OverrideSourceHasOverride,
}

impl CompositeSchemasPreMergeValidationErrorCode {
    fn severity(&self) -> Severity {
        use CompositeSchemasPreMergeValidationErrorCode::*;

        match self {
            TypeKindMismatch | OverrideSourceHasOverride => Severity::Error,
        }
    }
}

/// Composite Schemas spec [post-merge validation](https://graphql.github.io/composite-schemas-spec/draft/#sec-Post-Merge-Validation) error codes.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum CompositeSchemasPostMergeValidationErrorCode {
    /// https://graphql.github.io/composite-schemas-spec/draft/#sec-Invalid-Field-Sharing
    InvalidFieldSharing,
}

impl CompositeSchemasPostMergeValidationErrorCode {
    fn severity(&self) -> Severity {
        use CompositeSchemasPostMergeValidationErrorCode::*;

        match self {
            InvalidFieldSharing => Severity::Error,
        }
    }
}
