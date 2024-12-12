mod coercion;
mod error;
mod field;
mod model;
mod modifier;
mod selection_set;
mod validation;
mod variables;
mod walkers;

use std::collections::HashMap;

use cynic_parser::common::OperationType;
use fixedbitset::FixedBitSet;
use id_derives::IndexedFields;
use id_newtypes::IdRange;
use modifier::{finalize_query_modifiers, finalize_response_modifiers};
use schema::{CompositeTypeId, Schema};

pub(crate) use error::*;
pub(crate) use model::*;
pub(crate) use walkers::*;

use crate::{FragmentId, Operation, OperationContext, ResponseKeys};

use super::{ParsedOperation, QueryInputValues};

pub(crate) type BindResult<T> = Result<T, BindError>;

#[derive(IndexedFields)]
pub struct Binder<'schema, 'p> {
    schema: &'schema Schema,
    parsed_operation: &'p ParsedOperation,
    error_operation_name: ErrorOperationName,
    operation: Operation,
    variable_definition_in_use: Vec<bool>,
    fragment_name_to_id: HashMap<&'p str, FragmentId>,
}

pub(crate) fn bind(schema: &Schema, parsed_operation: &ParsedOperation) -> BindResult<Operation> {
    let operation = parsed_operation.operation();

    let mut binder = Binder {
        schema,
        parsed_operation,
        error_operation_name: ErrorOperationName(parsed_operation.name.clone()),
        operation: Operation {
            ty: match operation.operation_type() {
                OperationType::Query => grafbase_telemetry::graphql::OperationType::Query,
                OperationType::Mutation => grafbase_telemetry::graphql::OperationType::Mutation,
                OperationType::Subscription => grafbase_telemetry::graphql::OperationType::Subscription,
            },
            root_object_id: match operation.operation_type() {
                OperationType::Query => schema.query().id,
                OperationType::Mutation => schema.mutation().ok_or(BindError::NoMutationDefined)?.id,
                OperationType::Subscription => schema.subscription().ok_or(BindError::NoSubscriptionDefined)?.id,
            },
            root_selection_set_record: Default::default(),
            response_keys: Default::default(),
            data_fields: Vec::new(),
            typename_fields: Vec::new(),
            variable_definitions: Vec::new(),
            field_arguments: Vec::new(),
            inline_fragments: Vec::new(),
            fragment_spreads: Vec::new(),
            fragments: Vec::new(),
            query_input_values: Default::default(),
            shared_selection_ids: Vec::new(),
        },
        variable_definition_in_use: Vec::new(),
        fragment_name_to_id: HashMap::with_capacity(parsed_operation.document().fragments().count()),
    };

    // Must be executed before binding selection sets
    binder.bind_variable_definitions(operation.variable_definitions())?;
    binder.operation.root_selection_set_record = binder.bind_selection_set(operation.selection_set())?;
    binder.validate_all_variables_used()?;

    let operation = binder.operation;
    validation::validate(OperationContext {
        schema,
        operation: &operation,
    })?;

    Ok(operation)
}
