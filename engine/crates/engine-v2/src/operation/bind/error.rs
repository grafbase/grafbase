use crate::{
    operation::{Location, LocationError},
    response::GraphqlError,
    ErrorCode,
};

#[derive(thiserror::Error, Debug)]
pub enum BindError {
    #[error("Unknown type named '{name}'")]
    UnknownType { name: String, location: Location },
    #[error("The field `{field_name}` does not have an argument named `{argument_name}")]
    UnknownArgument {
        field_name: String,
        argument_name: String,
        location: Location,
    },
    #[error("{container} does not have a field named '{name}'")]
    UnknownField {
        container: String,
        name: String,
        location: Location,
    },
    #[error("Unknown fragment named '{name}'")]
    UnknownFragment { name: String, location: Location },
    #[error("Field '{name}' does not exists on {ty}, it's a union. Only interfaces and objects have fields, consider using a fragment with a type condition.")]
    UnionHaveNoFields {
        name: String,
        ty: String,
        location: Location,
    },
    #[error("Field '{name}' cannot have a selection set, it's a {ty}. Only interfaces, unions and objects can.")]
    CannotHaveSelectionSet {
        name: String,
        ty: String,
        location: Location,
    },
    #[error("Type conditions cannot be declared on '{name}', only on unions, interfaces or objects.")]
    InvalidTypeConditionTargetType { name: String, location: Location },
    #[error("Type condition on '{name}' cannot be used in a '{parent}' selection_set")]
    DisjointTypeCondition {
        parent: String,
        name: String,
        location: Location,
    },
    #[error("Mutations are not defined on this schema.")]
    NoMutationDefined,
    #[error("Subscriptions are not defined on this schema.")]
    NoSubscriptionDefined,
    #[error("Leaf field '{name}' must be a scalar or an enum, but is a {ty}.")]
    LeafMustBeAScalarOrEnum {
        name: String,
        ty: String,
        location: Location,
    },
    #[error(
        "Variable named '${name}' does not have a valid input type. Can only be a scalar, enum or input object. Found: '{ty}'."
    )]
    InvalidVariableType {
        name: String,
        ty: String,
        location: Location,
    },
    #[error("Too many fields selection set.")]
    TooManyFields { location: Location },
    #[error("There can only be one variable named '${name}'")]
    DuplicateVariable { name: String, location: Location },
    #[error("Variable '${name}' is not used{operation}")]
    UnusedVariable {
        name: String,
        operation: ErrorOperationName,
        location: Location,
    },
    #[error("Query is too big: {0}")]
    QueryTooBig(#[from] LocationError),
    #[error("{0}")]
    InvalidInputValue(#[from] super::coercion::InputValueError),
    #[error("Missing argument named '{name}' for field '{field}'")]
    MissingArgument {
        field: String,
        name: String,
        location: Location,
    },
    #[error("Missing argument named '{name}' for directive '{directive}'")]
    MissingDirectiveArgument {
        name: String,
        directive: String,
        location: Location,
    },
    #[error("Query is too high.")]
    QueryTooHigh,
    #[error("GraphQL introspection is not allowed, but the query contained __schema or __type")]
    IntrospectionIsDisabled { location: Location },
}

impl From<BindError> for GraphqlError {
    fn from(err: BindError) -> Self {
        let locations = match err {
            BindError::UnknownField { location, .. }
            | BindError::UnknownArgument { location, .. }
            | BindError::UnknownType { location, .. }
            | BindError::UnknownFragment { location, .. }
            | BindError::UnionHaveNoFields { location, .. }
            | BindError::InvalidTypeConditionTargetType { location, .. }
            | BindError::CannotHaveSelectionSet { location, .. }
            | BindError::DisjointTypeCondition { location, .. }
            | BindError::InvalidVariableType { location, .. }
            | BindError::TooManyFields { location }
            | BindError::LeafMustBeAScalarOrEnum { location, .. }
            | BindError::DuplicateVariable { location, .. }
            | BindError::MissingArgument { location, .. }
            | BindError::MissingDirectiveArgument { location, .. }
            | BindError::UnusedVariable { location, .. }
            | BindError::IntrospectionIsDisabled { location, .. } => vec![location],
            BindError::InvalidInputValue(ref err) => vec![err.location()],
            BindError::NoMutationDefined
            | BindError::NoSubscriptionDefined
            | BindError::QueryTooBig { .. }
            | BindError::QueryTooHigh => {
                vec![]
            }
        };
        GraphqlError::new(err.to_string(), ErrorCode::OperationValidationError).with_locations(locations)
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
