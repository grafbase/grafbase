use cynic_parser::Span;
use schema::{
    CompositeTypeId, DirectiveSiteId, EntityDefinitionId, FieldDefinitionId, TypeDefinitionId, UnionDefinitionId,
};

use crate::Location;

use super::{ParsedOperation, coercion::InputValueError};

#[derive(thiserror::Error, Debug)]
pub(crate) enum BindError {
    #[error("Unknown type named '{name}'")]
    UnknownType { name: String, span: Span },
    #[error("The field `{field_name}` does not have an argument named `{argument_name}")]
    UnknownArgument {
        site_id: FieldDefinitionId,
        field_name: String,
        argument_name: String,
        span: Span,
    },
    #[error("{ty} does not have a field named '{name}'.")]
    UnknownField {
        ty: String,
        name: String,
        span: Span,
        site_id: EntityDefinitionId,
    },
    #[error("Unknown fragment named '{name}'.")]
    UnknownFragment { name: String, span: Span },
    #[error(
        "Field '{name}' does not exists on {ty}, it's a union. Only interfaces and objects have fields, consider using a fragment with a type condition."
    )]
    UnionHaveNoFields {
        name: String,
        ty: String,
        span: Span,
        site_id: UnionDefinitionId,
    },
    #[error("Field '{name}' cannot have a selection set, it's a {ty}. Only interfaces, unions and objects can.")]
    CannotHaveSelectionSet {
        name: String,
        ty: String,
        span: Span,
        site_id: TypeDefinitionId,
    },
    #[error("Type conditions cannot be declared on '{name}', only on unions, interfaces or objects.")]
    InvalidTypeConditionTargetType {
        name: String,
        span: Span,
        site_id: TypeDefinitionId,
    },
    #[error("Type condition on '{name}' cannot be used in a '{parent}' selection_set")]
    DisjointTypeCondition {
        parent: String,
        name: String,
        span: Span,
        site_id: CompositeTypeId,
    },
    #[error("Mutations are not defined on this schema.")]
    NoMutationDefined,
    #[error("Subscriptions are not defined on this schema.")]
    NoSubscriptionDefined,
    #[error("Leaf field '{name}' must be a scalar or an enum, but is a {ty}.")]
    LeafMustBeAScalarOrEnum {
        name: String,
        ty: String,
        span: Span,
        site_id: CompositeTypeId,
    },
    #[error(
        "Variable named '${name}' does not have a valid input type. Can only be a scalar, enum or input object. Found: '{ty}'."
    )]
    InvalidVariableType { name: String, ty: String, span: Span },
    #[error("There can only be one variable named '${name}'")]
    DuplicateVariable { name: String, location: Location },
    #[error("Variable '${name}' is not used{operation}")]
    UnusedVariable {
        name: String,
        operation: ErrorOperationName,
        location: Location,
    },
    #[error("{0}")]
    InvalidInputValue(#[from] super::coercion::InputValueError),
    #[error("Missing argument named '{name}' for field '{field}'")]
    MissingArgument {
        field: String,
        name: String,
        span: Span,
        site_id: FieldDefinitionId,
    },
    #[error("Missing argument named '{name}' for directive '{directive}'")]
    MissingDirectiveArgument {
        name: &'static str,
        directive: String,
        span: Span,
    },
}

impl BindError {
    pub fn maybe_location(&self, operation: &ParsedOperation) -> Option<Location> {
        match self {
            BindError::UnknownField { span, .. }
            | BindError::UnknownArgument { span, .. }
            | BindError::UnknownType { span, .. }
            | BindError::UnknownFragment { span, .. }
            | BindError::UnionHaveNoFields { span, .. }
            | BindError::InvalidTypeConditionTargetType { span, .. }
            | BindError::CannotHaveSelectionSet { span, .. }
            | BindError::DisjointTypeCondition { span, .. }
            | BindError::InvalidVariableType { span, .. }
            | BindError::LeafMustBeAScalarOrEnum { span, .. }
            | BindError::MissingArgument { span, .. }
            | BindError::MissingDirectiveArgument { span, .. } => Some(operation.span_to_location(*span)),
            BindError::DuplicateVariable { location, .. } | BindError::UnusedVariable { location, .. } => {
                Some(*location)
            }
            BindError::InvalidInputValue(err) => Some(err.location()),
            BindError::NoMutationDefined | BindError::NoSubscriptionDefined => None,
        }
    }

    pub fn maybe_site_id(&self) -> Option<DirectiveSiteId> {
        match self {
            BindError::UnknownArgument { site_id, .. } => Some((*site_id).into()),
            BindError::UnknownField { site_id, .. } => Some((*site_id).into()),
            BindError::UnionHaveNoFields { site_id, .. } => Some((*site_id).into()),
            BindError::InvalidTypeConditionTargetType { site_id, .. } => Some((*site_id).into()),
            BindError::CannotHaveSelectionSet { site_id, .. } => Some((*site_id).into()),
            BindError::DisjointTypeCondition { site_id, .. } => Some((*site_id).into()),
            BindError::LeafMustBeAScalarOrEnum { site_id, .. } => Some((*site_id).into()),
            BindError::MissingArgument { site_id, .. } => Some((*site_id).into()),
            BindError::NoMutationDefined
            | BindError::NoSubscriptionDefined
            | BindError::InvalidVariableType { .. }
            | BindError::MissingDirectiveArgument { .. }
            | BindError::UnknownType { .. }
            | BindError::UnknownFragment { .. }
            | BindError::DuplicateVariable { .. }
            | BindError::UnusedVariable { .. }
            | BindError::InvalidInputValue(_) => None,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VariableError {
    #[error("Variable ${name} is missing")]
    MissingVariable { name: String, location: Location },
    #[error("Variable ${name} has an invalid value. {err}")]
    InvalidValue { name: String, err: InputValueError },
}

impl VariableError {
    pub fn location(&self) -> Location {
        match self {
            VariableError::MissingVariable { location, .. } => *location,
            VariableError::InvalidValue { err, .. } => err.location(),
        }
    }
}

/// A helper struct for optionally including operation names in error messages
#[derive(Debug, Clone)]
pub(crate) struct ErrorOperationName(pub(super) Option<String>);

impl std::fmt::Display for ErrorOperationName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = &self.0 {
            write!(f, " by operation '{name}'")?;
        }
        Ok(())
    }
}
