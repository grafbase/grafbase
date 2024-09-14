use std::collections::HashMap;

use engine_parser::{
    types::{DocumentOperations, OperationDefinition},
    Positioned,
};

use crate::response::{ErrorCode, GraphqlError};

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("Unknown operation named '{0}'.")]
    UnknowOperation(String),
    #[error("Missing operation name.")]
    MissingOperationName,
    #[error(transparent)]
    ParserError(#[from] engine_parser::Error),
}

pub type ParseResult<T> = Result<T, ParseError>;

impl From<ParseError> for GraphqlError {
    fn from(err: ParseError) -> Self {
        match err {
            ParseError::ParserError(err) => GraphqlError::new(err.to_string(), ErrorCode::OperationParsingError)
                .with_locations(err.positions().filter_map(|pos| pos.try_into().ok())),
            err => GraphqlError::new(err.to_string(), ErrorCode::OperationParsingError),
        }
    }
}

pub struct ParsedOperation {
    /// The name of the operation, if specified.
    pub name: Option<String>,

    /// The definition of the operation.
    pub definition: OperationDefinition,

    /// A mapping of fragment names to their definitions.
    pub fragments: HashMap<engine_value::Name, Positioned<engine_parser::types::FragmentDefinition>>,
}

impl ParsedOperation {
    pub fn get_fragment(&self, name: &str) -> Option<&Positioned<engine_parser::types::FragmentDefinition>> {
        self.fragments.get(name)
    }
}

/// Parses a GraphQL operation from the provided document string.
///
/// This function takes an optional operation name and the document string,
/// attempting to parse the operation from the string. If the operation name
/// is provided, it will look for that specific operation. If no name is
/// specified, it will attempt to extract a single operation from the document.
///
/// # Arguments
///
/// * `operation_name` - An optional string slice representing the name
///   of the operation to parse.
/// * `document` - A string slice containing the GraphQL document to parse.
///
/// # Returns
///
/// This function returns a `ParseResult<ParsedOperation>`, which is either
/// a `ParsedOperation` if the parsing is successful, or a `ParseError`
/// if there is an error in parsing the operation.
pub fn parse_operation(operation_name: Option<&str>, document: &str) -> ParseResult<ParsedOperation> {
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

    Ok(ParsedOperation {
        name,
        definition: operation.node,
        fragments: document.fragments,
    })
}
