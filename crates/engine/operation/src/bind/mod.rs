mod coercion;
pub mod error;
mod operation;

use std::collections::HashMap;

use coercion::coerce_variable;
use error::{BindError, ErrorOperationName, VariableError};
use id_derives::IndexedFields;
use schema::Schema;

use crate::{
    DataFieldRecord, FieldArgumentRecord, FragmentId, FragmentRecord, FragmentSpreadRecord, InlineFragmentRecord,
    Operation, OperationAttributes, OperationContext, QueryInputValues, RawVariables, ResponseKeys, SelectionId,
    TypenameFieldRecord, VariableDefinitionRecord, VariableInputValues, VariableValueRecord, Variables,
    parse::ParsedOperation,
};

type BindResult<T> = Result<T, BindError>;

#[derive(IndexedFields)]
struct OperationBinder<'schema, 'p> {
    schema: &'schema Schema,
    parsed_operation: &'p ParsedOperation,
    error_operation_name: ErrorOperationName,
    variable_definition_in_use: Vec<bool>,
    fragment_name_to_id: HashMap<&'p str, FragmentId>,
    selection_buffers: Vec<Vec<SelectionId>>,

    response_keys: ResponseKeys,
    data_fields: Vec<DataFieldRecord>,
    typename_fields: Vec<TypenameFieldRecord>,
    variable_definitions: Vec<VariableDefinitionRecord>,
    field_arguments: Vec<FieldArgumentRecord>,
    inline_fragments: Vec<InlineFragmentRecord>,
    fragment_spreads: Vec<FragmentSpreadRecord>,
    #[indexed_by(FragmentId)]
    fragments: Vec<FragmentRecord>,
    query_input_values: QueryInputValues,
    shared_selection_ids: Vec<SelectionId>,

    errors: Vec<BindError>,
}

#[allow(clippy::result_large_err)]
pub(crate) fn bind_operation(
    schema: &Schema,
    parsed_operation: &ParsedOperation,
    attributes: OperationAttributes,
) -> Result<Operation, (Vec<BindError>, OperationAttributes)> {
    let mut binder = OperationBinder {
        schema,
        parsed_operation,
        error_operation_name: ErrorOperationName(parsed_operation.name.clone()),
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
        variable_definition_in_use: Vec::new(),
        fragment_name_to_id: HashMap::with_capacity(parsed_operation.document().fragments().count()),
        selection_buffers: Vec::new(),
        errors: Vec::new(),
    };

    match binder.bind_root() {
        Ok((root_object_id, root_selection_set_record)) => {
            if !binder.errors.is_empty() {
                return Err((binder.errors, attributes));
            }
            let OperationBinder {
                response_keys,
                data_fields,
                typename_fields,
                variable_definitions,
                field_arguments,
                inline_fragments,
                fragment_spreads,
                fragments,
                query_input_values,
                shared_selection_ids,
                ..
            } = binder;

            Ok(Operation {
                attributes,
                root_object_id,
                root_selection_set_record,
                response_keys,
                data_fields,
                typename_fields,
                variable_definitions,
                field_arguments,
                inline_fragments,
                fragment_spreads,
                fragments,
                query_input_values,
                shared_selection_ids,
            })
        }
        Err(err) => {
            binder.errors.push(err);
            Err((binder.errors, attributes))
        }
    }
}

pub(crate) fn bind_variables(
    schema: &Schema,
    operation: &Operation,
    mut request_variables: RawVariables,
) -> Result<Variables, Vec<VariableError>> {
    let ctx = OperationContext { schema, operation };
    let mut errors = Vec::new();
    let mut variables = Variables {
        input_values: VariableInputValues::default(),
        definition_to_value: vec![VariableValueRecord::Undefined; operation.variable_definitions.len()],
    };

    for definition in ctx.variable_definitions() {
        match request_variables.remove(&definition.name) {
            Some(value) => match coerce_variable(schema, &mut variables.input_values, definition, value) {
                Ok(id) => variables[definition.id] = VariableValueRecord::Provided(id),
                Err(err) => {
                    errors.push(VariableError::InvalidValue {
                        name: definition.name.clone(),
                        err,
                    });
                }
            },
            None => {
                if let Some(default_value_id) = definition.default_value_id {
                    variables[definition.id] = VariableValueRecord::DefaultValue(default_value_id);
                } else if definition.ty_record.wrapping.is_required() {
                    errors.push(VariableError::MissingVariable {
                        name: definition.name.clone(),
                        location: definition.name_location,
                    });
                }
            }
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(variables)
}
