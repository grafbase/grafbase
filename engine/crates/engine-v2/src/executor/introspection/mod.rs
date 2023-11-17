use engine_value::ConstValue;
use schema::introspection::{IntrospectionDataSource, IntrospectionQuery, IntrospectionResolver};

use super::{Executor, ExecutorContext, ExecutorError, ExecutorInput, ExecutorOutput};
use crate::{request::OperationSelectionSet, response::ResponseObjectId};

mod resolver;

pub struct IntrospectionExecutor<'a> {
    response_object_id: ResponseObjectId,
    selection_set: &'a OperationSelectionSet,
    data_source: &'a IntrospectionDataSource,
    query: IntrospectionQuery,
}

impl<'a> IntrospectionExecutor<'a> {
    #[allow(clippy::unnecessary_wraps)]
    pub(super) fn build<'ctx, 'input>(
        ctx: ExecutorContext<'ctx>,
        resolver: &IntrospectionResolver,
        selection_set: &'ctx OperationSelectionSet,
        input: ExecutorInput<'input>,
    ) -> Result<Executor<'a>, ExecutorError>
    where
        'ctx: 'a,
    {
        Ok(Executor::Introspection(IntrospectionExecutor {
            response_object_id: input.response_object_roots.id(),
            query: resolver.query,
            selection_set,
            data_source: ctx.engine.schema[resolver.data_source_id].as_introspection().unwrap(),
        }))
    }

    #[allow(clippy::panic)]
    pub(super) async fn execute(
        self,
        ctx: ExecutorContext<'_>,
        output: &mut ExecutorOutput,
    ) -> Result<(), ExecutorError> {
        // There is no IO, we directly write into the response.
        let mut data = output.data.lock().await;
        for field in ctx.default_walker().walk(self.selection_set).all_fields() {
            let mut resolver = resolver::Resolver::new(&ctx.engine.schema, self.data_source, &mut data);
            let value = match self.query {
                IntrospectionQuery::Type => {
                    // There is a single argument if any so don't need to match anything, the
                    // query is already validated.
                    let name = field
                        .arguments()
                        .next()
                        .map(|arg| match arg.resolved_value() {
                            ConstValue::String(s) => s,
                            _ => panic!("Validation failure: Expected string argument"),
                        })
                        .expect("Validation failure: missing argument");
                    resolver.resolve_type_by_name(&name, field.subselection())
                }
                IntrospectionQuery::Schema => resolver.resolve_schema(field.subselection()),
            };
            data.get_mut(self.response_object_id)
                .insert(field.response_position(), field.response_name(), value);
        }
        Ok(())
    }
}
