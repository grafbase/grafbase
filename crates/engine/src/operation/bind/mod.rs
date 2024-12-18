mod coercion;
mod error;
mod field;
mod location;
mod model;
mod modifier;
mod selection_set;
mod validation;
mod variables;
mod walkers;

use std::collections::HashMap;

use cynic_parser::common::OperationType;
use id_derives::IndexedFields;
use id_newtypes::IdRange;
use modifier::{finalize_query_modifiers, finalize_response_modifiers};
use schema::{CompositeTypeId, Schema};

pub(crate) use error::*;
pub(crate) use location::*;
pub(crate) use model::*;
pub(crate) use variables::*;
pub(crate) use walkers::*;

use super::{ParsedOperation, QueryInputValues};
use crate::response::ResponseKeys;

pub(crate) type BindResult<T> = Result<T, BindError>;

#[derive(IndexedFields)]
pub struct Binder<'schema, 'p> {
    schema: &'schema Schema,
    parsed_operation: &'p ParsedOperation,
    operation_name: ErrorOperationName,
    response_keys: ResponseKeys,
    field_arguments: Vec<BoundFieldArgument>,
    location_to_field_arguments: HashMap<usize, IdRange<BoundFieldArgumentId>>,
    #[indexed_by(BoundFieldId)]
    fields: Vec<BoundField>,
    #[indexed_by(BoundSelectionSetId)]
    selection_sets: Vec<BoundSelectionSet>,
    variable_definition_in_use: Vec<bool>,
    variable_definitions: Vec<BoundVariableDefinition>,
    input_values: QueryInputValues,
    query_modifiers: HashMap<QueryModifierRule, (BoundQueryModifierId, Vec<BoundFieldId>)>,
    response_modifiers: HashMap<ResponseModifierRule, (BoundResponseModifierId, Vec<BoundFieldId>)>,
}

#[tracing::instrument(name = "bind", level = "debug", skip_all)]
pub(crate) fn bind(schema: &Schema, parsed_operation: &ParsedOperation) -> BindResult<BoundOperation> {
    let operation = parsed_operation.operation();
    let root_object_id = match operation.operation_type() {
        OperationType::Query => schema.query().id,
        OperationType::Mutation => schema.mutation().ok_or(BindError::NoMutationDefined)?.id,
        OperationType::Subscription => schema.subscription().ok_or(BindError::NoSubscriptionDefined)?.id,
    };

    let mut binder = Binder {
        schema,
        parsed_operation,
        operation_name: ErrorOperationName(parsed_operation.name.clone()),
        response_keys: ResponseKeys::default(),
        field_arguments: Vec::new(),
        location_to_field_arguments: HashMap::default(),
        fields: Vec::new(),
        selection_sets: Vec::new(),
        variable_definition_in_use: Vec::new(),
        variable_definitions: Vec::new(),
        query_modifiers: Default::default(),
        input_values: QueryInputValues::default(),
        response_modifiers: Default::default(),
    };

    // Must be executed before binding selection sets
    binder.bind_variable_definitions(operation.variable_definitions())?;

    let root_selection_set_id =
        binder.bind_merged_selection_sets(CompositeTypeId::Object(root_object_id), &[operation.selection_set()])?;

    binder.validate_all_variables_used()?;

    let root_query_modifier_ids = binder.generate_modifiers_for_root_object(root_object_id);
    let (query_modifiers, query_modifier_impacted_fields) = finalize_query_modifiers(binder.query_modifiers);
    let (response_modifiers, response_modifier_impacted_fields) =
        finalize_response_modifiers(binder.response_modifiers);

    let operation = BoundOperation {
        ty: match operation.operation_type() {
            OperationType::Query => grafbase_telemetry::graphql::OperationType::Query,
            OperationType::Mutation => grafbase_telemetry::graphql::OperationType::Mutation,
            OperationType::Subscription => grafbase_telemetry::graphql::OperationType::Subscription,
        },
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
    };

    validation::validate(schema, operation.walker_with(schema))?;

    Ok(operation)
}
