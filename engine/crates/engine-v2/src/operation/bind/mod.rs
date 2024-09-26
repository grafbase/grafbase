mod coercion;
mod field;
mod modifier;
mod selection_set;
mod validation;
mod variables;

use std::collections::HashMap;

pub use engine_parser::types::OperationType;
use id_derives::IndexedFields;
use id_newtypes::IdRange;
use itertools::Itertools;
use modifier::{finalize_query_modifiers, finalize_response_modifiers};
use schema::Schema;
use validation::validate_parsed_operation;

use super::{
    parse::ParsedOperation, FieldId, QueryInputValues, QueryModifierId, QueryModifierRule, ResponseModifierId,
    ResponseModifierRule,
};
use crate::{
    operation::SelectionSetType,
    operation::{
        Field, FieldArgument, FieldArgumentId, Location, Operation, SelectionSet, SelectionSetId, VariableDefinition,
    },
    response::{ErrorCode, GraphqlError, ResponseKeys},
};
pub use variables::*;

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
    #[error("Fragment cycle detected: {}", .cycle.iter().join(", "))]
    FragmentCycle { cycle: Vec<String>, location: Location },
    #[error("Query is too big: {0}")]
    QueryTooBig(String),
    #[error("{0}")]
    InvalidInputValue(#[from] coercion::InputValueError),
    #[error("Missing argument named '{name}' for field '{field}'")]
    MissingArgument {
        field: String,
        name: String,
        location: Location,
    },
    #[error("Query is too complex.")]
    QueryTooComplex { complexity: usize, location: Location },
    #[error("Query is nested too deep.")]
    QueryTooDeep { depth: usize, location: Location },
    #[error("Query contains too many root fields.")]
    QueryContainsTooManyRootFields { count: usize, location: Location },
    #[error("Query contains too many aliases.")]
    QueryContainsTooManyAliases { count: usize, location: Location },
    #[error("Missing argument named '{name}' for directive '{directive}'")]
    MissingDirectiveArgument {
        name: String,
        directive: String,
        location: Location,
    },
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
            | BindError::FragmentCycle { location, .. }
            | BindError::MissingArgument { location, .. }
            | BindError::MissingDirectiveArgument { location, .. }
            | BindError::UnusedVariable { location, .. }
            | BindError::QueryTooComplex { location, .. }
            | BindError::QueryTooDeep { location, .. }
            | BindError::QueryContainsTooManyAliases { location, .. }
            | BindError::QueryContainsTooManyRootFields { location, .. } => vec![location],
            BindError::InvalidInputValue(ref err) => vec![err.location()],
            BindError::NoMutationDefined | BindError::NoSubscriptionDefined | BindError::QueryTooBig { .. } => {
                vec![]
            }
        };
        GraphqlError::new(err.to_string(), ErrorCode::OperationValidationError).with_locations(locations)
    }
}

pub type BindResult<T> = Result<T, BindError>;

#[derive(IndexedFields)]
pub(crate) struct Binder<'schema, 'p> {
    schema: &'schema Schema,
    parsed_operation: &'p ParsedOperation,
    operation_name: ErrorOperationName,
    response_keys: ResponseKeys,
    field_arguments: Vec<FieldArgument>,
    location_to_field_arguments: HashMap<Location, IdRange<FieldArgumentId>>,
    #[indexed_by(FieldId)]
    fields: Vec<Field>,
    #[indexed_by(SelectionSetId)]
    selection_sets: Vec<SelectionSet>,
    variable_definitions: Vec<VariableDefinition>,
    input_values: QueryInputValues,
    query_modifiers: HashMap<QueryModifierRule, (QueryModifierId, Vec<FieldId>)>,
    response_modifiers: HashMap<ResponseModifierRule, (ResponseModifierId, Vec<FieldId>)>,
}

pub fn bind_operation(schema: &Schema, mut parsed_operation: ParsedOperation) -> BindResult<Operation> {
    validate_parsed_operation(&parsed_operation, &schema.settings.operation_limits)?;

    let root_object_id = match parsed_operation.definition.ty {
        OperationType::Query => schema.query().id(),
        OperationType::Mutation => schema.mutation().ok_or(BindError::NoMutationDefined)?.id(),
        OperationType::Subscription => schema.subscription().ok_or(BindError::NoSubscriptionDefined)?.id(),
    };

    let variable_definitions = std::mem::take(&mut parsed_operation.definition.variable_definitions);
    let mut binder = Binder {
        schema,
        parsed_operation: &parsed_operation,
        operation_name: ErrorOperationName(parsed_operation.name.clone()),
        response_keys: ResponseKeys::default(),
        field_arguments: Vec::new(),
        location_to_field_arguments: HashMap::default(),
        fields: Vec::new(),
        selection_sets: Vec::new(),
        variable_definitions: Vec::new(),
        query_modifiers: Default::default(),
        input_values: QueryInputValues::default(),
        response_modifiers: Default::default(),
    };

    // Must be executed before binding selection sets
    binder.variable_definitions = binder.bind_variable_definitions(variable_definitions)?;

    let root_selection_set_id = binder.bind_merged_selection_sets(
        SelectionSetType::Object(root_object_id),
        &[&parsed_operation.definition.selection_set],
    )?;

    binder.validate_all_variables_used()?;

    let root_query_modifier_ids = binder.generate_modifiers_for_root_object(root_object_id);
    let (query_modifiers, query_modifier_impacted_fields) = finalize_query_modifiers(binder.query_modifiers);
    let (response_modifiers, response_modifier_impacted_fields) =
        finalize_response_modifiers(binder.response_modifiers);

    Ok(Operation {
        ty: parsed_operation.definition.ty,
        root_object_id,
        root_query_modifier_ids,
        root_selection_set_id,
        selection_sets: binder.selection_sets,
        field_arguments: binder.field_arguments,
        response_keys: binder.response_keys,
        fields: binder.fields,
        variable_definitions: binder.variable_definitions,
        query_input_values: binder.input_values,
        query_modifiers,
        query_modifier_impacted_fields,
        response_modifiers,
        response_modifier_impacted_fields,
    })
}

/// A helper struct for optionally including operation names in error messages
#[derive(Debug, Clone)]
pub(crate) struct ErrorOperationName(Option<String>);

impl std::fmt::Display for ErrorOperationName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = &self.0 {
            write!(f, " by operation '{name}'")?;
        }
        Ok(())
    }
}
