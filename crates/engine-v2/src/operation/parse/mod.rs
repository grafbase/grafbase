mod error;
mod location;
mod validation;

use std::collections::HashMap;

use engine_parser::{
    types::{DocumentOperations, FragmentDefinition, OperationDefinition},
    Positioned,
};
use schema::Schema;

pub(crate) use error::*;
pub(crate) use location::*;

pub(crate) struct ParsedOperation {
    pub name: Option<String>,
    pub definition: OperationDefinition,
    pub fragments: HashMap<engine_value::Name, Positioned<FragmentDefinition>>,
}

impl ParsedOperation {
    pub fn get_fragment(&self, name: &str) -> Option<&Positioned<FragmentDefinition>> {
        self.fragments.get(name)
    }
}

/// Returns a valid GraphQL operation from the query string before.
pub(crate) fn parse(schema: &Schema, operation_name: Option<&str>, document: &str) -> ParseResult<ParsedOperation> {
    let document = engine_parser::parse_query(document)?;

    let (name, operation) = if let Some(name) = operation_name {
        match document.operations {
            DocumentOperations::Single(_) => None,
            DocumentOperations::Multiple(mut operations) => operations
                .remove(name)
                .map(|operation| (Some(name.to_string()), operation)),
        }
        .ok_or_else(|| ParseError::UnknowOperation(name.to_string()))?
    } else {
        match document.operations {
            DocumentOperations::Single(operation) => (None, operation),
            DocumentOperations::Multiple(map) if map.len() == 1 => map
                .into_iter()
                .next()
                .map(|(name, operation)| (Some(name.to_string()), operation))
                .unwrap(),
            _ => return Err(ParseError::MissingOperationName),
        }
    };

    let operation = ParsedOperation {
        name,
        definition: operation.node,
        fragments: document.fragments,
    };

    validation::validate(schema, &operation)?;

    Ok(operation)
}
