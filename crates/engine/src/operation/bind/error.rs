use cynic_parser::Span;

use crate::{response::GraphqlError, ErrorCode};

use super::{Location, ParsedOperation};

#[derive(thiserror::Error, Debug)]
pub enum BindError {
    #[error("Unknown type named '{name}'")]
    UnknownType { name: String, span: Span },
    #[error("The field `{field_name}` does not have an argument named `{argument_name}")]
    UnknownArgument {
        field_name: String,
        argument_name: String,
        span: Span,
    },
    #[error("{container} does not have a field named '{name}'")]
    UnknownField {
        container: String,
        name: String,
        span: Span,
    },
    #[error("Unknown fragment named '{name}'")]
    UnknownFragment { name: String, span: Span },
    #[error("Field '{name}' does not exists on {ty}, it's a union. Only interfaces and objects have fields, consider using a fragment with a type condition.")]
    UnionHaveNoFields { name: String, ty: String, span: Span },
    #[error("Field '{name}' cannot have a selection set, it's a {ty}. Only interfaces, unions and objects can.")]
    CannotHaveSelectionSet { name: String, ty: String, span: Span },
    #[error("Type conditions cannot be declared on '{name}', only on unions, interfaces or objects.")]
    InvalidTypeConditionTargetType { name: String, span: Span },
    #[error("Type condition on '{name}' cannot be used in a '{parent}' selection_set")]
    DisjointTypeCondition { parent: String, name: String, span: Span },
    #[error("Mutations are not defined on this schema.")]
    NoMutationDefined,
    #[error("Subscriptions are not defined on this schema.")]
    NoSubscriptionDefined,
    #[error("Leaf field '{name}' must be a scalar or an enum, but is a {ty}.")]
    LeafMustBeAScalarOrEnum { name: String, ty: String, span: Span },
    #[error(
        "Variable named '${name}' does not have a valid input type. Can only be a scalar, enum or input object. Found: '{ty}'."
    )]
    InvalidVariableType { name: String, ty: String, span: Span },
    #[error("Too many fields selection set.")]
    TooManyFields { span: Span },
    #[error("There can only be one variable named '${name}'")]
    DuplicateVariable { name: String, span: Location },
    #[error("Variable '${name}' is not used{operation}")]
    UnusedVariable {
        name: String,
        operation: ErrorOperationName,
        span: Location,
    },
    #[error("{0}")]
    InvalidInputValue(#[from] super::coercion::InputValueError),
    #[error("Missing argument named '{name}' for field '{field}'")]
    MissingArgument { field: String, name: String, span: Span },
    #[error("Missing argument named '{name}' for directive '{directive}'")]
    MissingDirectiveArgument {
        name: String,
        directive: String,
        span: Span,
    },
    #[error("Query is too high.")]
    QueryTooHigh,
    #[error("GraphQL introspection is not allowed, but the query contained __schema or __type")]
    IntrospectionIsDisabled { span: Location },
}

impl BindError {
    pub fn into_graphql_error(self, operation: &ParsedOperation) -> GraphqlError {
        let locations = match self {
            BindError::UnknownField { span: location, .. }
            | BindError::UnknownArgument { span: location, .. }
            | BindError::UnknownType { span: location, .. }
            | BindError::UnknownFragment { span: location, .. }
            | BindError::UnionHaveNoFields { span: location, .. }
            | BindError::InvalidTypeConditionTargetType { span: location, .. }
            | BindError::CannotHaveSelectionSet { span: location, .. }
            | BindError::DisjointTypeCondition { span: location, .. }
            | BindError::InvalidVariableType { span: location, .. }
            | BindError::TooManyFields { span: location }
            | BindError::LeafMustBeAScalarOrEnum { span: location, .. }
            | BindError::MissingArgument { span: location, .. }
            | BindError::MissingDirectiveArgument { span: location, .. } => vec![operation.span_to_location(location)],
            BindError::DuplicateVariable { span: location, .. }
            | BindError::UnusedVariable { span: location, .. }
            | BindError::IntrospectionIsDisabled { span: location, .. } => vec![location],
            BindError::InvalidInputValue(ref err) => vec![err.location()],
            BindError::NoMutationDefined | BindError::NoSubscriptionDefined | BindError::QueryTooHigh => {
                vec![]
            }
        };
        GraphqlError::new(self.to_string(), ErrorCode::OperationValidationError).with_locations(locations)
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
