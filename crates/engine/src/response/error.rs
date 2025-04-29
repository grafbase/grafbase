pub(crate) use ::error::{ErrorCode, ErrorCodeCounter, ErrorPath, ErrorPathSegment, GraphqlError};
use id_newtypes::BitSet;
use operation::Location;

use crate::prepare::{PreparedOperation, QueryErrorId};

#[derive(Default)]
pub(crate) struct ErrorParts {
    count: usize,
    code_counter: ErrorCodeCounter,
    parts: Vec<ErrorPart>,
}

impl ErrorParts {
    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn from_errors(errors: impl IntoIterator<Item: Into<GraphqlError>>) -> Self {
        let errors = errors.into_iter().map(Into::into).collect::<Vec<_>>();
        let code_counter = ErrorCodeCounter::from_errors(&errors);
        Self {
            count: errors.len(),
            code_counter,
            parts: vec![ErrorPart {
                errors,
                ..Default::default()
            }],
        }
    }

    pub fn code_counter(&self) -> &ErrorCodeCounter {
        &self.code_counter
    }

    pub fn parts(&self) -> &[ErrorPart] {
        &self.parts
    }

    pub fn push(&mut self, part: ErrorPartBuilder<'_>) {
        self.count += part.code_counter.count();
        self.code_counter.add(&part.code_counter);
        self.parts.push(part.inner);
    }
}

#[derive(Default, id_derives::IndexedFields)]
pub(crate) struct ErrorPart {
    errors: Vec<GraphqlError>,
    shared_query_errors: Vec<QueryErrorWithLocationAndPath>,
}

pub(crate) struct QueryErrorWithLocationAndPath {
    pub error_id: QueryErrorId,
    pub location: Location,
    pub path: ErrorPath,
}

impl ErrorPart {
    pub fn errors(&self) -> &[GraphqlError] {
        &self.errors
    }
    pub fn shared_query_errors(&self) -> &[QueryErrorWithLocationAndPath] {
        &self.shared_query_errors
    }
}

pub(crate) struct ErrorPartBuilder<'ctx> {
    operation: &'ctx PreparedOperation,
    query_errors_bitset: BitSet<QueryErrorId>,
    code_counter: ErrorCodeCounter,
    inner: ErrorPart,
}

impl<'ctx> ErrorPartBuilder<'ctx> {
    pub fn new(operation: &'ctx PreparedOperation) -> Self {
        ErrorPartBuilder {
            operation,
            code_counter: ErrorCodeCounter::default(),
            inner: ErrorPart::default(),
            query_errors_bitset: BitSet::with_capacity(operation.plan.query_modifications.errors.len()),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.code_counter.count() == 0
    }

    pub fn len(&self) -> usize {
        self.code_counter.count()
    }

    pub fn push(&mut self, error: impl Into<GraphqlError>) {
        let error: GraphqlError = error.into();
        self.code_counter.increment(error.code);
        self.inner.errors.push(error);
    }

    pub fn push_query_error(&mut self, error_id: QueryErrorId, location: Location, path: impl Into<ErrorPath>) {
        if !self.query_errors_bitset.put(error_id) {
            self.code_counter
                .increment(self.operation.plan.query_modifications[error_id].code);
            self.inner.shared_query_errors.push(QueryErrorWithLocationAndPath {
                error_id,
                location,
                path: path.into(),
            });
        }
    }
}
