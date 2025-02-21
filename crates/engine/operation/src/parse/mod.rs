mod error;
mod offsets;

use cynic_parser::{ExecutableDocument, executable::OperationDefinition};
use offsets::LineOffsets;

use self::error::{ParseError, ParseResult};
use super::Location;

pub(crate) struct ParsedOperation {
    pub name: Option<String>,
    document: cynic_parser::ExecutableDocument,
    line_offsets: offsets::LineOffsets,
}

impl ParsedOperation {
    pub fn operation(&self) -> OperationDefinition<'_> {
        match &self.name {
            None => self.document().operations().next().unwrap(),
            Some(name) => self
                .document()
                .operations()
                .find(|operation| operation.name() == Some(name))
                .unwrap(),
        }
    }

    pub fn document(&self) -> &ExecutableDocument {
        &self.document
    }

    pub fn span_to_location(&self, span: cynic_parser::Span) -> Location {
        self.line_offsets
            .span_to_location(span)
            .unwrap_or_else(|| Location::new(0, 0))
    }
}

/// Returns a valid GraphQL operation from the query string before.
#[tracing::instrument(name = "parse", level = "debug", skip_all)]
pub(crate) fn parse_operation(operation_name: Option<&str>, document_str: &str) -> crate::Result<ParsedOperation> {
    let line_offsets = LineOffsets::new(document_str);

    let (name, document) =
        parse_impl(operation_name, document_str).map_err(|err| err.into_graphql_error(&line_offsets))?;

    Ok(ParsedOperation {
        name,
        document,
        line_offsets,
    })
}

fn parse_impl(operation_name: Option<&str>, document_str: &str) -> ParseResult<(Option<String>, ExecutableDocument)> {
    let document = cynic_parser::parse_executable_document(document_str)?;

    let count_operations = document.operations().count();

    let mut name = operation_name.map(ToOwned::to_owned);

    match operation_name {
        None if count_operations > 1 => return Err(ParseError::MissingOperationName),
        None => {
            let operation = document.operations().next().ok_or(ParseError::MissingOperations)?;
            name = operation.name().map(ToOwned::to_owned);
        }
        Some(name) => {
            document
                .operations()
                .find(|operation| operation.name() == Some(name))
                .ok_or_else(|| ParseError::UnknowOperation(name.to_string()))?;
        }
    };

    Ok((name, document))
}
