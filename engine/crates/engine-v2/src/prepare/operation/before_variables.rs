use crate::{
    prepare::{
        error::{PrepareError, PrepareResult},
        CachedOperation, PrepareContext,
    },
    request::Request,
    Runtime,
};

impl<'ctx, R: Runtime> PrepareContext<'ctx, R> {
    pub(super) fn build_cached_operation(
        &mut self,
        request: &Request,
        document: &str,
    ) -> PrepareResult<CachedOperation> {
        let parsed_operation = crate::operation::parse(self.schema(), request.operation_name.as_deref(), document)?;
        let attributes = crate::operation::extract_attributes(&parsed_operation, document);

        let bound_operation = match crate::operation::bind(self.schema(), parsed_operation) {
            Ok(op) => op,
            Err(err) => {
                return Err(PrepareError::Bind {
                    attributes: Box::new(attributes),
                    err,
                })
            }
        };

        let operation_solution = match crate::plan::solve(self.schema(), bound_operation) {
            Ok(op) => op,
            Err(err) => {
                return Err(PrepareError::Plan {
                    attributes: Box::new(attributes),
                    err,
                })
            }
        };

        let attributes = attributes.ok_or(PrepareError::NormalizationError)?;
        Ok(CachedOperation {
            solution: operation_solution,
            attributes,
        })
    }
}
