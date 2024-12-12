#![deny(unused_crate_dependencies)]

use grafbase_workspace_hack as _;

pub mod analytics;
mod attributes;
mod bind;
mod error;
mod model;
mod parse;
mod prelude;
mod request;
mod validation;

pub use bind::error::VariableError;
pub use error::*;
pub use model::*;
pub use request::*;
use schema::Schema;
pub use validation::complexity::{ComplexityCost, ComplexityError};

impl Operation {
    pub fn parse(schema: &Schema, operation_name: Option<&str>, document: &str) -> Result<Operation> {
        let parsed_operation = parse::parse_operation(operation_name, document)?;
        let attributes = attributes::extract_attributes(&parsed_operation);

        if let Err(err) = validation::after_parsing::validate(schema, &parsed_operation) {
            return Err(Error::validation(err.to_string(), attributes)
                .with_location(parsed_operation.span_to_location(err.span())));
        }

        let operation = bind::bind_operation(schema, &parsed_operation, attributes).map_err(|(err, attributes)| {
            Error::validation(err.to_string(), attributes).with_locations(err.location(&parsed_operation))
        })?;

        if let Err(err) = validation::after_binding::validate(schema, &operation) {
            return Err(Error::validation(err.to_string(), operation.attributes).with_locations(err.location()));
        }

        Ok(operation)
    }

    pub fn compute_and_validate_complexity(
        &self,
        schema: &Schema,
        variables: &Variables,
    ) -> std::result::Result<Option<ComplexityCost>, ComplexityError> {
        validation::complexity::compute_and_validate_complexity(
            OperationContext {
                schema,
                operation: self,
            },
            variables,
        )
    }
}

impl Variables {
    pub fn bind(
        schema: &Schema,
        operation: &Operation,
        variables: RawVariables,
    ) -> std::result::Result<Self, Vec<VariableError>> {
        bind::bind_variables(schema, operation, variables)
    }
}
