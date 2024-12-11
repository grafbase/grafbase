use crate::{
    prepare::{
        error::{PrepareError, PrepareResult},
        CachedOperation, PrepareContext,
    },
    request::Request,
    Runtime,
};

impl<R: Runtime> PrepareContext<'_, R> {
    pub(super) fn build_cached_operation(
        &mut self,
        request: &Request,
        document: &str,
    ) -> PrepareResult<CachedOperation> {
        if document.len() >= self.schema().settings.executable_document_limit_bytes {
            return Err(PrepareError::QueryTooBig);
        }

        let parsed_operation = crate::operation::parse(self.schema(), request.operation_name.as_deref(), document)
            .map_err(PrepareError::Parse)?;

        let attributes = crate::operation::extract_attributes(&parsed_operation);

        let bound_operation = match crate::operation::bind(self.schema(), &parsed_operation) {
            Ok(op) => op,
            Err(err) => {
                return Err(PrepareError::Bind {
                    attributes: Box::new(Some(attributes.attributes_for_error())),
                    err: err.into_graphql_error(&parsed_operation),
                });
            }
        };

        let mut operation = None;
        if !self.schema().settings.complexity_control.is_disabled() {
            operation = Some(bound_operation.clone());
        }

        let solved_operation = match crate::operation::solve(self.schema(), bound_operation) {
            Ok(op) => op,
            Err(err) => {
                return Err(PrepareError::Solve {
                    attributes: Box::new(Some(attributes.attributes_for_error())),
                    err,
                })
            }
        };

        Ok(CachedOperation {
            solved: solved_operation,
            attributes,
            operation,
        })
    }
}
