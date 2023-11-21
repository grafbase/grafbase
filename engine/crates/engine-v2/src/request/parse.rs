use engine_parser::types::{DocumentOperations, OperationDefinition};

use crate::response::GraphqlError;

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
            ParseError::ParserError(err) => GraphqlError {
                message: err.to_string(),
                locations: err.positions().collect(),
                path: vec![],
            },
            err => GraphqlError {
                message: err.to_string(),
                locations: vec![],
                path: vec![],
            },
        }
    }
}

pub struct UnboundOperation {
    pub name: Option<String>,
    pub definition: OperationDefinition,
}

/// Returns a valid GraphQL operation from the query string before.
pub fn parse_operation(request: &engine::Request) -> ParseResult<UnboundOperation> {
    let document = engine_parser::parse_query(&request.query)?;

    let (operation_name, operation) = if let Some(operation_name) = &request.operation_name {
        match document.operations {
            DocumentOperations::Single(_) => None,
            DocumentOperations::Multiple(mut operations) => operations
                .remove(operation_name.as_str())
                .map(|operation| (Some(operation_name.clone()), operation)),
        }
        .ok_or_else(|| ParseError::UnknowOperation(operation_name.to_string()))?
    } else {
        match document.operations {
            DocumentOperations::Single(operation) => (None, operation),
            DocumentOperations::Multiple(map) => map
                .into_iter()
                .next()
                .map(|(name, operation)| (Some(name.to_string()), operation))
                .ok_or_else(|| ParseError::MissingOperationName)?,
        }
    };

    Ok(UnboundOperation {
        name: operation_name,
        definition: operation.node,
    })
}
